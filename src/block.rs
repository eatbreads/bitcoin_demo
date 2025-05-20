use super::*;
use crate::transaction::Transaction;
use bincode::serialize;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use merkle_cbt::merkle_tree::Merge;
use merkle_cbt::merkle_tree::CBMT;
use std::time::{SystemTime, UNIX_EPOCH, Duration};  // 合并所有 time 相关导入
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;  // 添加这行

const TARGET_HEXS: usize = 5;
/// Block keeps block headers
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub timestamp: u128,    // 添加 pub 使字段公开
    pub transactions: Vec<Transaction>,
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

    pub fn get_transaction(&self) -> &Vec<Transaction> {
        &self.transactions
    }
    /// NewBlock creates and returns Block
    pub fn new_block(transactions: Vec<Transaction>, prev_block_hash: String) -> Result<Block> {
        let timestamp = SystemTime::now()
           .duration_since(SystemTime::UNIX_EPOCH)?
           .as_millis();
        
        let mut block = Block {
            timestamp,
            transactions,
            prev_block_hash,
            hash: String::new(),
            nonce: 0,
        };
        block.run_proof_of_work()?;
        Ok(block)
    }

    /// NewGenesisBlock creates and returns genesis Block
    pub fn new_genesis_block(coinbase: Transaction) -> Block {
        Block::new_block(vec![coinbase], String::new()).unwrap()
    }
    fn run_proof_of_work(&mut self) -> Result<()> {
        println!("正在挖掘的区块包含\"{:#?}\"\n", self.transactions);
        while !self.validate()? {
            self.nonce += 1;
        }

        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        self.hash = hasher.result_str();
        Ok(())
    }

    fn hash_transactions(&self) -> Result<Vec<u8>> {
        let mut transactions = Vec::new();
        for tx in &self.transactions {
            transactions.push(tx.hash()?.as_bytes().to_owned());
        }
        let tree = CBMT::<Vec<u8>, MergeVu8>::build_merkle_tree(transactions);

        Ok(tree.root())
    }
    fn prepare_hash_data(&self) -> Result<Vec<u8>> {
        let content = (
            self.prev_block_hash.clone(),
            self.hash_transactions()?,
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


// impl fmt::Display for Block {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         writeln!(f, "Block [")?;
//         writeln!(f, "  时间: {}", self.get_readable_time())?;
//         writeln!(f, "  交易列表:")?;
//         for (i, tx) in self.transactions.iter().enumerate() {
//             writeln!(f, "  {}. {}", i + 1, tx)?;
//         }
//         writeln!(f, "  前区块哈希: {}", self.prev_block_hash)?;
//         writeln!(f, "  当前哈希: {}", self.hash)?;
//         writeln!(f, "  Nonce: {}", self.nonce)?;
//         write!(f, "]")
//     }
// }
struct MergeVu8 {}

impl Merge for MergeVu8 {
    type Item = Vec<u8>;
    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
        let mut hasher = Sha256::new();
        let mut data: Vec<u8> = left.clone();
        data.append(&mut right.clone());
        hasher.input(&data);
        let mut re: [u8; 32] = [0; 32];
        hasher.result(&mut re);
        re.to_vec()
    }
}
