use std::{
    collections::HashMap,
    fs::OpenOptions,
    hash::Hasher,
    io::{self, Read, Write},
    str::from_utf8,
};

use anyhow::anyhow;
use base58::{FromBase58, ToBase58};
use ecdsa::{
    elliptic_curve::{PublicKey, SecretKey},
    signature::rand_core::OsRng,
    SignatureEncoding, VerifyingKey,
};
use p256::{
    ecdsa::{signature::Signer, SigningKey},
    NistP256,
};

use ripemd::Ripemd160;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

const VERSION: u8 = 0x00;
const ADDRESS_CHECK_SUM_LEN: usize = 4;
const WALLET_FILE: &str = "./wallet.dat";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets {
    pub fn new_wallets() -> anyhow::Result<Self> {
        let mut wallets = Self {
            ..Default::default()
        };

        wallets.load_from_file()?;
        Ok(wallets)
    }
}

impl Wallets {
    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new_wallet();
        let address = wallet.get_address();
        self.wallets.insert(address.clone(), wallet);
        address
    }

    pub fn get_wallet(&self, address: &str) -> anyhow::Result<Wallet> {
        let wallet = self
            .wallets
            .get(address)
            .cloned()
            .map_or(Err(anyhow!("Get wallet, return None")), |v| Ok(v));
        wallet
    }

    pub fn save_to_file(&self) -> io::Result<()> {
        // 也可以直接使用 fs::write("path", "data");
        let mut file = OpenOptions::new()
            .write(true)
            // .append(true)
            .create(true)
            .open(WALLET_FILE)?;

        let data = serde_json::to_string(self)?;

        file.write_all(data.as_bytes()).map_err(|e| {
            println!("Write wallets to file err: {e}");
            e
        })
    }

    pub fn load_from_file(&mut self) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(WALLET_FILE)?;

        let mut buf = String::new();
        file.read_to_string(&mut buf).map_err(|e| {
            println!("Read wallets file err: {e}");
            e
        })?;

        if !buf.is_empty() {
            let wallets = serde_json::from_str::<Wallets>(buf.as_str())?;
            self.wallets = wallets.wallets;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Wallet {
    pub secret_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl Wallet {
    pub fn new_wallet() -> Self {
        let (secret_key, public_key) = new_key_pair();
        Self {
            secret_key,
            public_key,
        }
    }
}

impl Wallet {
    // version + public_key_hash + check_sum => base58
    pub fn get_address(&self) -> String {
        let mut pubkey_hash = hash_pubkey(&self.public_key);

        let mut versioned_payload = vec![];
        versioned_payload.push(VERSION);
        versioned_payload.append(&mut pubkey_hash);
        let check_sum: Vec<u8> = check_sum(&versioned_payload);

        versioned_payload.extend_from_slice(&check_sum[..ADDRESS_CHECK_SUM_LEN]);

        versioned_payload.to_base58()
    }
}

pub fn hash_pubkey(pubkey: &Vec<u8>) -> Vec<u8> {
    let pubkey_hash = sha256::digest(pubkey);
    Ripemd160::digest(pubkey_hash).to_vec()
}

fn check_sum(payload: &Vec<u8>) -> Vec<u8> {
    let first_sha = sha256::digest(payload);
    sha256::digest(first_sha).into_bytes()
}

fn new_key_pair() -> (Vec<u8>, Vec<u8>) {
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    let secret_key: SecretKey<NistP256> = signing_key.into();
    let pubkey = secret_key.public_key();
    (
        secret_key.to_bytes().to_vec(),
        pubkey.to_sec1_bytes().to_vec(),
    )
}

pub fn pubkey_hash_from_base58(address: &str) -> anyhow::Result<String> {
    match address.from_base58() {
        Ok(pubkey_hash) => {
            let pubkey_hash = &pubkey_hash[1..pubkey_hash.len() - 4];
            Ok(hex::encode(pubkey_hash.to_vec()))
        }
        Err(e) => Err(anyhow!("Decode address to pubkey hash err:{:?}", e)),
    }
}

#[cfg(test)]
mod test {
    use crate::wallet::hash_pubkey;

    use super::Wallet;

    #[test]
    fn test_get_address() {
        let wallet = Wallet::new_wallet();

        let pubkey = hash_pubkey(&wallet.public_key);

        let address = wallet.get_address();
        println!("------address:{address}------");
        println!("------pubkey.len:{}------", pubkey.len());

        assert_eq!(pubkey.len(), 20);
    }
}
