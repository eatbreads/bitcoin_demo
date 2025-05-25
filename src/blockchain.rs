use super::*;
use crate::block::*;
use crate::transaction::*;
use std::fmt;
use failure::format_err;
use bincode::{deserialize, serialize};
use sled;
use std::collections::HashMap;


const GENESIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

/// Blockchain keeps a sequence of Blocks
#[derive(Debug)]
pub struct Blockchain {
    pub tip: String,
    pub db: sled::Db,
}
pub struct BlockchainIterator<'a> {
    current_hash: String,
    bc: &'a Blockchain,
}
impl Blockchain {
    /// NewBlockchain creates a new Blockchain with genesis Block
    pub fn new() -> Result<Blockchain> {
        info!("open blockchain");

        let db = sled::open("data/blocks")?;
        let hash = match db.get("LAST")? {
            Some(l) => l.to_vec(),
            None => Vec::new(),
        };
        info!("Found block database");
        let lasthash = if hash.is_empty() {
            String::new()
        } else {
            String::from_utf8(hash.to_vec())?
        };
        Ok(Blockchain { tip: lasthash, db })
    }
    /// CreateBlockchain creates a new blockchain DB
    pub fn create_blockchain(address: String) -> Result<Blockchain> {
        info!("Creating new blockchain");

        std::fs::remove_dir_all("data/blocks").ok();
        let db = sled::open("data/blocks")?;
        debug!("Creating new block database");
        let cbtx = Transaction::new_coinbase(address, String::from(GENESIS_COINBASE_DATA))?;
        let genesis: Block = Block::new_genesis_block(cbtx);
        db.insert(genesis.get_hash(), serialize(&genesis)?)?;
        db.insert("LAST", genesis.get_hash().as_bytes())?;
        let bc = Blockchain {
            tip: genesis.get_hash(),
            db,
        };
        bc.db.flush()?;
        Ok(bc)
    }
    /// MineBlock mines a new block with the provided transactions
    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Result<Block> {
        info!("mine a new block");

        for tx in &transactions {
            if !self.verify_transacton(tx)? {
                return Err(format_err!("ERROR: Invalid transaction"));
            }
        }

        let lasthash = self.db.get("LAST")?.unwrap();

        let newblock = Block::new_block(
            transactions,
            String::from_utf8(lasthash.to_vec())?,
            self.get_best_height()? + 1,
        )?;
        self.db.insert(newblock.get_hash(), serialize(&newblock)?)?;
        self.db.insert("LAST", newblock.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = newblock.get_hash();
        Ok(newblock)
    }
    /// 定义这个类的迭代器,这个迭代器里面会方
    pub fn iter(&self) -> BlockchainIterator {
        BlockchainIterator {
            current_hash: self.tip.clone(),
            bc: &self,
        }
    }
    /// FindUTXO finds and returns all unspent transaction outputs
    pub fn find_UTXO(&self) -> HashMap<String, TXOutputs> {
        let mut utxos: HashMap<String, TXOutputs> = HashMap::new();
        let mut spend_txos: HashMap<String, Vec<i32>> = HashMap::new();

        for block in self.iter() {
            for tx in block.get_transaction() {
                for index in 0..tx.vout.len() {
                    if let Some(ids) = spend_txos.get(&tx.id) {
                        if ids.contains(&(index as i32)) {
                            continue;
                        }
                    }

                    match utxos.get_mut(&tx.id) {
                        Some(v) => {
                            v.outputs.push(tx.vout[index].clone());
                        }
                        None => {
                            utxos.insert(
                                tx.id.clone(),
                                TXOutputs {
                                    outputs: vec![tx.vout[index].clone()],
                                },
                            );
                        }
                    }
                }

                if !tx.is_coinbase() {
                    for i in &tx.vin {
                        match spend_txos.get_mut(&i.txid) {
                            Some(v) => {
                                v.push(i.vout);
                            }
                            None => {
                                spend_txos.insert(i.txid.clone(), vec![i.vout]);
                            }
                        }
                    }
                }
            }
        }

        utxos
    }
    /// FindTransaction finds a transaction by its ID
    pub fn find_transacton(&self, id: &str) -> Result<Transaction> {
        for b in self.iter() {
            for tx in b.get_transaction() {
                if tx.id == id {
                    return Ok(tx.clone());
                }
            }
        }
        Err(format_err!("Transaction is not found"))
    }
    fn get_prev_TXs(&self, tx: &Transaction) -> Result<HashMap<String, Transaction>> {
        let mut prev_TXs = HashMap::new();
        for vin in &tx.vin {
            let prev_TX = self.find_transacton(&vin.txid)?;
            prev_TXs.insert(prev_TX.id.clone(), prev_TX);
        }
        Ok(prev_TXs)
    }
    /// SignTransaction signs inputs of a Transaction
    pub fn sign_transacton(&self, tx: &mut Transaction, private_key: &[u8]) -> Result<()> {
        let prev_TXs = self.get_prev_TXs(tx)?;
        tx.sign(private_key, prev_TXs)?;
        Ok(())
    }

    /// VerifyTransaction verifies transaction input signatures
    pub fn verify_transacton(&self, tx: & Transaction) -> Result<bool> {
        if tx.is_coinbase() {
            return Ok(true);
        }
        let prev_TXs = self.get_prev_TXs(tx)?;
        tx.verify(prev_TXs)
    }

    /// AddBlock saves the block into the blockchain
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        let data = serialize(&block)?;
        if let Some(_) = self.db.get(block.get_hash())? {
            return Ok(());
        }
        self.db.insert(block.get_hash(), data)?;

        let lastheight = self.get_best_height()?;
        if block.get_height() > lastheight {
            self.db.insert("LAST", block.get_hash().as_bytes())?;
            self.tip = block.get_hash();
            self.db.flush()?;
        }
        Ok(())
    }

    // GetBlock finds a block by its hash and returns it
    pub fn get_block(&self, block_hash: &str) -> Result<Block> {
        let data = self.db.get(block_hash)?.unwrap();
        let block = deserialize(&data.to_vec())?;
        Ok(block)
    }

    /// GetBestHeight returns the height of the latest block
    pub fn get_best_height(&self) -> Result<i32> {
        let lasthash = if let Some(h) = self.db.get("LAST")? {
            h
        } else {
            return Ok(-1);
        };
        let last_data = self.db.get(lasthash)?.unwrap();
        let last_block: Block = deserialize(&last_data.to_vec())?;
        Ok(last_block.get_height())
    }

    /// GetBlockHashes returns a list of hashes of all the blocks in the chain
    pub fn get_block_hashs(&self) -> Vec<String> {
        let mut list = Vec::new();
        for b in self.iter() {
            list.push(b.get_hash());
        }
        list
    }
    // /// FindUnspentTransactions returns a list of transactions containing unspent outputs
    // pub fn find_spendable_outputs(
    //     &self,
    //     pub_key_hash: &[u8],
    //     amount: i32,
    // ) -> (i32, HashMap<String, Vec<i32>>) {     
    //     //返回值未元组,第一个参数是已经累加的金额,第二个参数是一个map,key是txid,value是一个数组,数组里面是这个txid对应的output的index
    //     //结构类似这样
    //     //(8, {
    //     //     "tx1" => [0, 2],  // 交易tx1的第0个和第2个输出可用
    //     //     "tx2" => [1]      // 交易tx2的第1个输出可用
    //     // })
    //     let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();
    //     let mut accumulated = 0;
    //     let unspend_TXs = self.find_unspent_transactions(pub_key_hash);

    //     for tx in unspend_TXs {
    //         for index in 0..tx.vout.len() {
    //             if tx.vout[index].is_locked_with_key(pub_key_hash) && accumulated < amount {
    //                 match unspent_outputs.get_mut(&tx.id) {
    //                     Some(v) => v.push(index as i32),
    //                     None => {
    //                         unspent_outputs.insert(tx.id.clone(), vec![index as i32]);
    //                     }
    //                 }
    //                 accumulated += tx.vout[index].value;

    //                 if accumulated >= amount {
    //                     return (accumulated, unspent_outputs);
    //                 }
    //             }
    //         }
    //     }
    //     (accumulated, unspent_outputs)
    // }

    // ///如果这个交易中有未花费的output,就会返回这个交易
    // fn find_unspent_transactions(&self, pub_key_hash: &[u8]) -> Vec<Transaction> {
    //     //key为tx的id,value式为一个数组,数组里面是这个tx的output的index
    //     //结构类似这样
    //     //{
    //     //     "tx1" => [0, 2],  // 交易tx1的第0个和第2个输出已经被使用
    //     //     "tx2" => [1]      // 交易tx2的第1个输出已经被使用
    //     // }

    //     //key为tx的id,value式为一个数组,数组里面是这个tx的所有已经花费的output的index
    //     let mut spent_TXOs: HashMap<String, Vec<i32>> = HashMap::new();
    //     //对应的返回值,存储一堆未使用的tx
    //     let mut unspend_TXs: Vec<Transaction> = Vec::new();

    //     for block in self.iter() {//遍历每个块
    //         for tx in block.get_transaction() {//遍历每个块里面的每个tx
    //             for index in 0..tx.vout.len() {//遍历每个tx里面的每个output
    //                 //这边会先判断vout,因为遍历区块是从新到旧,所以第一次进来的vout肯定是可以用的
    //                 if let Some(ids) = spent_TXOs.get(&tx.id) {//先定位是否有键值对ps:上文的tx1
    //                     if ids.contains(&(index as i32)) { //键值对中是否有这个index,有就跳过ps:上文的[0,2]
    //                         continue;
    //                     }
    //                 }
    //                 //用地址鉴权,通过了就推入返回值
    //                 if tx.vout[index].is_locked_with_key(pub_key_hash) {
    //                     unspend_TXs.push(tx.to_owned())
    //                 }
    //             }
    //             //然后这里才推入vin,然后进入下一层循环之后,进入一个相对旧的区块,再用这个vin去判断他的vout有没有花费
    //             if !tx.is_coinbase() {//如果是矿工的奖励,就跳过,不需要被记录
    //                 for i in &tx.vin {  //遍历全部的vin
    //                     if i.uses_key(pub_key_hash){
    //                         match spent_TXOs.get_mut(&i.txid) {
    //                             //如果这个交易已经记录过了,直接把他的vec新增即可
    //                             Some(v) => {
    //                                 v.push(i.vout);
    //                             }
    //                             None => {//如果没有记录过,就新建一个string和vec的键值对
    //                                 spent_TXOs.insert(i.txid.clone(), vec![i.vout]);
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     unspend_TXs
    // }
}

impl<'a>  Iterator for BlockchainIterator<'a>{
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(encoded_block) = self.bc.db.get(&self.current_hash) {
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