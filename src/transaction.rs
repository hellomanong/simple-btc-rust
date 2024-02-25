use std::collections::HashMap;
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::Result;
use base58::FromBase58;
use ecdsa::{signature::Signer, signature::Verifier, Signature, SigningKey, VerifyingKey};
use p256::NistP256;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::blockchain::Blockchain;
use crate::utxoset;
use crate::utxoset::UTXOSet;
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
    pub txid: String, // 引用的交易
    pub vout: isize,  // 引用的交易中，输出的索引
    pub signature: String,
    pub pubkey: Vec<u8>, // 原始公钥，未hash
}

impl TxInput {
    pub fn uses_key(&self, pubkey_hash: &str) -> bool {
        let locking_hash = hash_pubkey(&self.pubkey);
        hex::encode(&locking_hash) == pubkey_hash
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
        let mut inputs: Vec<TxInput> = vec![];
        let mut outputs = vec![];

        let wallets = Wallets::new_wallets()?;
        let wallet = wallets.get_wallet(from.as_str())?;
        let pubkey_hash = hash_pubkey(&wallet.public_key);

        let utxoset = UTXOSet::new(bc.clone());

        let (acc, valid_outputs) =
            utxoset.find_spentable_outputs(hex::encode(pubkey_hash).as_str(), amount)?;

        println!("-------------acc:{acc}----------------------");

        if acc < amount {
            return Err(anyhow!("Error: Not enough funds"));
        }

        for (txid, outs) in valid_outputs {
            let input: Vec<TxInput> = outs
                .into_iter()
                .map(|out| TxInput {
                    txid: txid.clone(),
                    vout: out,
                    signature: String::new(),
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

        let mut tx = Transaction {
            id: String::new(),
            vin: inputs,
            vout: outputs,
        };

        tx.set_id()?;

        bc.sign_transaction(&mut tx, wallet.secret_key.as_slice())?;

        Ok(tx)
    }

    pub fn new_coin_base_tx(to: String, mut data: String) -> Result<Self> {
        if data.is_empty() {
            data = format!("Reward to {}", to);
        }

        let txin = TxInput {
            txid: String::new(),
            vout: -1,
            signature: String::new(),
            pubkey: data.into_bytes(),
        };

        let txout = TxOutput::new_tx_output(SUBSIDY, to)?;

        let mut tx = Transaction {
            id: String::new(),
            vin: vec![txin],
            vout: vec![txout],
        };

        tx.set_id()?;

        Ok(tx)
    }
}

impl Transaction {
    pub fn sign(&mut self, privkey: &[u8], prev_txs: HashMap<String, Transaction>) -> Result<()> {
        if self.is_coinbase() {
            return Ok(());
        }

        let mut tx_copy = self.trimmed_copy();

        let signing_key: SigningKey<NistP256> = SigningKey::from_slice(privkey)?;
        for (in_id, vin) in self.vin.clone().iter().enumerate() {
            if let Some(prev_tx) = prev_txs.get(&vin.txid) {
                let pubkey = prev_tx.vout[vin.vout as usize].pubkey_hash.clone();
                tx_copy.vin[in_id].pubkey = pubkey.into();
                tx_copy.id = tx_copy.hash()?;
                tx_copy.vin[in_id].pubkey = Vec::new();

                let msg = hex::decode(tx_copy.id.as_str())?;
                let signature: Signature<NistP256> = signing_key.try_sign(msg.as_slice())?;
                self.vin[in_id].signature = hex::encode(signature.to_vec());
            }
        }

        Ok(())
    }

    pub fn verify(&self, prev_txs: HashMap<String, Transaction>) -> Result<bool> {
        let mut tx_copy = self.trimmed_copy();

        for (in_id, vin) in self.vin.iter().enumerate() {
            if let Some(prev_tx) = prev_txs.get(&vin.txid) {
                let pubkey = prev_tx.vout[vin.vout as usize].pubkey_hash.clone();
                tx_copy.vin[in_id].pubkey = pubkey.into();
                tx_copy.id = tx_copy.hash()?;
                tx_copy.vin[in_id].pubkey = Vec::new();

                let msg = hex::decode(tx_copy.id.as_str())?;
                let verfiying_key: VerifyingKey<NistP256> =
                    VerifyingKey::from_sec1_bytes(vin.pubkey.as_slice())?;

                let signature: Signature<NistP256> = Signature::from_str(vin.signature.as_str())?;
                if let Err(e) = verfiying_key.verify(msg.as_slice(), &signature) {
                    println!("verfiying_key err: {e}");
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn trimmed_copy(&self) -> Self {
        let inputs: Vec<TxInput> = self
            .vin
            .clone()
            .into_iter()
            .map(|v| TxInput {
                txid: v.txid,
                vout: v.vout,
                ..Default::default()
            })
            .collect();

        let outputs: Vec<TxOutput> = self
            .vout
            .clone()
            .into_iter()
            .map(|v| TxOutput {
                value: v.value,
                pubkey_hash: v.pubkey_hash,
            })
            .collect();

        Self {
            id: self.id.clone(),
            vin: inputs,
            vout: outputs,
        }
    }

    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }

    pub fn set_id(&mut self) -> Result<()> {
        self.id = self.hash()?;
        Ok(())
    }

    pub fn hash(&self) -> Result<String> {
        let data = serde_json::to_string(self).map_err(|e| {
            error!("Serialize transaction err: {e}");
            e
        })?;

        let hash = sha256::digest(data);
        Ok(hash)
    }
}
