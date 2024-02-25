use std::collections::HashMap;
use std::str::from_utf8;

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::transaction::TxOutput;
use anyhow::anyhow;
use anyhow::Result;
use serde::Serialize;
use sled::{transaction::ConflictableTransactionError, Transactional};

const UTXO_BUCKET: &str = "chainstate";
pub struct UTXOSet {
    bc: Blockchain,
}

impl UTXOSet {
    pub fn new(bc: Blockchain) -> Self {
        Self { bc }
    }

    pub fn reindex(&self) -> Result<()> {
        let db: sled::Db = self.bc.get_db();
        db.drop_tree(UTXO_BUCKET)?;

        let utxo = self.bc.find_utxo()?;

        let bucket = db.open_tree(UTXO_BUCKET)?;
        let r = bucket.transaction(|tx_db| {
            for (tx_id, outs) in utxo.iter() {
                let value = serde_json::to_string(&outs)
                    .map_err(|e| ConflictableTransactionError::Abort(anyhow!(e)))?;
                tx_db.insert(tx_id.as_bytes(), value.as_bytes())?;
            }

            Ok(())
        });

        match r {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub fn find_spentable_outputs(
        &self,
        pubkey_hash: &str,
        amount: isize,
        // isize：余额， map：<String：address， Vec：index of txoutput>
    ) -> Result<(isize, HashMap<String, Vec<isize>>)> {
        let mut unspent_outputs = HashMap::<String, Vec<isize>>::new();
        let mut accumulated = 0;

        let bucket = self.bc.get_db().open_tree(UTXO_BUCKET)?;
        for r in bucket.iter() {
            match r {
                Ok((key, value)) => {
                    let tx_id = from_utf8(key.as_ref())?;
                    let outs: Vec<TxOutput> = serde_json::from_slice(value.as_ref())?;

                    for (index, out) in outs.iter().enumerate() {
                        if out.is_locked_with_key(pubkey_hash) && accumulated < amount {
                            accumulated += out.value;
                            let out_idx = unspent_outputs.entry(tx_id.into()).or_insert(vec![]);
                            out_idx.push(index as isize);
                        }
                    }
                }
                Err(e) => return Err(anyhow!(e)),
            }
        }

        Ok((accumulated, unspent_outputs))
    }

    pub fn find_utxo(&self, pubkey_hash: &str) -> Result<Vec<TxOutput>> {
        let mut outputs = Vec::new();

        let bucket = self.bc.get_db().open_tree(UTXO_BUCKET)?;
        for r in bucket.iter() {
            match r {
                Ok((_, value)) => {
                    let outs: Vec<TxOutput> = serde_json::from_slice(value.as_ref())?;

                    for out in outs.into_iter() {
                        if out.is_locked_with_key(pubkey_hash) {
                            outputs.push(out);
                        }
                    }
                }
                Err(e) => return Err(anyhow!(e)),
            }
        }

        Ok(outputs)
    }

    pub fn update(&self, block: Block) -> Result<()> {
        let db = self.bc.get_db();
        let bucket = db.open_tree(UTXO_BUCKET)?;

        let r = bucket.transaction(|tx_db| {
            for tx in block.transactions.iter() {
                if !tx.is_coinbase() {
                    for vin in tx.vin.iter() {
                        let oiv = tx_db.get(vin.txid.clone())?;
                        if let Some(iv) = oiv {
                            let outs: Vec<TxOutput> = serde_json::from_slice(iv.as_ref())
                                .map_err(|e| ConflictableTransactionError::Abort(anyhow!(e)))?;

                            let new_outs: Vec<TxOutput> = outs
                                .into_iter()
                                .enumerate()
                                .filter(|(index, _)| *index != vin.vout as usize)
                                .map(|(_, v)| v)
                                .collect();

                            if new_outs.is_empty() {
                                tx_db.remove(vin.txid.as_bytes())?;
                            } else {
                                let value = serde_json::to_string(&new_outs)
                                    .map_err(|e| ConflictableTransactionError::Abort(anyhow!(e)))?;
                                tx_db.insert(vin.txid.as_bytes(), value.as_bytes())?;
                            }
                        }
                    }
                }

                let value = serde_json::to_string(&tx.vout)
                    .map_err(|e| ConflictableTransactionError::Abort(anyhow!(e)))?;
                tx_db.insert(tx.id.as_bytes(), value.as_bytes())?;
            }

            Ok(())
        });

        match r {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!(e)),
        }
    }
}
