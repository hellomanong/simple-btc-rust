use anyhow::anyhow;
use anyhow::Result;
use serde::{Deserialize, Serialize};

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
}
