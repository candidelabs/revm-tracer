use revm::primitives::{Address, Bytes, U256};
use std::str::FromStr;
use revm::context::BlockEnv;
use revm::primitives::HashMap;

// Import from the library
use revm_tracer::trace::{
    database::AccountDetails,
    trace::trace_transaction,
};

fn main() {
    println!("=== REVM Transaction Tracer Example ===\n");
    // Setup: Define addresses
    let from_address = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();
    let to_address = Address::from_str("0x0987654321098765432109876543210987654321").unwrap();

    // Setup: Create a simple prestate with account balances
    let mut prestate: HashMap<Address, AccountDetails> = HashMap::default();

    // From account with balance and nonce
    prestate.insert(
        from_address,
        AccountDetails {
            balance: Some(U256::from(1_000_000_000_000_000_000u64)), // 1 ETH in wei
            nonce: Some(5),
            code: None,
            storage: None,
        },
    );

    // To account (can be empty for simple transfers)
    prestate.insert(
        to_address,
        AccountDetails {
            balance: Some(U256::from(500_000_000_000_000_000u64)), // 0.5 ETH in wei
            nonce: Some(0),
            code: None,
            storage: None,
        },
    );


    // Setup: Create block environment
    let block_env = BlockEnv {
        number: U256::from(18_000_000),
        beneficiary: Address::from_str("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(),
        timestamp: U256::from(1_700_000_000u64),
        gas_limit: 30_000_000u64,
        basefee: 20_000_000_000u64, // 20 gwei
        difficulty: U256::ZERO,
        prevrandao: Some(revm::primitives::B256::from([0u8; 32])),
        blob_excess_gas_and_price: Some(
            revm::context_interface::block::BlobExcessGasAndPrice::new(1, 1)
        ),
    };

    // Setup: Transaction parameters
    let chain_id = 1u64; // Ethereum mainnet
    let from_nonce = 5u64;
    let gas_limit = 21_000u64; // Standard ETH transfer
    let gas_price = 25_000_000_000u128; // 25 gwei
    let gas_priority_fee = 2_000_000_000u128; // 2 gwei priority fee

    // Simple ETH transfer (empty data)
    let data = Bytes::new();

    println!("Transaction Details:");
    println!("  Chain ID: {}", chain_id);
    println!("  From: {:?}", from_address);
    println!("  To: {:?}", to_address);
    println!("  Nonce: {}", from_nonce);
    println!("  Gas Limit: {}", gas_limit);
    println!("  Gas Price: {} gwei", gas_price / 1_000_000_000);
    println!("  Priority Fee: {} gwei", gas_priority_fee / 1_000_000_000);
    // println!("  Data: 0x{}", hex::encode(data));
    println!("\nBlock Details:");
    println!("  Number: {}", block_env.number);
    println!("  Timestamp: {}", block_env.timestamp);
    // println!("  Base Fee: {} gwei\n", block_env.basefee / U256::from(1_000_000_000u64));

    // Execute the trace
    println!("Executing transaction trace...\n");

    match trace_transaction(
        chain_id,
        from_address,
        from_nonce,
        to_address,
        data,
        gas_limit,
        gas_price,
        gas_priority_fee,
        block_env,
        prestate,
    ) {
        Ok(result) => {
            println!("=== Trace Result ===\n");
            let json = serde_json::to_string_pretty(&result).unwrap();
            println!("{}", json);

            println!("\n=== Summary ===");
            println!("Call Type: {}", result.calls.call_type);
            println!("From: {:?}", result.calls.from);
            println!("To: {:?}", result.calls.to);
            println!("Value: {}", result.calls.value);
            println!("Gas Used: {}", result.calls.gas_used);
            println!("Success: {}", result.calls.error.is_none());

            if let Some(error) = &result.calls.error {
                println!("Error: {}", error);
            }
            if let Some(revert_reason) = &result.calls.revert_reason {
                println!("Revert Reason: {}", revert_reason);
            }

            println!("Subcalls: {}", result.calls.calls.len());
        }
        Err(e) => {
            eprintln!("Error tracing transaction: {}", e);
        }
    }
}
