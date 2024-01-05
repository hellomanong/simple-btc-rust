use anyhow::{Ok, Result};
use std::time::{self, UNIX_EPOCH};

use crate::proof_of_work::ProofOfWork;

#[derive(Debug, Clone, Default)]
pub struct Block {
    timestamp: u128,         //当前时间戳，也就是区块创建的时间
    data: String,            //区块存储的实际有效信息，也就是交易
    prev_block_hash: String, //前一个块的哈希，即父哈希
    hash: String,            //当前块的哈希 (pre_block_hash+timestamp+data)
    nonce: u128,
}

impl Block {
    pub fn new_block(prev_block_hash: String, data: String) -> Result<Self> {
        let now = time::SystemTime::now().duration_since(UNIX_EPOCH)?;
        let mut block = Self {
            timestamp: now.as_millis(),
            data: data,
            prev_block_hash: prev_block_hash,
            ..Default::default()
        };

        let proof_of_work = ProofOfWork::new_proof_of_work(block.clone());
        let (nonce, hash) = proof_of_work.run()?;
        block.hash = hash;
        block.nonce = nonce;
        Ok(block)
    }

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn get_data(&self) -> String {
        self.data.clone()
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
