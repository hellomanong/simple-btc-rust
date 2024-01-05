use anyhow::Result;
use blockchain::Blockchain;

use crate::proof_of_work::ProofOfWork;
mod block;
mod blockchain;
mod proof_of_work;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut bc = Blockchain::new_block_chain()?;
    bc.add_block("Send 1 btc to Zhangsan")?;
    bc.add_block("Send 2 more btc to ZhangSan")?;

    for block in bc.get_blocks().iter() {
        println!("Prev. hash: {}", block.get_prehash());
        println!("Data: {}", block.get_data());
        println!("Hash: {}", block.get_hash());
        let pow = ProofOfWork::new_proof_of_work(block.clone());
        println!("POW: {}", pow.validate());
        println!("");
    }
    Ok(())
}
