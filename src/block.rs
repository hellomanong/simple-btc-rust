use anyhow::Result;
use std::time::{self, UNIX_EPOCH};

use sha256::digest;

#[derive(Debug)]
pub struct Block {
    timestamp: u128,         //当前时间戳，也就是区块创建的时间
    data: String,            //区块存储的实际有效信息，也就是交易
    prev_block_hash: String, //前一个块的哈希，即父哈希
    hash: String,            //当前块的哈希 (pre_block_hash+timestamp+data)
}

impl Block {
    pub fn new_block(prev_block_hash: String, data: String) -> Result<Self> {
        let now = time::SystemTime::now().duration_since(UNIX_EPOCH)?;
        let mut block = Self {
            timestamp: now.as_millis(),
            data: data,
            prev_block_hash: prev_block_hash,
            hash: "".into(),
        };

        block.set_hash();
        Ok(block)
    }

    pub fn set_hash(&mut self) {
        let tmp_str = format!("{}:{}:{}", self.prev_block_hash, self.timestamp, self.data);
        let hash = digest(tmp_str);
        self.hash = hash;
    }

    pub fn get_hash(&self) -> &String {
        &self.hash
    }

    pub fn get_data(&self) -> &String {
        &self.data
    }

    pub fn get_timestamp(&self) -> u128 {
        self.timestamp
    }

    pub fn get_prehash(&self) -> &String {
        &self.prev_block_hash
    }
}
