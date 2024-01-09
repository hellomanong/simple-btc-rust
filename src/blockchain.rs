use std::str::from_utf8;

use crate::block::Block;
use anyhow::{anyhow, Result};
use sled::{
    transaction::{ConflictableTransactionError, TransactionError},
    IVec,
};
use tracing::info;

#[derive(Debug)]
pub struct Blockchain {
    tip: String,
    db: sled::Db,
}

const DB_FILE: &str = "btc_data";
const BLOCKS: &str = "blocks";
const LAST: &str = "last";
impl Blockchain {
    pub fn new_block_chain() -> Result<Self> {
        let db = sled::open(DB_FILE).unwrap();

        let bucket = db.open_tree(BLOCKS).unwrap();
        let genesis = new_genesis_block()?;

        let tip = match bucket.get(LAST)? {
            Some(iv) => from_utf8(iv.as_ref())?.into(),
            None => {
                let genesis_json = genesis.serialize()?;
                bucket.insert(genesis.get_hash().as_str(), genesis_json.as_str())?;
                bucket.insert(LAST, genesis_json.as_str())?;
                genesis.get_hash()
            }
        };

        let block_chain = Self { tip, db };
        Ok(block_chain)
    }

    pub fn add_block(&mut self, data: String) -> Result<()> {
        let db = self.db.open_tree(BLOCKS)?;

        let res: Result<String, TransactionError<anyhow::Error>> =
            db.transaction(|tx_db: &sled::transaction::TransactionalTree| {
                match tx_db.get(LAST)? {
                    Some(iv) => match from_utf8(iv.as_ref()) {
                        Ok(_) => {
                            let block = Block::new_block(self.tip.clone(), data.clone())
                                .map_err(|e| ConflictableTransactionError::Abort(anyhow!(e)))?;

                            tx_db.insert(
                                block.get_hash().as_str(),
                                block
                                    .serialize()
                                    .map_err(|e| ConflictableTransactionError::Abort(anyhow!(e)))?
                                    .as_str(),
                            )?;

                            tx_db.insert(LAST, block.get_hash().as_str())?;
                            Ok(block.get_hash())
                        }
                        Err(e) => Err(ConflictableTransactionError::Abort(anyhow!(e))),
                    },
                    None => Err(ConflictableTransactionError::Abort(anyhow!(
                        "Get key=={}, return None",
                        LAST
                    ))),
                }
            });

        match res {
            Ok(v) => {
                self.tip = v;
                Ok(())
            }
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub fn iterator(&self) -> BlockChainIter {
        BlockChainIter {
            hash: self.tip.clone(),
            db: self.db.clone(),
        }
    }
}

pub fn new_genesis_block() -> Result<Block> {
    Block::new_block("".into(), "Genesis Block".into())
}

pub struct BlockChainIter {
    hash: String,
    db: sled::Db,
}

impl BlockChainIter {
    pub fn next(&mut self) -> Result<Block> {
        let db = self.db.open_tree(BLOCKS)?;
        match db.get(self.hash.clone())? {
            Some(iv) => {
                let data = from_utf8(iv.as_ref())?;
                let bc = Block::deserialize(data)?;
                self.hash = bc.get_prehash();
                Ok(bc)
            }

            None => Err(anyhow!("Get block, return None")),
        }
    }
}

pub struct StorageIter<T> {
    data: T,
}

impl<T> StorageIter<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T> Iterator for StorageIter<T>
where
    T: DoubleEndedIterator,
    T::Item: TryInto<Block>,
{
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        self.data.next_back().and_then(|v| match v.try_into() {
            Ok(block) => Some(block),
            Err(_) => None,
        })
    }
}

impl TryFrom<std::prelude::v1::Result<(IVec, IVec), sled::Error>> for Block {
    type Error = anyhow::Error;

    fn try_from(
        value: std::prelude::v1::Result<(IVec, IVec), sled::Error>,
    ) -> std::prelude::v1::Result<Self, Self::Error> {
        let data: String = match value {
            Ok((_, v2)) => from_utf8(v2.as_ref())?.into(),
            Err(e) => return Err(anyhow!(e)),
        };

        Block::deserialize(data.as_str())
    }
}
