use anyhow::Result;
use std::{cmp::Ordering, ops::Shl};
use tracing::info;

use num_bigint::{BigInt, ToBigInt};

use crate::block::Block;

const TARGET_BITS: usize = 10;

pub struct ProofOfWork {
    block: Block,
    target: BigInt,
}

impl ProofOfWork {
    pub fn new_proof_of_work(block: Block) -> Self {
        let mut target: BigInt = 1.to_bigint().unwrap();
        target = target.shl(256 - TARGET_BITS);

        Self {
            block: block,
            target,
        }
    }

    pub fn prepare_data(&self, nonce: u128) -> String {
        let data = format!(
            "{}:{}:{}:{}:{}",
            self.block.get_prehash(),
            self.block.get_data(),
            self.block.get_timestamp(),
            TARGET_BITS,
            nonce
        );
        data
    }

    pub fn run(&self) -> Result<(u128, String)> {
        let mut nonce = 0;
        let mut hash: String = "".into();
        println!("Mining the block containing {}", self.block.get_data());
        while nonce < u128::MAX {
            hash = self.prepare_data(nonce);
            hash = sha256::digest(hash);

            let big_hash = BigInt::parse_bytes(hash.as_bytes(), 16).unwrap();
            match big_hash.cmp(&self.target) {
                Ordering::Equal | Ordering::Less => {
                    println!("{hash}");
                    break;
                }
                _ => {
                    nonce += 1;
                }
            }
        }
        println!("");
        Ok((nonce, hash))
    }

    pub fn validate(&self) -> bool {
        let data = self.prepare_data(self.block.get_nonce());
        let hash = sha256::digest(data);
        let hash_big = BigInt::parse_bytes(hash.as_bytes(), 16).unwrap();

        if Ordering::Greater != hash_big.cmp(&self.target) {
            true
        } else {
            false
        }
    }
}
