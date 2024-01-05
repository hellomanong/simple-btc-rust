use crate::block::Block;
use anyhow::Result;

#[derive(Debug)]
pub struct Blockchain {
    blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new_block_chain() -> Result<Self> {
        let block = new_genesis_block()?;
        let block_chain = Self {
            blocks: vec![block],
        };

        Ok(block_chain)
    }

    pub fn add_block(&mut self, data: impl Into<String>) -> Result<()> {
        let pre_block = self.blocks.last().unwrap(); //如果没有之前的区块，直接crash就行了
        let block = Block::new_block(pre_block.get_hash().into(), data.into())?;
        self.blocks.push(block);
        Ok(())
    }

    pub fn get_blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
}

pub fn new_genesis_block() -> Result<Block> {
    Block::new_block("".into(), "Genesis Block".into())
}
