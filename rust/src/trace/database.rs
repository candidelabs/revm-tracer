use serde::Deserialize;
use revm::database::InMemoryDB;
use revm::state::{AccountInfo, Bytecode};
use revm::primitives::{Address, StorageKey, StorageValue, Bytes, HashMap, U256};

/// Account state details from prestate tracer
#[derive(Debug, Deserialize)]
pub struct AccountDetails {
    pub balance: Option<U256>,
    pub nonce: Option<u64>,
    pub code: Option<Bytes>,
    pub storage: Option<HashMap<StorageKey, StorageValue>>,
}

pub fn create_in_memory_database_from_prestate_trace(
    prestate_tracer_result: HashMap<Address, AccountDetails>
)->InMemoryDB { 
    let mut database = InMemoryDB::default();
    for account_result in prestate_tracer_result.into_iter() {
        let account_address = account_result.0;
        match account_result.1.storage {
            Some(storage) => {
                for storage_result in storage.into_iter() {
                    _ = database.insert_account_storage(
                            account_address, storage_result.0, storage_result.1
                    ).unwrap();
                };
            }
            None =>{ }
        }

        let balance: U256 = account_result.1.balance.unwrap_or(U256::ZERO);
        let nonce: u64 = account_result.1.nonce.unwrap_or(0);
        let code_hash;
        let code: Option<Bytecode>;
        match account_result.1.code {
            Some(code_res) => {
                code_hash = revm::primitives::keccak256(&code_res);
                code = Some(Bytecode::new_raw(code_res));
            }
            None =>{ 
                code_hash = revm::primitives::KECCAK_EMPTY;
                code = None;
            }
        };
        database.insert_account_info(
            account_address,
            AccountInfo {
                balance,
                nonce,
                code_hash,
                code,
            }
        );
    };
    database
}
