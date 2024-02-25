#![allow(unused_variables, dead_code, unused_imports)]

use anyhow::Result;
use block::Block;
use blockchain::Blockchain;
use clap::Parser;
use cli::Cli;
use wallet::Wallets;

use crate::{
    proof_of_work::ProofOfWork, transaction::Transaction, utxoset::UTXOSet,
    wallet::pubkey_hash_from_base58,
};

mod block;
mod blockchain;
mod cli;
mod error;
mod proof_of_work;
mod transaction;
mod utxoset;
mod wallet;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // let mut bc = Blockchain::new_block_chain("0xxxxxxx".into()).unwrap();
    // bc.add_block("Send 1 btc to Zhangsan".into())?;
    // bc.add_block("Send 2 more btc to ZhangSan".into())?;

    match cli.command {
        cli::Commands::Addblock { data } => {
            println!("Success!")
        }
        cli::Commands::CreateBlockChain { address } => {
            let bc = Blockchain::create_block_chain(address)?;
            let utxoset = UTXOSet::new(bc);
            utxoset.reindex()?;
            println!("Done");
        }

        cli::Commands::CreateWallet => {
            let mut wallets = Wallets::new_wallets()?;
            let address = wallets.create_wallet();
            wallets.save_to_file()?;
            println!("Your new address: {address}")
        }
        cli::Commands::GetBalance { address } => {
            let bc = Blockchain::new_block_chain()?;

            let pubek_hash = pubkey_hash_from_base58(address.as_str())?;

            let utxoset = UTXOSet::new(bc);

            let utxos = utxoset.find_utxo(pubek_hash.as_str())?;
            let mut balance = 0;
            for out in utxos {
                balance += out.value;
            }

            println!("Balance of {}:{}", address, balance);
        }
        cli::Commands::Send { from, to, amount } => {
            let mut bc = Blockchain::new_block_chain()?;
            let tx = Transaction::new_utxo_transaction(from, to, amount, &bc)?;
            let block = bc.mine_block(vec![tx])?;
            let utxoset = UTXOSet::new(bc);
            utxoset.update(block)?;
            println!("Send Success!");
        }
        cli::Commands::Reindex => {
            let bc = Blockchain::new_block_chain()?;
            let utxoset = UTXOSet::new(bc);
            utxoset.reindex()?;
            println!("Reindex ok!");
        }
        cli::Commands::PrintChain => {
            let bc = Blockchain::new_block_chain()?;
            let mut iterator = bc.iterator();
            loop {
                let block = iterator.next()?;
                println!("Prev. hash: {}", block.get_prehash());
                println!("Transaction: {:?}", block.transactions);
                println!("Hash: {}", block.get_hash());
                let pow = ProofOfWork::new_proof_of_work(block.clone());
                println!("POW: {}", pow.validate());
                println!("");
                if block.get_prehash().is_empty() {
                    break;
                }
            }
        }
    }
    Ok(())
}
