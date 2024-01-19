use std::{collections::HashMap, hash::Hasher, str::from_utf8};

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
use sha3::{Digest, Sha3_256};

const VERSION: u8 = 0x00;
const ADDRESS_CHECK_SUM_LEN: usize = 4;

#[derive(Debug, Default)]
pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets {
    pub fn new_wallets() -> Self {
        Default::default()
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
}

#[derive(Debug, Default, Clone)]
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
