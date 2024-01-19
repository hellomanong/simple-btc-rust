use anyhow::anyhow;
use anyhow::Result;
use base58::FromBase58;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::blockchain::Blockchain;
use crate::wallet::hash_pubkey;
use crate::wallet::pubkey_hash_from_base58;
use crate::wallet::Wallets;

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
    pub signature: String,
    pub pubkey: Vec<u8>, // 原始公钥，未hash
}

impl TxInput {
    pub fn uses_key(&self, pubkey_hash: &str) -> bool {
        let locking_hash = hash_pubkey(&self.pubkey);
        locking_hash == pubkey_hash.as_bytes()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TxOutput {
    pub value: isize,
    pub pubkey_hash: String,
}

impl TxOutput {
    pub fn new_tx_output(value: isize, address: String) -> Result<Self> {
        let mut out = Self {
            value,
            pubkey_hash: Default::default(),
        };

        out.lock(address.as_str())?;
        Ok(out)
    }
}

impl TxOutput {
    pub fn is_locked_with_key(&self, pubkey_hash: &str) -> bool {
        self.pubkey_hash == pubkey_hash
    }

    pub fn lock(&mut self, address: &str) -> Result<()> {
        self.pubkey_hash = pubkey_hash_from_base58(address)?;
        Ok(())
    }
}

impl Transaction {
    pub fn new_utxo_transaction(
        from: String,
        to: String,
        amount: isize,
        bc: &Blockchain,
    ) -> Result<Transaction> {
        let mut inputs = vec![];
        let mut outputs = vec![];

        let wallets = Wallets::new_wallets();
        let wallet = wallets.get_wallet(from.as_str())?;
        let pubkey_hash = hash_pubkey(&wallet.public_key);

        let (acc, valid_outputs) =
            bc.find_spentable_outputs(hex::encode(pubkey_hash).as_str(), amount)?;
        if acc < amount {
            return Err(anyhow!("Error: Not enough funds"));
        }

        for (txid, outs) in valid_outputs {
            let input: Vec<TxInput> = outs
                .into_iter()
                .map(|out| TxInput {
                    txid: txid.clone(),
                    vout: out,
                    signature: "".into(),
                    pubkey: wallet.public_key.clone(),
                })
                .collect();
            inputs.extend(input);
        }

        outputs.push(TxOutput::new_tx_output(amount, to)?);

        // 找零
        if acc > amount {
            let other_output = TxOutput::new_tx_output(acc - amount, from)?;
            outputs.push(other_output);
        }

        let tx = Transaction {
            id: "".into(),
            vin: inputs,
            vout: outputs,
        };

        Ok(tx)
    }

    pub fn new_coin_base_tx(to: String, mut data: String) -> Result<Self> {
        if data.is_empty() {
            data = format!("Reward to {}", to);
        }

        let txin = TxInput {
            txid: "".into(),
            vout: -1,
            signature: "".into(),
            pubkey: data.into_bytes(),
        };

        let txout = TxOutput::new_tx_output(SUBSIDY, to)?;

        let tx = Transaction {
            id: "".into(),
            vin: vec![txin],
            vout: vec![txout],
        };

        Ok(tx)
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
