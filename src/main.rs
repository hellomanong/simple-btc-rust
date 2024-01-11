#![allow(unused_variables, dead_code, unused_imports)]

use anyhow::Result;
use block::Block;
use blockchain::Blockchain;
use clap::Parser;
use cli::Cli;

use crate::proof_of_work::ProofOfWork;

mod block;
mod blockchain;
mod cli;
mod error;
mod proof_of_work;
mod transaction;

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
            println!("Done");
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
