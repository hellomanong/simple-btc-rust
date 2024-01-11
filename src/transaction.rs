use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TxOutput {
    pub value: isize,
    pub script_pubkey: String,
}

impl Transaction {
    // pub fn new_utxo_transction(
    //     from: String,
    //     to: String,
    //     amount: isize,
    //     bc: &Blockchain,
    // ) -> Result<Transaction> {
    // }

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
}
