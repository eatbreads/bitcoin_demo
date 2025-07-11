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
    height: i32,
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
    pub fn get_height(&self) -> i32 {
        self.height
    }
    /// NewBlock creates and returns Block
    /// NewBlock creates and returns Block
    pub fn new_block(
        transactions: Vec<Transaction>,
        prev_block_hash: String,
        height: i32,
    ) -> Result<Block> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let mut block = Block {
            timestamp,
            transactions,
            prev_block_hash,
            hash: String::new(),
            nonce: 0,
            height,
        };
        block.run_proof_of_work()?;
        Ok(block)
    }

    /// NewGenesisBlock creates and returns genesis Block
    pub fn new_genesis_block(coinbase: Transaction) -> Block {
        Block::new_block(vec![coinbase], String::new(), 0).unwrap()
    }
    // fn run_proof_of_work(&mut self) -> Result<()> {
    //     println!("正在挖掘的区块包含\"{:#?}\"\n", self.transactions);
    //     while !self.validate()? {
    //         self.nonce += 1;
    //     }

    //     let data = self.prepare_hash_data()?;
    //     let mut hasher = Sha256::new();
    //     hasher.input(&data[..]);
    //     self.hash = hasher.result_str();
    //     Ok(())
    // }
    fn run_proof_of_work(&mut self) -> Result<()> {
        println!("⛏️  开始挖矿...");
        println!("📦 区块信息:");
        println!("   📏 高度: {}", self.height);
        println!("   📊 交易数量: {}", self.transactions.len());
        println!("   ⏰ 时间戳: {}", self.get_readable_time());
        
        // 美化显示交易信息
        for (i, tx) in self.transactions.iter().enumerate() {
            if tx.is_coinbase() {
                println!("   💰 交易 {}: Coinbase奖励 ({}币)", i + 1, crate::transaction::SUBSIDY);
            } else {
                println!("   💸 交易 {}: ID={}", i + 1, &tx.id[..8]);
            }
        }
        
        print!("🔍 正在寻找合适的Nonce");
        let mut attempts = 0;
        while !self.validate()? {
            self.nonce += 1;
            attempts += 1;
            if attempts % 10000 == 0 {
                print!(".");
            }
        }
        println!();
        
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        self.hash = hasher.result_str();
        
        println!("✅ 挖矿成功!");
        println!("   🎲 Nonce: {}", self.nonce);
        println!("   🔗 区块哈希: {}...", &self.hash[..16]);
        println!();
        
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



impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "╭─────────────────────────────────────────────────────────────╮")?;
        writeln!(f, "│                      🧱 区块信息 🧱                        │")?;
        writeln!(f, "├─────────────────────────────────────────────────────────────┤")?;
        writeln!(f, "│ 📏 区块高度: {:>46} │", self.height)?;
        writeln!(f, "│ ⏰ 时间戳:   {:>46} │", self.get_readable_time())?;
        writeln!(f, "│ 📊 交易数量: {:>46} │", self.transactions.len())?;
        writeln!(f, "│ 🎲 Nonce:    {:>46} │", self.nonce)?;
        writeln!(f, "├─────────────────────────────────────────────────────────────┤")?;
        writeln!(f, "│ ⬅️  前区块哈希:                                              │")?;
        let prev_hash_display = if self.prev_block_hash.is_empty() { 
            "🌟 [创世区块]".to_string() 
        } else { 
            self.prev_block_hash.clone() 
        };
        writeln!(f, "│ {:>59} │", prev_hash_display)?;
        writeln!(f, "├─────────────────────────────────────────────────────────────┤")?;
        writeln!(f, "│ 🆔 当前哈希:                                                │")?;
        writeln!(f, "│ {:>59} │", self.hash)?;
        writeln!(f, "├─────────────────────────────────────────────────────────────┤")?;
        writeln!(f, "│                      💰 交易列表 💰                        │")?;
        writeln!(f, "├─────────────────────────────────────────────────────────────┤")?;
        
        for (i, tx) in self.transactions.iter().enumerate() {
            writeln!(f, "│ 📋 交易 {}: {:>49} │", i + 1, "")?;
            let tx_str = format!("{}", tx);
            for line in tx_str.lines() {
                let display_line = if line.len() > 57 { &line[..57] } else { line };
                writeln!(f, "│   {:57} │", display_line)?;
            }
            if i < self.transactions.len() - 1 {
                writeln!(f, "├─────────────────────────────────────────────────────────────┤")?;
            }
        }
        
        writeln!(f, "╰─────────────────────────────────────────────────────────────╯")?;
        Ok(())
    }
}
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
