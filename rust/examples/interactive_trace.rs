use revm::primitives::{Address, Bytes, U256, HashMap};
use revm::context::BlockEnv;
use std::str::FromStr;
use std::io::{self, Write};
use serde_json::json;

// Import from the library
use revm_tracer::trace::{
    database::AccountDetails,
    trace::trace_transaction,
    block::BlockDetails,
};

fn main() {
    println!("=== Interactive REVM Transaction Tracer ===\n");
    println!("This tool will trace a transaction by fetching data from an RPC node.\n");

    // Get RPC URL
    let rpc_url = prompt_input("Enter RPC URL (e.g., https://eth.llamarpc.com): ");

    // Get transaction parameters
    let from_address = prompt_address("Enter FROM address: ");
    let to_address = prompt_address("Enter TO address: ");
    let value = prompt_u256("Enter value in wei (0 for no value transfer): ");
    let calldata = prompt_hex("Enter calldata (e.g., 0x for empty, 0xa9059cbb... for data): ");
    let block_number = prompt_input("Enter block number (or 'latest'): ");

    println!("\n=== Fetching data from RPC... ===\n");

    // Fetch block details
    let block_details = match fetch_block_details(&rpc_url, &block_number) {
        Ok(details) => {
            println!("✓ Block details fetched successfully");
            println!("  Block Number: {}", details.number);
            println!("  Timestamp: {}", details.timestamp);
            println!("  Base Fee: {} gwei", details.base_fee_per_gas / U256::from(1_000_000_000u64));
            details
        }
        Err(e) => {
            eprintln!("✗ Failed to fetch block details: {}", e);
            return;
        }
    };

    // Fetch prestate using debug_traceCall
    let prestate = match fetch_prestate(
        &rpc_url,
        &from_address,
        &to_address,
        &value,
        &calldata,
        &block_number,
    ) {
        Ok(state) => {
            println!("✓ Prestate fetched successfully ({} accounts)", state.len());
            state
        }
        Err(e) => {
            eprintln!("✗ Failed to fetch prestate: {}", e);
            eprintln!("\nNote: Make sure your RPC endpoint supports debug_traceCall");
            return;
        }
    };

    // Get additional transaction parameters
    let chain_id = prompt_u64("\nEnter chain ID (1 for mainnet, 11155111 for sepolia): ");
    let from_nonce = prompt_u64("Enter sender nonce: ");
    let gas_limit = prompt_u64("Enter gas limit: ");
    let gas_price = prompt_u128("Enter gas price in gwei: ") * 1_000_000_000;
    let gas_priority_fee = prompt_u128("Enter priority fee in gwei: ") * 1_000_000_000;

    println!("\n=== Executing transaction trace... ===\n");

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

    // Execute the trace
    match trace_transaction(
        chain_id,
        from_address,
        from_nonce,
        to_address,
        calldata,
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
            println!("Value: {} wei", result.calls.value);
            println!("Gas: {}", result.calls.gas);
            println!("Gas Used: {}", result.calls.gas_used);
            println!("Success: {}", result.calls.error.is_none());

            if let Some(error) = &result.calls.error {
                println!("Error: {}", error);
            }
            if let Some(revert_reason) = &result.calls.revert_reason {
                println!("Revert Reason: {}", revert_reason);
            }

            println!("Number of Subcalls: {}", result.calls.calls.len());

            if !result.calls.calls.is_empty() {
                println!("\nSubcalls:");
                print_subcalls(&result.calls.calls, 1);
            }
        }
        Err(e) => {
            eprintln!("✗ Error tracing transaction: {}", e);
        }
    }
}

fn print_subcalls(calls: &[revm_tracer::trace::inspector::CallFrame], depth: usize) {
    let indent = "  ".repeat(depth);
    for (i, call) in calls.iter().enumerate() {
        println!("{}{}. {} from {:?} to {:?}", indent, i + 1, call.call_type, call.from, call.to);
        println!("{}   Gas Used: {}", indent, call.gas_used);
        if call.error.is_some() {
            println!("{}   Status: FAILED", indent);
        } else {
            println!("{}   Status: SUCCESS", indent);
        }
        if !call.calls.is_empty() {
            print_subcalls(&call.calls, depth + 1);
        }
    }
}

fn prompt_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn prompt_address(prompt: &str) -> Address {
    loop {
        let input = prompt_input(prompt);
        match Address::from_str(&input) {
            Ok(addr) => return addr,
            Err(_) => println!("Invalid address format. Please try again."),
        }
    }
}

fn prompt_u64(prompt: &str) -> u64 {
    loop {
        let input = prompt_input(prompt);
        match input.parse::<u64>() {
            Ok(val) => return val,
            Err(_) => println!("Invalid number. Please try again."),
        }
    }
}

fn prompt_u128(prompt: &str) -> u128 {
    loop {
        let input = prompt_input(prompt);
        match input.parse::<u128>() {
            Ok(val) => return val,
            Err(_) => println!("Invalid number. Please try again."),
        }
    }
}

fn prompt_u256(prompt: &str) -> U256 {
    loop {
        let input = prompt_input(prompt);
        match U256::from_str(&input) {
            Ok(val) => return val,
            Err(_) => println!("Invalid U256. Please try again."),
        }
    }
}

fn prompt_hex(prompt: &str) -> Bytes {
    loop {
        let input = prompt_input(prompt);
        let hex_str = input.strip_prefix("0x").unwrap_or(&input);
        match hex::decode(hex_str) {
            Ok(bytes) => return Bytes::from(bytes),
            Err(_) => println!("Invalid hex string. Please try again."),
        }
    }
}

fn fetch_block_details(rpc_url: &str, block_number: &str) -> Result<BlockDetails, String> {
    let client = reqwest::blocking::Client::new();

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "eth_getBlockByNumber",
        "params": [block_number, false],
        "id": 1
    });

    let response = client
        .post(rpc_url)
        .json(&request_body)
        .send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let response_text = response.text().map_err(|e| format!("Failed to read response: {}", e))?;

    let json_response: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if let Some(error) = json_response.get("error") {
        return Err(format!("RPC error: {}", error));
    }

    let result = json_response
        .get("result")
        .ok_or("No result in response")?;

    serde_json::from_value(result.clone())
        .map_err(|e| format!("Failed to deserialize block details: {}", e))
}

fn fetch_prestate(
    rpc_url: &str,
    from: &Address,
    to: &Address,
    value: &U256,
    data: &Bytes,
    block_number: &str,
) -> Result<HashMap<Address, AccountDetails>, String> {
    let client = reqwest::blocking::Client::new();

    // Construct the transaction object
    let tx_object = json!({
        "from": format!("{:?}", from),
        "to": format!("{:?}", to),
        "value": format!("0x{:x}", value),
        "data": format!("0x{}", hex::encode(data)),
    });

    println!("{tx_object}");

    // Request with prestateTracer
    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "debug_traceCall",
        "params": [
            tx_object,
            block_number,
            {
                "tracer": "prestateTracer"
            }
        ],
        "id": 1
    });

    let response = client
        .post(rpc_url)
        .json(&request_body)
        .send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let response_text = response.text().map_err(|e| format!("Failed to read response: {}", e))?;

    let json_response: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if let Some(error) = json_response.get("error") {
        return Err(format!("RPC error: {}", error));
    }

    let result = json_response
        .get("result")
        .ok_or("No result in response")?;

    // The prestate tracer returns the accounts directly
    serde_json::from_value(result.clone())
        .map_err(|e| format!("Failed to deserialize prestate: {}. Response: {}", e, result))
}
