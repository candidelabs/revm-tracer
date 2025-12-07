use revm::context::result::{ExecutionResult, HaltReason};
use revm::context::BlockEnv;
use revm::context::CfgEnv;
use revm::context::JournalTr;
use revm::handler::instructions::EthInstructions;
use revm::handler::EthPrecompiles;
use revm::primitives::HashMap;
use revm::primitives::TxKind;
use revm::{ExecuteEvm, MainnetEvm};
use revm::InspectEvm;

use serde::{Serialize, Deserialize};

use revm::{
    context::TxEnv,
    primitives::{Address, Bytes, B256, U256},
    Context,
    MainContext,
};

// Optimism-specific imports
use op_revm::{
    L1BlockInfo,
    OpContext,
    OpEvm,
    OpSpecId,
    OpTransaction,
    OpHaltReason,
};
use revm::context::LocalContext;
use revm::Journal;

use crate::trace::database::create_in_memory_database_from_prestate_trace;
use crate::trace::database::AccountDetails;
use crate::trace::inspector::{CallFrame, CallTracer};
use crate::trace::error::TraceError;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceTransactionResult<T> {
    pub execution_result: ExecutionResult<T>,
    pub state_diff: HashMap<Address, revm::state::Account>,
    pub calls: CallFrame,
}

/// Trace a transaction execution with detailed call information
///
/// # Arguments
///
/// * `chain_id` - The chain ID for the transaction
/// * `from` - The sender address
/// * `from_nonce` - The sender's nonce
/// * `to` - The recipient address
/// * `data` - The transaction calldata
/// * `gas_limit` - Maximum gas allowed for execution
/// * `gas_price` - Gas price in wei
/// * `gas_priority_fee` - Priority fee in wei
/// * `latest_block_env` - Block environment for execution
/// * `prestate_tracer_result` - Account states before execution
///
/// # Returns
///
/// Returns a `TraceTransactionResult` containing execution details, state changes, and call trace
///
/// # Errors
///
/// Returns `TraceError` if:
/// - Transaction environment cannot be built
/// - Transaction execution fails
/// - No trace result is available from the inspector
pub fn trace_transaction(
    chain_id: u64,
    from: Address,
    from_nonce: u64,
    to: Address,
    data: Bytes,
    gas_limit: u64,
    gas_price: u128,
    gas_priority_fee: u128,
    latest_block_env: BlockEnv,
    prestate_tracer_result: HashMap<Address, AccountDetails>
) -> Result<TraceTransactionResult<HaltReason>, TraceError> {
    // Build transaction environment - errors are automatically converted via From trait
    let tx = TxEnv::builder()
        .chain_id(Some(chain_id))
        .caller(from)
        .kind(TxKind::Call(to))
        .nonce(from_nonce)
        .gas_limit(gas_limit)
        .gas_price(gas_price)
        .gas_priority_fee(Some(gas_priority_fee))
        .data(data)
        .build()?;

    let inspector = CallTracer::new();

    // Create in-memory database from prestate
    let db = create_in_memory_database_from_prestate_trace(prestate_tracer_result);

    // Configure EVM with chain settings
    let mut cfg_env = CfgEnv::new().with_chain_id(chain_id);
    cfg_env.disable_eip3607 = true;

    // Setup execution context
    let context = Context::mainnet()
        .with_db(db)
        .with_cfg(cfg_env)
        .with_block(latest_block_env);

    let mut my_evm = MainnetEvm::new_with_inspector(
        context,
        inspector,
        EthInstructions::new_mainnet(),
        EthPrecompiles::default()
    );

    // Execute transaction and collect trace
    let execution_result = my_evm.inspect_one_tx(tx)
        .map_err(|e| TraceError::Execution(e.to_string()))?;

    // Get state changes from the EVM context
    let state_diff = my_evm.ctx.journaled_state.state.clone();

    let inspector = my_evm.inspector;
    let calls = inspector.into_result()
        .ok_or(TraceError::NoTraceResult)?;

    Ok(TraceTransactionResult {
        execution_result,
        state_diff,
        calls
    })
}

/// Trace an Optimism transaction execution with detailed call information
///
/// This function is specifically for Optimism (OP Stack) chains and uses op-revm.
/// It provides the same tracing capabilities as `trace_transaction` but with
/// Optimism-specific transaction handling and context.
///
/// # Arguments
///
/// * `chain_id` - The chain ID (e.g., 10 for OP Mainnet, 420 for OP Goerli)
/// * `from` - The sender address
/// * `from_nonce` - The sender's nonce
/// * `to` - The recipient address
/// * `data` - The transaction calldata
/// * `gas_limit` - Maximum gas allowed for execution
/// * `gas_price` - Gas price in wei
/// * `gas_priority_fee` - Priority fee in wei
/// * `latest_block_env` - Block environment for execution
/// * `prestate_tracer_result` - Account states before execution
/// * `op_spec` - Optimism specification version (e.g., Bedrock, Canyon, Delta)
/// * `l1_block_info` - Optional L1 block information for L1 fee calculation
///
/// # Returns
///
/// Returns a `TraceTransactionResult` containing execution details, state changes, and call trace
///
/// # Errors
///
/// Returns `TraceError` if:
/// - Transaction environment cannot be built
/// - Transaction execution fails
/// - No trace result is available from the inspector
///
/// # Example
///
/// ```ignore
/// use op_revm::OpSpecId;
/// let result = trace_transaction_op(
///     10,  // OP Mainnet
///     from_address,
///     nonce,
///     to_address,
///     calldata,
///     gas_limit,
///     gas_price,
///     priority_fee,
///     block_env,
///     prestate,
///     OpSpecId::CANYON,
///     None,  // No custom L1 block info
/// )?;
/// ```
pub fn trace_transaction_op(
    chain_id: u64,
    from: Address,
    from_nonce: u64,
    to: Address,
    data: Bytes,
    gas_limit: u64,
    gas_price: u128,
    gas_priority_fee: u128,
    latest_block_env: BlockEnv,
    prestate_tracer_result: HashMap<Address, AccountDetails>,
) -> Result<TraceTransactionResult<OpHaltReason>, TraceError> {
    // Build base transaction environment
    let base_tx = TxEnv::builder()
        .chain_id(Some(chain_id))
        .caller(from)
        .kind(TxKind::Call(to))
        .nonce(from_nonce)
        .gas_limit(gas_limit)
        .gas_price(gas_price)
        .gas_priority_fee(Some(gas_priority_fee))
        .data(data);

    // Build Optimism-specific transaction
    // mint: 0 for regular transactions (L1->L2 deposit amount)
    // source_hash: Identifier for the L1 transaction that triggered this (dummy for user transactions)
    let op_tx = OpTransaction::builder()
        .base(base_tx)
        .enveloped_tx(None)
        .not_system_transaction()
        .mint(0u128)
        .source_hash(B256::from([1u8; 32]))
        .build()?;

    let inspector = CallTracer::new();

    // Create in-memory database from prestate
    let db = create_in_memory_database_from_prestate_trace(prestate_tracer_result);

    // Configure EVM with chain settings
    let cfg_env = CfgEnv::new().with_chain_id(chain_id);
    let spec_id = cfg_env.spec;

    // Setup Optimism-specific configuration
    let op_spec = OpSpecId::default();
    let mut chain = L1BlockInfo::default();

    // Isthmus upgrade requires operator fee parameters
    if op_spec == OpSpecId::ISTHMUS {
        chain.operator_fee_constant = Some(U256::from(0));
        chain.operator_fee_scalar = Some(U256::from(0));
    }

    let op_cfg = cfg_env.with_spec(op_spec);

    // Setup Optimism execution context
    let op_context = OpContext {
        journaled_state: {
            let mut journal = Journal::new(db);
            journal.set_spec_id(spec_id);
            journal
        },
        block: latest_block_env,
        cfg: op_cfg,
        tx: OpTransaction::default(), // Will be set by inspect_one_tx
        chain,
        local: LocalContext::default(),
        error: Ok(()),
    };

    let mut my_evm = OpEvm::new(op_context, inspector);

    // Execute transaction and collect trace
    let execution_result = my_evm.inspect_one_tx(op_tx)
        .map_err(|e| TraceError::Execution(e.to_string()))?;

    // Finalize to get state changes
    let state_diff = my_evm.finalize();

    // Extract call trace from inspector
    let inspector = my_evm.into_inspector();
    let calls = inspector.into_result()
        .ok_or(TraceError::NoTraceResult)?;

    Ok(TraceTransactionResult {
        execution_result,
        state_diff,
        calls
    })
}