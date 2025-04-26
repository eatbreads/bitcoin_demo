use super::*;
use crate::block::*;
use std::fmt;


/// Blockchain keeps a sequence of Blocks
#[derive(Debug)]
pub struct Blockchain {
    blocks: Vec<Block>,
}

impl Blockchain {
    /// NewBlockchain creates a new Blockchain with genesis Block
    pub fn new() -> Blockchain {
        Blockchain {
            blocks: vec![Block::new_genesis_block()],
        }
    }

    /// AddBlock saves provided data as a block in the blockchain
    pub fn add_block(&mut self, data: String) -> Result<()> {
        let prev = self.blocks.last().unwrap();
        let newblock = Block::new_block(data, prev.get_hash())?;
        self.blocks.push(newblock);
        Ok(())
    }
}
impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, block) in self.blocks.iter().enumerate() {
            writeln!(f, "区块 #{}", i)?;
            writeln!(f, "时间戳: {}", block.get_readable_time())?;
            writeln!(f, "数据: {}", block.data)?;
            writeln!(f, "前一个区块哈希: {}", block.prev_block_hash)?;
            writeln!(f, "当前区块哈希: {}", block.hash)?;
            writeln!(f, "工作量证明 Nonce: {}", block.nonce)?;
            writeln!(f, "")?;
        }
        Ok(())
    }
}