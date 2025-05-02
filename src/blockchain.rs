use super::*;
use crate::block::*;
use crate::transaction::*;
use std::fmt;
use bincode::{deserialize, serialize};
use sled;
use std::collections::HashMap;


const GENESIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

/// Blockchain keeps a sequence of Blocks
#[derive(Debug)]
pub struct Blockchain {
    tip: String,
    db: sled::Db,
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
        let hash = db
            .get("LAST")?
            .expect("Must create a new block database first");
        info!("Found block database");
        let lasthash = String::from_utf8(hash.to_vec())?;
        Ok(Blockchain {
            tip: lasthash.clone(),
            db,
        })
    }
    /// CreateBlockchain creates a new blockchain DB
    pub fn create_blockchain(address: String) -> Result<Blockchain> {
        info!("Creating new blockchain");

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
    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Result<()> {
        info!("mine a new block");
        let lasthash = self.db.get("LAST")?.unwrap();

        let newblock = Block::new_block(transactions, String::from_utf8(lasthash.to_vec())?)?;
        self.db.insert(newblock.get_hash(), serialize(&newblock)?)?;
        self.db.insert("LAST", newblock.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = newblock.get_hash();
        Ok(())
    }
    /// 定义这个类的迭代器,这个迭代器里面会方
    pub fn iter(&self) -> BlockchainIterator {
        BlockchainIterator {
            current_hash: self.tip.clone(),
            bc: &self,
        }
    }
    ///仅仅把find_unspent_transactions的返回的tx提纯,直接返回UTXO
    pub fn find_UTXO(&self, pub_key_hash: &[u8]) -> Vec<TXOutput> {
        let mut utxos: Vec<TXOutput> = Vec::<TXOutput>::new();
        let unspend_TXs: Vec<Transaction> = self.find_unspent_transactions(pub_key_hash);
        for tx in unspend_TXs {
            for out in &tx.vout {
                if out.is_locked_with_key(pub_key_hash) {
                    utxos.push(out.clone());
                }
            }
        }
        utxos
    }

    /// FindUnspentTransactions returns a list of transactions containing unspent outputs
    pub fn find_spendable_outputs(
        &self,
        pub_key_hash: &[u8],
        amount: i32,
    ) -> (i32, HashMap<String, Vec<i32>>) {     
        //返回值未元组,第一个参数是已经累加的金额,第二个参数是一个map,key是txid,value是一个数组,数组里面是这个txid对应的output的index
        //结构类似这样
        //(8, {
        //     "tx1" => [0, 2],  // 交易tx1的第0个和第2个输出可用
        //     "tx2" => [1]      // 交易tx2的第1个输出可用
        // })
        let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();
        let mut accumulated = 0;
        let unspend_TXs = self.find_unspent_transactions(pub_key_hash);

        for tx in unspend_TXs {
            for index in 0..tx.vout.len() {
                if tx.vout[index].is_locked_with_key(pub_key_hash) && accumulated < amount {
                    match unspent_outputs.get_mut(&tx.id) {
                        Some(v) => v.push(index as i32),
                        None => {
                            unspent_outputs.insert(tx.id.clone(), vec![index as i32]);
                        }
                    }
                    accumulated += tx.vout[index].value;

                    if accumulated >= amount {
                        return (accumulated, unspent_outputs);
                    }
                }
            }
        }
        (accumulated, unspent_outputs)
    }

    ///如果这个交易中有未花费的output,就会返回这个交易
    fn find_unspent_transactions(&self, pub_key_hash: &[u8]) -> Vec<Transaction> {
        //key为tx的id,value式为一个数组,数组里面是这个tx的output的index
        //结构类似这样
        //{
        //     "tx1" => [0, 2],  // 交易tx1的第0个和第2个输出已经被使用
        //     "tx2" => [1]      // 交易tx2的第1个输出已经被使用
        // }

        //key为tx的id,value式为一个数组,数组里面是这个tx的所有已经花费的output的index
        let mut spent_TXOs: HashMap<String, Vec<i32>> = HashMap::new();
        //对应的返回值,存储一堆未使用的tx
        let mut unspend_TXs: Vec<Transaction> = Vec::new();

        for block in self.iter() {//遍历每个块
            for tx in block.get_transaction() {//遍历每个块里面的每个tx
                for index in 0..tx.vout.len() {//遍历每个tx里面的每个output
                    //这边会先判断vout,因为遍历区块是从新到旧,所以第一次进来的vout肯定是可以用的
                    if let Some(ids) = spent_TXOs.get(&tx.id) {//先定位是否有键值对ps:上文的tx1
                        if ids.contains(&(index as i32)) { //键值对中是否有这个index,有就跳过ps:上文的[0,2]
                            continue;
                        }
                    }
                    //用地址鉴权,通过了就推入返回值
                    if tx.vout[index].is_locked_with_key(pub_key_hash) {
                        unspend_TXs.push(tx.to_owned())
                    }
                }
                //然后这里才推入vin,然后进入下一层循环之后,进入一个相对旧的区块,再用这个vin去判断他的vout有没有花费
                if !tx.is_coinbase() {//如果是矿工的奖励,就跳过,不需要被记录
                    for i in &tx.vin {  //遍历全部的vin
                        if i.uses_key(pub_key_hash){
                            match spent_TXOs.get_mut(&i.txid) {
                                //如果这个交易已经记录过了,直接把他的vec新增即可
                                Some(v) => {
                                    v.push(i.vout);
                                }
                                None => {//如果没有记录过,就新建一个string和vec的键值对
                                    spent_TXOs.insert(i.txid.clone(), vec![i.vout]);
                                }
                            }
                        }
                    }
                }
            }
        }

        unspend_TXs
    }
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