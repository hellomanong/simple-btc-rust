use anyhow::anyhow;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::blockchain::Blockchain;

const SUBSIDY: isize = 50;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Transaction {
    pub id: String,
    pub vin: Vec<TxInput>,
    pub vout: Vec<TxOutput>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TxInput {
    pub txid: String,
    pub vout: isize,
    pub script_sig: String,
}

impl TxInput {
    pub fn can_unlock_output_with(&self, unlocing_data: &str) -> bool {
        self.script_sig == unlocing_data
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TxOutput {
    pub value: isize,
    pub script_pubkey: String,
}

impl TxOutput {
    pub fn can_be_unlocked_with(&self, unlocking_data: &str) -> bool {
        self.script_pubkey == unlocking_data
    }
}

impl Transaction {
    pub fn new_utxo_transction(
        from: String,
        to: String,
        amount: isize,
        bc: &Blockchain,
    ) -> Result<Transaction> {
        let mut inputs = vec![];
        let mut outputs = vec![];

        let (acc, valid_outputs) = bc.find_spentable_outputs(from.as_str(), amount)?;
        if acc < amount {
            return Err(anyhow!("Error: Not enough funds"));
        }

        for (txid, outs) in valid_outputs {
            for out in outs {
                let input = TxInput {
                    txid: txid.clone(),
                    vout: out,
                    script_sig: from.clone(),
                };

                inputs.push(input);
            }
        }

        let output = TxOutput {
            value: amount,
            script_pubkey: to,
        };

        outputs.push(output);

        // 找零
        if acc > amount {
            let other_output = TxOutput {
                value: acc - amount,
                script_pubkey: from,
            };
            outputs.push(other_output);
        }

        let tx = Transaction {
            id: "".into(),
            vin: inputs,
            vout: outputs,
        };

        Ok(tx)
    }

    pub fn new_coin_base_tx(to: String, mut data: String) -> Self {
        if data.is_empty() {
            data = format!("Reward to {}", to);
        }

        let txin = TxInput {
            txid: "".into(),
            vout: -1,
            script_sig: data,
        };

        let txout = TxOutput {
            value: SUBSIDY,
            script_pubkey: to,
        };

        let tx = Transaction {
            id: "".into(),
            vin: vec![txin],
            vout: vec![txout],
        };

        tx
    }

    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }

    pub fn set_id(&mut self) -> Result<()> {
        let data = serde_json::to_string(self).map_err(|e| {
            error!("Serialize transaction err: {e}");
            e
        })?;

        let id = sha256::digest(data);
        self.id = id;
        Ok(())
    }
}
