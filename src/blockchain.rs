use super::*;
use crate::block::*;
use std::fmt;
use bincode::{deserialize, serialize};
use sled;

/// Blockchain keeps a sequence of Blocks
#[derive(Debug)]
pub struct Blockchain {
    tip: String,
    current_hash: String,
    db: sled::Db,
}

impl Blockchain {
    /// NewBlockchain creates a new Blockchain with genesis Block
    pub fn new() -> Result<Blockchain>{
        info!("New Blockchain");
        let db = sled::open("data/blockchain")?;
        match db.get("LAST")? {
           Some(hash) => {
               info!("从数据库中读取区块链");
               let current_hash = String::from_utf8_lossy(&hash).to_string();
               let lasthash = String::from_utf8(hash.to_vec())?;
               Ok(Blockchain {
                   tip: lasthash.clone(),
                   current_hash: lasthash,
                   db,
               })
           }
           None => {
                info!("生成创世区块");
                let block = Block::new_genesis_block();
                db.insert(block.get_hash(), serialize(&block)?)?;
                db.insert("LAST", block.get_hash().as_bytes())?;
                let bc = Blockchain {
                    tip: block.get_hash(),
                    current_hash: block.get_hash(),
                    db,
                };
                bc.db.flush()?;
                Ok(bc)
           },
        }
    }

    /// AddBlock saves provided data as a block in the blockchain
    pub fn add_block(&mut self, data: String) -> Result<()> {
        info!("添加区块");

        let lasthash = self.db.get("LAST")?.unwrap();

        let newblock = Block::new_block(data,String::from_utf8(lasthash.to_vec())?)?;
        self.db.insert(newblock.get_hash(), serialize(&newblock)?)?;
        self.db.insert("LAST", newblock.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = newblock.get_hash();
        self.current_hash = newblock.get_hash();
        Ok(())
    }
}

//这段由于结构变化导致弃用,blockchain当中不存放block了

// impl fmt::Display for Blockchain {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         for (i, block) in self.blocks.iter().enumerate() {
//             writeln!(f, "区块 #{}", i)?;
//             writeln!(f, "时间戳: {}", block.get_readable_time())?;
//             writeln!(f, "数据: {}", block.data)?;
//             writeln!(f, "前一个区块哈希: {}", block.prev_block_hash)?;
//             writeln!(f, "当前区块哈希: {}", block.hash)?;
//             writeln!(f, "工作量证明 Nonce: {}", block.nonce)?;
//             writeln!(f, "")?;
//         }
//         Ok(())
//     }
// }


impl Iterator for Blockchain {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(encoded_block) = self.db.get(&self.current_hash) {
            return match encoded_block {
                Some(b) => {
                    if let Ok(block) = deserialize::<Block>(&b) {
                        self.current_hash = block.get_prev_hash();
                        Some(block)
                    } else {
                        None
                    }
                }
                None => None,
            };
        }
        None
    }
}