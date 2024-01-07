use std::str::from_utf8;

use crate::block::Block;
use anyhow::{anyhow, Result};
use sled::{
    transaction::{ConflictableTransactionError, TransactionError, Transactional},
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
const BLOCKS_LAST: &str = "blocks_last";
const LAST: &str = "last";
impl Blockchain {
    pub fn new_block_chain() -> Result<Self> {
        let db = sled::open(DB_FILE).unwrap();

        let bucket = db.open_tree(BLOCKS).unwrap();
        let bucket_last = db.open_tree(BLOCKS_LAST).unwrap();

        let genesis = new_genesis_block()?;
        let data = bucket_last.get(LAST)?;

        let tip = match data {
            Some(iv) => from_utf8(iv.as_ref())?.into(),
            None => {
                let genesis_json = genesis.serialize()?;
                bucket.insert(genesis.get_hash().as_str(), genesis_json.as_str())?;
                bucket_last.insert(LAST, genesis_json.as_str())?;
                genesis.get_hash()
            }
        };

        let block_chain = Self { tip, db };
        Ok(block_chain)
    }

    pub fn add_block(&mut self, data: String) -> Result<()> {
        let db = self.db.open_tree(BLOCKS)?;
        let db_last = self.db.open_tree(BLOCKS_LAST)?;

        let res: Result<(), TransactionError<anyhow::Error>> = [db, db_last].transaction(|tx_db| {
            let block_db = &tx_db[0];
            let last_db = &tx_db[1];
            match last_db.get(LAST)? {
                Some(iv) => match from_utf8(iv.as_ref()) {
                    Ok(v) => {
                        let last_hash = v.into();
                        let block = Block::new_block(last_hash, data.clone())
                            .map_err(|e| ConflictableTransactionError::Abort(anyhow!(e)))?;

                        block_db.insert(
                            block.get_hash().as_str(),
                            block
                                .serialize()
                                .map_err(|e| ConflictableTransactionError::Abort(anyhow!(e)))?
                                .as_str(),
                        )?;

                        last_db.insert(LAST, block.get_hash().as_str())?;
                        Ok(())
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
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub fn iter_blocks(&self) -> Result<impl Iterator<Item = Block>> {
        let db = self.db.open_tree(BLOCKS)?;
        let data = db.iter();
        Ok(StorageIter::new(data))
    }
}

pub fn new_genesis_block() -> Result<Block> {
    Block::new_block("".into(), "Genesis Block".into())
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

        info!("-----data:{data}-----");
        Block::deserialize(data.as_str())
    }
}
