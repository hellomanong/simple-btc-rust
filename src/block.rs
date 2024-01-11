use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::time::{self, UNIX_EPOCH};
use tracing::error;

use crate::{proof_of_work::ProofOfWork, transaction::Transaction};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Block {
    pub timestamp: u128,                //当前时间戳，也就是区块创建的时间
    pub transactions: Vec<Transaction>, //区块存储的实际有效信息，也就是交易
    pub prev_block_hash: String,        //前一个块的哈希，即父哈希
    pub hash: String,                   //当前块的哈希 (pre_block_hash+timestamp+data)
    pub nonce: u128,
}

impl Block {
    pub fn new_block(prev_block_hash: String, transactions: Vec<Transaction>) -> Result<Self> {
        let now = time::SystemTime::now().duration_since(UNIX_EPOCH)?;
        let mut block = Self {
            timestamp: now.as_millis(),
            transactions: transactions,
            prev_block_hash: prev_block_hash,
            ..Default::default()
        };

        let proof_of_work = ProofOfWork::new_proof_of_work(block.clone());
        let (nonce, hash) = proof_of_work.run()?;
        block.hash = hash;
        block.nonce = nonce;
        Ok(block)
    }

    pub fn serialize(&self) -> Result<String> {
        let data = serde_json::to_string(self).map_err(|e| {
            error!("Serialize block err: {e}");
            e
        })?;

        Ok(data)
    }

    pub fn deserialize(data: &str) -> Result<Block> {
        let data = serde_json::from_str(data).map_err(|e| {
            error!("Deserialize block err: {e}");
            e
        })?;

        Ok(data)
    }

    pub fn hash_transactions(&self) -> String {
        let mut tx_hashes = vec![];
        for tx in &self.transactions {
            tx_hashes.extend(tx.id.as_bytes());
        }

        let hash = sha256::digest(tx_hashes);
        hash
    }

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn get_timestamp(&self) -> u128 {
        self.timestamp
    }

    pub fn get_prehash(&self) -> String {
        self.prev_block_hash.clone()
    }

    pub fn get_nonce(&self) -> u128 {
        self.nonce.clone()
    }
}
