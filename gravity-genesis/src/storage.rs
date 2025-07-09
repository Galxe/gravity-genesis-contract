use std::collections::HashMap;

use revm::{
    db::PlainAccount,
    primitives::{
        alloy_primitives::U160, keccak256, ruint::UintTryFrom, AccountInfo, Address, Bytecode,
        B256, I256, U256,
    },
    DatabaseRef,
};


/// A DatabaseRef that stores chain data in memory.
#[derive(Debug, Default, Clone)]
pub struct InMemoryDB {
    pub accounts: HashMap<Address, PlainAccount>,
    pub bytecodes: HashMap<B256, Bytecode>,
    pub block_hashes: HashMap<u64, B256>,
    /// Simulated query latency in microseconds
    pub latency_us: u64,
}

impl InMemoryDB {
    pub fn new(
        accounts: HashMap<Address, PlainAccount>,
        bytecodes: HashMap<B256, Bytecode>,
        block_hashes: HashMap<u64, B256>,
    ) -> Self {
        Self { accounts, bytecodes, block_hashes, latency_us: 0 }
    }
}

impl DatabaseRef for InMemoryDB {
    type Error = String;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        if self.latency_us > 0 {
            std::thread::sleep(std::time::Duration::from_micros(self.latency_us));
        }
        Ok(self.accounts.get(&address).map(|account| account.info.clone()))
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        if self.latency_us > 0 {
            std::thread::sleep(std::time::Duration::from_micros(self.latency_us));
        }
        self.bytecodes
            .get(&code_hash)
            .cloned()
            .ok_or(String::from(format!("can't find code by hash {code_hash}")))
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        if self.latency_us > 0 {
            std::thread::sleep(std::time::Duration::from_micros(self.latency_us));
        }
        let storage = self.accounts.get(&address).ok_or(format!("can't find account {address}"))?;
        Ok(storage.storage.get(&index).cloned().unwrap_or_default())
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        if self.latency_us > 0 {
            std::thread::sleep(std::time::Duration::from_micros(self.latency_us));
        }
        Ok(self
            .block_hashes
            .get(&number)
            .cloned()
            // Matching REVM's [EmptyDB] for now
            .unwrap_or_else(|| keccak256(number.to_string().as_bytes())))
    }
}
