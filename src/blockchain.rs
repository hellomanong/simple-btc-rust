use std::{
    collections::{hash_map::Entry, HashMap},
    fs,
    process::{self, Output},
    str::from_utf8,
};

use crate::{
    block::Block,
    transaction::{Transaction, TxOutput},
};
use anyhow::{anyhow, Error, Result};
use sled::{
    transaction::{ConflictableTransactionError, TransactionError},
    IVec,
};
use tracing::error;

const GENESISCOINBASEDATA: &str = "GenesisCoinBaseData";

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub tip: String,
    db: sled::Db,
}

const DB_FILE: &str = "btc_data";
const BLOCKS: &str = "blocks";
const LAST: &str = "last";

impl Blockchain {
    pub fn new_block_chain() -> Result<Self> {
        if db_exists() == false {
            error!("No existing blockchian found, Create one first");
            return Err(anyhow!("No existing blockchian found, Create one first"));
        }

        let db = sled::open(DB_FILE)?;
        let bucket = db.open_tree(BLOCKS)?;
        let tip = match bucket.get(LAST)? {
            Some(iv) => from_utf8(iv.as_ref())?.into(),
            None => return Err(anyhow!("Get last info, return None")),
        };

        let block_chain = Self { tip, db };
        Ok(block_chain)
    }

    pub fn create_block_chain(address: String) -> Result<Self> {
        if db_exists() {
            error!("Blockchian already exist");
            return Err(anyhow!("Blockchian already exist"));
        }
        let db = sled::open(DB_FILE).unwrap();

        let bucket = db.open_tree(BLOCKS).unwrap();

        let tx = Transaction::new_coin_base_tx(address, GENESISCOINBASEDATA.into())?;
        let genesis = new_genesis_block(tx)?;

        let genesis_json = genesis.serialize()?;
        bucket.insert(genesis.get_hash().as_str(), genesis_json.as_str())?;
        bucket.insert(LAST, genesis.get_hash().as_str())?;

        let block_chain = Self {
            tip: genesis.get_hash(),
            db,
        };
        Ok(block_chain)
    }
}

impl Blockchain {
    pub fn mine_block(&mut self, txes: Vec<Transaction>) -> Result<Block> {
        for tx in txes.iter() {
            if !self.verify_transaction(tx)? {
                return Err(anyhow!("Verity tx failed"));
            }
        }

        let db = self.db.open_tree(BLOCKS)?;

        let res: Result<Block, TransactionError<anyhow::Error>> =
            db.transaction(|tx_db: &sled::transaction::TransactionalTree| {
                match tx_db.get(LAST)? {
                    Some(iv) => match from_utf8(iv.as_ref()) {
                        Ok(_) => {
                            let block = Block::new_block(self.tip.clone(), txes.clone())
                                .map_err(|e| ConflictableTransactionError::Abort(anyhow!(e)))?;

                            tx_db.insert(
                                block.get_hash().as_str(),
                                block
                                    .serialize()
                                    .map_err(|e| ConflictableTransactionError::Abort(anyhow!(e)))?
                                    .as_str(),
                            )?;

                            tx_db.insert(LAST, block.get_hash().as_str())?;
                            Ok(block)
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
                self.tip = v.get_hash();
                Ok(v)
            }
            Err(e) => Err(anyhow!(e)),
        }
    }

    // 暂时从所有的区块中获取未花费的交易
    pub fn find_utxo(&self) -> Result<HashMap<String, Vec<TxOutput>>> {
        let mut utxo: HashMap<String, Vec<TxOutput>> = HashMap::new();
        // vec中存储的是一笔交易中的已经花费的输出的索引
        // 使用vec 是因为，这笔交易中包含，不确定几个人的交易输出
        let mut spent_txos: HashMap<String, Vec<isize>> = HashMap::new();

        let mut bci = self.iterator();
        loop {
            let block = bci.next()?;
            for tx in block.transactions {
                'Output:
                // 先遍历交易中的所有输出
                for (index, out) in tx.vout.iter().enumerate() {
                    // 已花费的map中，包含此交易
                    if spent_txos.contains_key(tx.id.as_str()) {
                        for spent_out in &spent_txos[tx.id.as_str()] {
                            // 就判断是否包含本条输出,如果包含，说明已经花费了，继续下一条判断 
                            if index as isize == *spent_out {
                                continue 'Output;
                            }
                        }
                    }   

                let txs =  utxo.entry(tx.id.clone()).or_insert(vec![]);
                txs.push(out.clone());

                }

                if !tx.is_coinbase() {
                    for i in tx.vin {
                        // if i.uses_key(pubkey_hash) {
                            // println!("------------i.txid:{:?}------------", i.txid);
                            let vec_txos: &mut Vec<isize> = spent_txos.entry(i.txid).or_insert(vec![]);
                            vec_txos.push(i.vout);
                        // }
                    }
                }
            }

            if block.prev_block_hash.len() == 0 {
                break;
            }
        }

        Ok(utxo)
    }

    

    pub fn find_transaction(&self, id: &str) -> Result<Transaction> {

        let mut bci = self.iterator();
        loop {
            let block: Block = bci.next()?;
            for tx in block.transactions {
                if tx.id.eq(id) {
                    return Ok(tx);
                }
            }

            if block.prev_block_hash.is_empty() {
                return Err(anyhow!("Do not cantains this tx"));
            }
        }

    }

    pub fn sign_transaction(&self, tx: &mut Transaction, privkey:  &[u8]) -> Result<()> {
        let prev_txs: Result<HashMap<String, Transaction>, _> = tx.vin.iter().map(|v| -> Result<(String, Transaction), anyhow::Error> {
            let prev_tx = self.find_transaction(&v.txid)?;
            Ok((prev_tx.id.clone(), prev_tx))
        }).collect();


        tx.sign(privkey, prev_txs?)
    }

    pub fn verify_transaction(&self, tx: &Transaction) -> Result<bool> {
        let prev_txs: Result<HashMap<String, Transaction>, _> = tx.vin.iter().map(|v| -> Result<(String, Transaction), anyhow::Error> {
            let prev_tx = self.find_transaction(&v.txid)?;
            Ok((prev_tx.id.clone(), prev_tx))
        }).collect();

        tx.verify(prev_txs?)
    }


    pub fn iterator(&self) -> BlockChainIter {
        BlockChainIter {
            hash: self.tip.clone(),
            db: self.db.clone(),
        }
    }

    pub fn get_db(&self) -> sled::Db{
        self.db.clone()
    }


}

pub fn new_genesis_block(coinbase: Transaction) -> Result<Block> {
    Block::new_block("".into(), vec![coinbase])
}

pub fn db_exists() -> bool {
    fs::metadata(DB_FILE).is_ok()
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
