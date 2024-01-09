use anyhow::Result;
use blockchain::Blockchain;

use crate::proof_of_work::ProofOfWork;

mod block;
mod blockchain;
mod cli;
mod error;
mod proof_of_work;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut bc = Blockchain::new_block_chain().unwrap();
    bc.add_block("Send 1 btc to Zhangsan".into())?;
    bc.add_block("Send 2 more btc to ZhangSan".into())?;

    let mut iterator = bc.iterator();
    loop {
        let block = iterator.next()?;
        println!("Prev. hash: {}", block.get_prehash());
        println!("Data: {}", block.get_data());
        println!("Hash: {}", block.get_hash());
        let pow = ProofOfWork::new_proof_of_work(block.clone());
        println!("POW: {}", pow.validate());
        println!("");
        if block.get_prehash().is_empty() {
            break;
        }
    }

    Ok(())
}
