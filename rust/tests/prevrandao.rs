use revm::primitives::B256;
use revm_tracer::trace::block::{create_block_env_from_block_details, BlockDetails};

fn block(difficulty: &str, mix_hash: Option<&str>) -> BlockDetails {
    let mut json = serde_json::json!({
        "number": "0x64",
        "miner": "0x4444444444444444444444444444444444444444",
        "timestamp": "0x6553f100",
        "gasLimit": "0x1c9c380",
        "baseFeePerGas": "0x3b9aca00",
        "difficulty": difficulty,
        "excessBlobGas": "0x0",
    });
    if let Some(mix_hash) = mix_hash {
        json["mixHash"] = serde_json::Value::String(mix_hash.to_string());
    }
    serde_json::from_value(json).unwrap()
}

const MIX: &str = "0x3333333333333333333333333333333333333333333333333333333333333333";

#[test]
fn post_merge_uses_mix_hash() {
    let env = create_block_env_from_block_details(block("0x0", Some(MIX))).unwrap();
    assert_eq!(env.prevrandao, Some(MIX.parse::<B256>().unwrap()));
}

#[test]
fn nonzero_difficulty_keeps_difficulty() {
    // Arbitrum (difficulty 1), BSC and Polygon return the difficulty for PREVRANDAO
    let env = create_block_env_from_block_details(block("0x1", Some(MIX))).unwrap();
    assert_eq!(env.prevrandao, Some(B256::from(revm::primitives::U256::from(1))));
}

#[test]
fn missing_mix_hash_falls_back_to_difficulty() {
    let env = create_block_env_from_block_details(block("0x0", None)).unwrap();
    assert_eq!(env.prevrandao, Some(B256::ZERO));
}
