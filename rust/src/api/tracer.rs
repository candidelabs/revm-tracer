use std::str::FromStr;

use crate::trace::{
    block::{create_block_env_from_block_details, BlockDetails},
    database::AccountDetails,
    trace::{trace_transaction, trace_transaction_op},
    error::TraceError,
};
use revm::{context::BlockEnv, primitives::{Bytes, HashMap, Address}};

/// Formats and traces a transaction, returning the result as a JSON string
///
/// This is the main entry point for Flutter/Dart via flutter_rust_bridge.
/// All errors are converted to JSON error responses for graceful handling on the client side.
///
/// # Arguments
///
/// * `chain_id` - The chain ID
/// * `from` - Sender address as hex string
/// * `from_nonce` - Sender's nonce
/// * `to` - Recipient address as hex string
/// * `data` - Transaction data as hex string
/// * `gas_limit` - Gas limit
/// * `gas_price` - Gas price in wei
/// * `gas_priority_fee` - Priority fee in wei
/// * `latest_block_env` - Block environment as JSON string
/// * `prestate_tracer_result` - Prestate as JSON string
/// * `is_op_stack` - If true, use Optimism tracer; if false, use standard Ethereum tracer
///
/// # Returns
///
/// JSON string containing either:
/// - Success: The trace result
/// - Error: An error object with details
#[flutter_rust_bridge::frb(sync)]
pub fn format_and_trace_transaction(
    chain_id: u64,
    from: &str,
    from_nonce: u64,
    to: &str,
    data: &str,
    gas_limit: u64,
    gas_price: u128,
    gas_priority_fee: u128,
    latest_block_env: &str,
    prestate_tracer_result: &str,
    is_op_stack: bool,
) -> String {
    match format_and_trace_transaction_internal(
        chain_id,
        from,
        from_nonce,
        to,
        data,
        gas_limit,
        gas_price,
        gas_priority_fee,
        latest_block_env,
        prestate_tracer_result,
        is_op_stack,
    ) {
        Ok(result) => result,
        Err(e) => {
            // Return error as JSON for client-side handling
            serde_json::json!({
                "error": true,
                "message": e.to_string(),
                "type": format!("{:?}", e)
            }).to_string()
        }
    }
}

/// Internal function that does the actual work with proper error handling
fn format_and_trace_transaction_internal(
    chain_id: u64,
    from: &str,
    from_nonce: u64,
    to: &str,
    data: &str,
    gas_limit: u64,
    gas_price: u128,
    gas_priority_fee: u128,
    latest_block_env: &str,
    prestate_tracer_result: &str,
    is_op_stack: bool,
) -> Result<String, TraceError> {
    // Parse block details from JSON
    let latest_block: BlockDetails = serde_json::from_str(latest_block_env)?;
    let latest_block_env: BlockEnv = create_block_env_from_block_details(latest_block)?;

    // Parse prestate from JSON
    let prestate_tracer_result: HashMap<Address, AccountDetails> =
        serde_json::from_str(prestate_tracer_result)?;

    // Parse addresses
    let from_address = from.parse()
        .map_err(|_| TraceError::InvalidAddress(from.to_string()))?;
    let to_address = to.parse()
        .map_err(|_| TraceError::InvalidAddress(to.to_string()))?;

    // Parse calldata
    let data_bytes = Bytes::from_str(data)
        .map_err(|_| TraceError::InvalidHexData(data.to_string()))?;

    // Execute trace based on chain type
    let json = if is_op_stack {
        // Use Optimism tracer for OP Stack chains
        let result = trace_transaction_op(
            chain_id,
            from_address,
            from_nonce,
            to_address,
            data_bytes,
            gas_limit,
            gas_price,
            gas_priority_fee,
            latest_block_env,
            prestate_tracer_result,
        )?;
        serde_json::to_string_pretty(&result)?
    } else {
        // Use standard Ethereum tracer
        let result = trace_transaction(
            chain_id,
            from_address,
            from_nonce,
            to_address,
            data_bytes,
            gas_limit,
            gas_price,
            gas_priority_fee,
            latest_block_env,
            prestate_tracer_result,
        )?;
        serde_json::to_string_pretty(&result)?
    };

    Ok(json)
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}
