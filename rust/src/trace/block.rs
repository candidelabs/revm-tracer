use serde::Deserialize;
use revm::primitives::{Address, U256, B256};
use revm::{
    context::BlockEnv,
    context_interface::block::BlobExcessGasAndPrice,
    primitives::ruint::FromUintError
};

/// Block details from eth_getBlockByNumber RPC call
#[derive(Debug, Deserialize)]
pub struct BlockDetails {
    pub number: U256,
    pub miner: Address,
    pub timestamp: U256,
    #[serde(rename(deserialize = "gasLimit"))]
    pub gas_limit: U256,
    #[serde(rename(deserialize = "baseFeePerGas"))]
    pub base_fee_per_gas: U256,
    pub difficulty: U256,
    #[serde(rename(deserialize = "excessBlobGas"))]
    pub excess_blob_gas: U256,
}

pub fn create_block_env_from_block_details(
    block_details: BlockDetails
)->Result<BlockEnv, FromUintError<u64>> {
    Ok(BlockEnv {
        number: block_details.number,
        beneficiary: block_details.miner,
        timestamp: block_details.timestamp,
        gas_limit: block_details.gas_limit.try_into()?,
        basefee: block_details.base_fee_per_gas.try_into()?,
        difficulty: block_details.difficulty,
        prevrandao: Some(B256::from(block_details.difficulty)),
        blob_excess_gas_and_price: Some(
            BlobExcessGasAndPrice::new(
                1
                /*std::cmp::max( //zero excess blob gas causes revm to error
                    block_details.excess_blob_gas.try_into()?,
                    1
                )*/,
                1 // blob gas price
            )
        )
    })
}
