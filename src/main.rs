use anyhow::Result;
use blockchain::Blockchain;
mod block;
mod blockchain;

fn main() -> Result<()> {
    let mut bc = Blockchain::new_block_chain()?;
    bc.add_block("Send 1 btc to Zhangsan")?;
    bc.add_block("Send 2 more btc to ZhangSan")?;

    for block in bc.get_blocks().iter() {
        println!("Prev. hash: {}", block.get_prehash());
        println!("Data: {}", block.get_data());
        println!("Hash: {}", block.get_hash());
        println!("");
    }
    Ok(())
}
