use super::*;
use bincode::{deserialize, serialize};
use bitcoincash_addr::*;
use crypto::digest::Digest;
use crypto::ed25519;
use crypto::ripemd160::Ripemd160;
use crypto::sha2::Sha256;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sled;
use std::collections::HashMap;
 
#[derive(Serialize,  Deserialize, Debug,Clone,PartialEq)]
pub struct Wallet {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

impl Wallet {
    fn new() -> Self {
        let mut key: [u8; 32] = [0; 32];
        let mut rand = rand::OsRng::new().unwrap();//这是操作系统级别的随机数
        rand.fill_bytes(&mut key);//填充这个随机数到key中
        //然后用这个随机数生成公钥和私钥,ed25519是一种椭圆曲线算法
        //keypair是生成公钥和私钥的函数
        let (secret_key, public_key) = ed25519::keypair(&key);
        let secret_key = secret_key.to_vec();
        let public_key = public_key.to_vec();
        Wallet {
            secret_key,
            public_key,
        }
    }
    pub fn get_address(&self) -> String {
        let mut pub_hash: Vec<u8> = self.public_key.clone();
        hash_pub_key(&mut pub_hash);    //对这个公钥进行两次哈希
        let address = Address {
            body: pub_hash,
            scheme: Scheme::Base58, //使用base58编码
            hash_type: HashType::Script,
            ..Default::default()
        };
        address.encode().unwrap()
    }
}

pub fn hash_pub_key(pubKey: &mut Vec<u8>) {
    let mut hasher1 = Sha256::new();
    hasher1.input(pubKey);//对这个公钥先进行哈希
    hasher1.result(pubKey);//把结果拷贝进公钥
    let mut hasher2 = Ripemd160::new();
    hasher2.input(pubKey);//再使用ripemd160进行哈希
    pubKey.resize(20, 0);//把公钥的长度变成20
    hasher2.result(pubKey);
}

pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets {
    pub fn new() -> Result<Wallets> {
        let mut wlt = Wallets {
            wallets: HashMap::<String, Wallet>::new(),
        };
        let db = sled::open("data/wallets")?;

        for item in db.into_iter() {    //打开数据库,遍历所有的钱包,加入内存中
            let i = item?;
            let address = String::from_utf8(i.0.to_vec())?;
            let wallet = deserialize(&i.1.to_vec())?;
            wlt.wallets.insert(address, wallet);    
        }
        Ok(wlt)
    }

    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        self.wallets.insert(address.clone(), wallet);
        info!("create wallet: {}", address);
        address
    }
    //拼接一个vec返回哈希表中的所有钱包地址(没有钱包)
    pub fn get_all_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::<String>::new();
        for (address, _) in &self.wallets {
            addresses.push(address.clone());
        }
        addresses
    }
    //返回单个钱包
    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }
    //把内存中的钱包保存到数据库中
    pub fn save_all(&self) -> Result<()> {
        let db = sled::open("data/wallets")?;

        for (address, wallet) in &self.wallets {
            let data = serialize(wallet)?;
            db.insert(address, data)?;
        }

        db.flush()?;
        drop(db);
        Ok(())
    }
}