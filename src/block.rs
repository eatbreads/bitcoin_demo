use super::*;
use bincode::serialize;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH, Duration};  // 合并所有 time 相关导入
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;  // 添加这行

const TARGET_HEXS: usize = 6;
/// Block keeps block headers
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub timestamp: u128,    // 添加 pub 使字段公开
    pub data: String,
    pub prev_block_hash: String,
    pub hash: String,
    pub nonce: i32,
}

impl Block {
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }
    pub fn get_prev_hash(&self) -> String {
        self.prev_block_hash.clone()
    }
    /// NewBlock creates and returns Block
    pub fn new_block(data: String, prev_block_hash: String) -> Result<Block> {
        let timestamp = SystemTime::now()
           .duration_since(SystemTime::UNIX_EPOCH)?
           .as_millis();
        
        let mut block = Block {
            timestamp,
            data,
            prev_block_hash,
            hash: String::new(),
            nonce: 0,
        };
        block.run_proof_of_work()?;
        Ok(block)
    }

    /// NewGenesisBlock creates and returns genesis Block
    pub fn new_genesis_block() -> Block {
        Block::new_block(String::from("生成创世块"), String::new()).unwrap()
    }
    fn run_proof_of_work(&mut self) -> Result<()> {
        println!("正在挖掘的区块包含\"{}\"", self.data);
        while !self.validate()? {
            self.nonce += 1;
        }

        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        self.hash = hasher.result_str();
        Ok(())
    }

    fn prepare_hash_data(&self) -> Result<Vec<u8>> {
        let content = (
            self.prev_block_hash.clone(),
            self.data.clone(),
            self.timestamp,
            TARGET_HEXS,
            self.nonce,
        );
        let bytes = serialize(&content)?;
        Ok(bytes)
    }

    fn validate(&self) -> Result<bool> {
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        let mut vec1: Vec<u8> = Vec::new();
        vec1.resize(TARGET_HEXS, '0' as u8);
        Ok(&hasher.result_str()[0..TARGET_HEXS] == String::from_utf8(vec1)?)
       
    }


    // 添加新方法：将时间戳转换为可读形式
    pub fn get_readable_time(&self) -> String {
        let d = Duration::from_millis(self.timestamp as u64);
        let datetime = DateTime::<Utc>::from(UNIX_EPOCH + d);
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}


impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "时间戳: {}", self.get_readable_time())?;
        writeln!(f, "数据: {}", self.data)?;
        writeln!(f, "前一个区块哈希: {}", self.prev_block_hash)?;
        writeln!(f, "当前区块哈希: {}", self.hash)?;
        writeln!(f, "工作量证明 Nonce: {}", self.nonce)?;
        Ok(())
    }
}