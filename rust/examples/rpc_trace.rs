/// RPC-based Transaction Tracer Example
///
/// This example fetches real blockchain data from an RPC node and traces a transaction.
/// Configure the parameters below before running.
///
/// Usage: cargo run --example rpc_trace

use revm::primitives::{Address, Bytes, U256, HashMap};
use revm::context::BlockEnv;
use std::str::FromStr;
use serde_json::json;

use revm_tracer::trace::{
    database::AccountDetails,
    trace::trace_transaction,
    block::BlockDetails,
};

// ============================================================================
// CONFIGURATION - EDIT THESE VALUES BEFORE RUNNING
// ============================================================================

/// RPC endpoint URL (must support debug_traceCall)
/// Examples:
/// - Local node: "http://localhost:8545"
/// - Alchemy: "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
/// - Public: "https://eth.llamarpc.com"
const RPC_URL: &str = "https://opt-mainnet.g.alchemy.com/v2/4c-qn3iFsGpZ35gzUyPYz";

/// Transaction parameters
const FROM_ADDRESS: &str = "0x6D37817D118f72F362cf01e64D9454bDD8E8E92F";
const TO_ADDRESS: &str = "0x94b008aA00579c1307B0EF2c499aD98a8ce58e58";

/// Value to transfer in wei (0 for no ETH transfer)
const VALUE_WEI: &str = "0";

/// Calldata in hex format
/// Examples:
/// - Empty call: "0x"
/// - ERC20 transfer: "0xa9059cbb000000000000000000000000RECIPIENT0000000000000000000000000000000000000000000000000000000000000000000000000000000064"
const CALLDATA: &str = "0xa9059cbb000000000000000000000000b139265251e61e72a29ee5649202827f554db999000000000000000000000000000000000000000000000000000000000536e2dc";

/// Block number to trace at
/// Use "latest" or a specific number like "18000000"
const BLOCK_NUMBER: &str = "latest";

/// Chain ID (1 = Ethereum Mainnet, 11155111 = Sepolia, etc.)
const CHAIN_ID: u64 = 10;

/// Sender nonce
const FROM_NONCE: u64 = 19128;

/// Gas limit for the transaction
const GAS_LIMIT: u64 = 500_000;

/// Gas price in gwei
const GAS_PRICE_GWEI: u128 = 1;

/// Priority fee in gwei
const PRIORITY_FEE_GWEI: u128 = 1;

// ============================================================================
// MAIN EXECUTION - NO NEED TO EDIT BELOW THIS LINE
// ============================================================================

fn main() {
    println!("=== RPC-based REVM Transaction Tracer ===\n");

    // Parse addresses
    let from_address = Address::from_str(FROM_ADDRESS)
        .expect("Invalid FROM_ADDRESS");
    let to_address = Address::from_str(TO_ADDRESS)
        .expect("Invalid TO_ADDRESS");

    // Parse value
    let value = U256::from_str(VALUE_WEI)
        .expect("Invalid VALUE_WEI");

    // Parse calldata
    let calldata_hex = CALLDATA.strip_prefix("0x").unwrap_or(CALLDATA);
    let calldata = Bytes::from(hex::decode(calldata_hex).expect("Invalid CALLDATA hex"));

    println!("Configuration:");
    println!("  RPC URL: {}", RPC_URL);
    println!("  From: {:?}", from_address);
    println!("  To: {:?}", to_address);
    println!("  Value: {} wei", value);
    println!("  Calldata: {} bytes", calldata.len());
    println!("  Block: {}", BLOCK_NUMBER);
    println!("  Chain ID: {}", CHAIN_ID);
    println!("  Nonce: {}", FROM_NONCE);
    println!("  Gas Limit: {}", GAS_LIMIT);
    println!("  Gas Price: {} gwei", GAS_PRICE_GWEI);
    println!("  Priority Fee: {} gwei\n", PRIORITY_FEE_GWEI);

    println!("=== Fetching data from RPC... ===\n");

    // Fetch block details
    let block_details = match fetch_block_details(RPC_URL, BLOCK_NUMBER) {
        Ok(details) => {
            println!("✓ Block details fetched successfully");
            println!("  Block Number: {}", details.number);
            println!("  Timestamp: {}", details.timestamp);
            println!("  Base Fee: {} gwei", details.base_fee_per_gas / U256::from(1_000_000_000u64));
            details
        }
        Err(e) => {
            eprintln!("✗ Failed to fetch block details: {}", e);
            eprintln!("\nMake sure:");
            eprintln!("  1. RPC_URL is correct and accessible");
            eprintln!("  2. BLOCK_NUMBER exists");
            return;
        }
    };

    // Fetch prestate using debug_traceCall
    let prestate = match fetch_prestate(
        RPC_URL,
        &from_address,
        &to_address,
        &value,
        &calldata,
        BLOCK_NUMBER,
    ) {
        Ok(state) => {
            println!("✓ Prestate fetched successfully ({} accounts)\n", state.len());
            state
        }
        Err(e) => {
            eprintln!("✗ Failed to fetch prestate: {}", e);
            eprintln!("\nMake sure:");
            eprintln!("  1. RPC endpoint supports debug_traceCall");
            eprintln!("  2. All addresses and parameters are valid");
            eprintln!("  3. You have access to debug API (may require paid plan or local node)");
            return;
        }
    };

    println!("=== Executing transaction trace... ===\n");

    // Create block environment
    let block_env = BlockEnv {
        number: block_details.number,
        beneficiary: block_details.miner,
        timestamp: block_details.timestamp,
        gas_limit: block_details.gas_limit.try_into().unwrap_or(30_000_000u64),
        basefee: block_details.base_fee_per_gas.try_into().unwrap_or(20_000_000_000u64),
        difficulty: block_details.difficulty,
        prevrandao: Some(revm::primitives::B256::from(block_details.difficulty)),
        blob_excess_gas_and_price: Some(
            revm::context_interface::block::BlobExcessGasAndPrice::new(1, 1)
        ),
    };

    // Convert gas prices from gwei to wei
    let gas_price = GAS_PRICE_GWEI * 1_000_000_000;
    let gas_priority_fee = PRIORITY_FEE_GWEI * 1_000_000_000;

    // Execute the trace
    match trace_transaction(
        CHAIN_ID,
        from_address,
        FROM_NONCE,
        to_address,
        calldata,
        GAS_LIMIT,
        gas_price,
        gas_priority_fee,
        block_env,
        prestate,
    ) {
        Ok(result) => {
            println!("✓ Trace completed successfully!\n");
            println!("=== Trace Result (JSON) ===\n");
            let json = serde_json::to_string_pretty(&result).unwrap();
            println!("{}\n", json);

            println!("=== Summary ===");
            println!("Call Type: {}", result.calls.call_type);
            println!("From: {:?}", result.calls.from);
            println!("To: {:?}", result.calls.to);
            println!("Value: {} wei", result.calls.value);
            println!("Gas: {}", result.calls.gas);
            println!("Gas Used: {}", result.calls.gas_used);

            let success = result.calls.error.is_none();
            println!("Status: {}", if success { "✓ SUCCESS" } else { "✗ FAILED" });

            if let Some(error) = &result.calls.error {
                println!("\n❌ Error: {}", error);
            }
            if let Some(revert_reason) = &result.calls.revert_reason {
                println!("Revert Reason: {}", revert_reason);
            }

            if let Some(output) = &result.calls.output {
                if !output.is_empty() {
                    println!("\nOutput: 0x{}", hex::encode(output));
                }
            }

            println!("\nNumber of Subcalls: {}", result.calls.calls.len());

            if !result.calls.calls.is_empty() {
                println!("\n=== Subcalls ===");
                print_subcalls(&result.calls.calls, 1);
            }

            println!("\n✓ Trace analysis complete!");
        }
        Err(e) => {
            eprintln!("✗ Error tracing transaction: {}", e);
            eprintln!("\nThis could be due to:");
            eprintln!("  - Invalid transaction parameters");
            eprintln!("  - Insufficient gas limit");
            eprintln!("  - Invalid nonce");
            eprintln!("  - Precompile or system contract interaction");
        }
    }
}

fn print_subcalls(calls: &[revm_tracer::trace::inspector::CallFrame], depth: usize) {
    let indent = "  ".repeat(depth);
    for (i, call) in calls.iter().enumerate() {
        let status = if call.error.is_none() { "✓" } else { "✗" };
        println!("{}{} {}. {} from {:?} to {:?}",
            indent, status, i + 1, call.call_type, call.from, call.to);
        println!("{}   Gas Used: {}", indent, call.gas_used);

        if let Some(error) = &call.error {
            println!("{}   Error: {}", indent, error);
        }

        if !call.calls.is_empty() {
            print_subcalls(&call.calls, depth + 1);
        }
    }
}

// ============================================================================
// RPC HELPER FUNCTIONS
// ============================================================================

fn fetch_block_details(rpc_url: &str, block_number: &str) -> Result<BlockDetails, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "eth_getBlockByNumber",
        "params": [block_number, false],
        "id": 1
    });

    let response = client
        .post(rpc_url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let response_text = response.text()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let json_response: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse JSON: {}. Response: {}", e, response_text))?;

    if let Some(error) = json_response.get("error") {
        return Err(format!("RPC error: {}", error));
    }

    let result = json_response
        .get("result")
        .ok_or_else(|| format!("No result in response: {}", response_text))?;

    if result.is_null() {
        return Err(format!("Block not found: {}", block_number));
    }

    serde_json::from_value(result.clone())
        .map_err(|e| format!("Failed to deserialize block details: {}. Result: {}", e, result))
}

fn fetch_prestate(
    rpc_url: &str,
    from: &Address,
    to: &Address,
    value: &U256,
    data: &Bytes,
    block_number: &str,
) -> Result<HashMap<Address, AccountDetails>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Construct the transaction object
    let tx_object = json!({
        "from": format!("{:?}", from),
        "to": format!("{:?}", to),
        "value": format!("0x{:x}", value),
        "data": format!("0x{}", hex::encode(data)),
    });

    println!("Debug - Transaction object: {}", tx_object);

    // Request with prestateTracer
    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "debug_traceCall",
        "params": [
            tx_object,
            block_number,
            {
                "tracer": "prestateTracer",
                "stateOverrides": {
                "0x123c09e092faa42ac4b3322661345ff7956a350e": {
                    "stateDiff": {
                        "0x6b0c432833943740bc6e9ef81debbd2891bfc95a9bf62994fbd90cdae4b251f9": "0x0000000000000000000000000000000000000000000000000000000000000001",
                        "0xbe282059c802d59471d7d8082e29c73d62e56f511b3cafb30ef1cddf3567dbec": "0x0000000000000000000000000000000000000000000000000000000000000001"
                    }
                },
                "0x671376e3434c64b56fd943d5240ba6d1ae24e56a": {
                    "balance": "0x3635c9adc5dea00000"
                }
            }
            }
        ],
        "id": 1
    });

    let response = client
        .post(rpc_url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let response_text = response.text()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let json_response: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse JSON: {}. Response: {}", e, response_text))?;

    if let Some(error) = json_response.get("error") {
        return Err(format!("RPC error: {}. Make sure debug_traceCall is supported.", error));
    }

    let result = json_response
        .get("result")
        .ok_or_else(|| format!("No result in response: {}", response_text))?;

    // The prestate tracer returns the accounts directly
    serde_json::from_value(result.clone())
        .map_err(|e| format!("Failed to deserialize prestate: {}. Response: {}", e, result))
}
