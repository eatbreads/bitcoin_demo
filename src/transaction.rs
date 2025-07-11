use super::*;
use crate::utxoset::*;
use crate::wallets::*;
use bincode::serialize;
use bitcoincash_addr::Address;
use crypto::digest::Digest;
use crypto::ed25519;
use crypto::sha2::Sha256;
use failure::format_err;
use rand::Rng;  // 用于生成随机数
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
pub const SUBSIDY: i32 = 10;
/// TXInput represents a transaction input
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub txid: String,
    pub vout: i32,
    pub signature: Vec<u8>,
    pub pub_key: Vec<u8>,
}


/// TXOutput represents a transaction output
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32,
    pub pub_key_hash: Vec<u8>,
}

// TXOutputs collects TXOutput
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutputs {
    pub outputs: Vec<TXOutput>,
}
/// Transaction represents a Bitcoin transaction
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
}


impl Transaction {
    /// NewUTXOTransaction creates a new transaction
    pub fn new_UTXO(wallet: &Wallet, to: &str, amount: i32, utxo: &UTXOSet) -> Result<Transaction> {
        info!(
            "new UTXO Transaction from: {} to: {}",
            wallet.get_address(),
            to
        );
        let mut vin = Vec::new();

        let mut pub_key_hash = wallet.public_key.clone();
        hash_pub_key(&mut pub_key_hash);

        let acc_v = utxo.find_spendable_outputs(&pub_key_hash, amount)?;

        if acc_v.0 < amount {
            error!("Not Enough balance");
            return Err(format_err!(
                "Not Enough balance: current balance {}",
                acc_v.0
            ));
        }

        for tx in acc_v.1 {
            for out in tx.1 {
                let input = TXInput {
                    txid: tx.0.clone(),
                    vout: out,
                    signature: Vec::new(),
                    pub_key: wallet.public_key.clone(),
                };
                vin.push(input);
            }
        }

        let mut vout = vec![TXOutput::new(amount, to.to_string())?];
        if acc_v.0 > amount {
            vout.push(TXOutput::new(acc_v.0 - amount, wallet.get_address())?)
        }

        let mut tx = Transaction {
            id: String::new(),
            vin,
            vout,
        };
        tx.id = tx.hash()?;
        utxo.blockchain
            .sign_transacton(&mut tx, &wallet.secret_key)?;
        Ok(tx)
    }


    /// NewCoinbaseTX creates a new coinbase transaction
    pub fn new_coinbase(to: String, mut data: String) -> Result<Transaction> {
        info!("new coinbase Transaction to: {}", to);
        let mut key: [u8; 32] = [0; 32];
        if data.is_empty() {
            let mut rand = rand::OsRng::new().unwrap();
            rand.fill_bytes(&mut key);
            data = format!("Reward to '{}'", to);
        }
        let mut pub_key = Vec::from(data.as_bytes());
        pub_key.append(&mut Vec::from(key));

        let mut tx = Transaction {
            id: String::new(),
            vin: vec![TXInput {
                txid: String::new(),
                vout: -1,
                signature: Vec::new(),
                pub_key,
            }],
            vout: vec![TXOutput::new(SUBSIDY, to)?],
        };
        tx.id = tx.hash()?;
        Ok(tx)
    }



    /// IsCoinbase checks whether the transaction is coinbase
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }
    // pub fn is_coinbase(&self) -> bool {
    //     self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    // }
    pub fn verify(&self, prev_TXs: HashMap<String, Transaction>) -> Result<bool> {
        if self.is_coinbase() {
            return Ok(true);
        }

        for vin in &self.vin {
            if prev_TXs.get(&vin.txid).unwrap().id.is_empty() {
                return Err(format_err!("ERROR: Previous transaction is not correct"));
            }
        }

        let mut tx_copy = self.trim_copy();

        for in_id in 0..self.vin.len() {
            let prev_Tx = prev_TXs.get(&self.vin[in_id].txid).unwrap();
            tx_copy.vin[in_id].signature.clear();
            tx_copy.vin[in_id].pub_key = prev_Tx.vout[self.vin[in_id].vout as usize]
                .pub_key_hash
                .clone();
            tx_copy.id = tx_copy.hash()?;
            tx_copy.vin[in_id].pub_key = Vec::new();

            if !ed25519::verify(
                &tx_copy.id.as_bytes(),
                &self.vin[in_id].pub_key,
                &self.vin[in_id].signature,
            ) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn sign(
        &mut self,
        private_key: &[u8],
        prev_TXs: HashMap<String, Transaction>,
    ) -> Result<()> {
        if self.is_coinbase() {
            return Ok(());
        }

        for vin in &self.vin {
            if prev_TXs.get(&vin.txid).unwrap().id.is_empty() {
                return Err(format_err!("ERROR: Previous transaction is not correct"));
            }
        }

        let mut tx_copy = self.trim_copy();

        for in_id in 0..tx_copy.vin.len() {
            let prev_Tx = prev_TXs.get(&tx_copy.vin[in_id].txid).unwrap();
            tx_copy.vin[in_id].signature.clear();
            tx_copy.vin[in_id].pub_key = prev_Tx.vout[tx_copy.vin[in_id].vout as usize]
                .pub_key_hash
                .clone();
            tx_copy.id = tx_copy.hash()?;
            tx_copy.vin[in_id].pub_key = Vec::new();
            let signature = ed25519::signature(tx_copy.id.as_bytes(), private_key);
            self.vin[in_id].signature = signature.to_vec();
        }

        Ok(())
    }
    
    /// Hash returns the hash of the Transaction
    pub fn hash(&self) -> Result<String> {
        let mut copy = self.clone();
        copy.id = String::new();
        let data = serialize(&copy)?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        Ok(hasher.result_str())
    }

    /// TrimmedCopy creates a trimmed copy of Transaction to be used in signing
    fn trim_copy(&self) -> Transaction {
        let mut vin = Vec::new();
        let mut vout = Vec::new();

        for v in &self.vin {
            vin.push(TXInput {
                txid: v.txid.clone(),
                vout: v.vout.clone(),
                signature: Vec::new(),
                pub_key: Vec::new(),
            })
        }

        for v in &self.vout {
            vout.push(TXOutput {
                value: v.value,
                pub_key_hash: v.pub_key_hash.clone(),
            })
        }

        Transaction {
            id: self.id.clone(),
            vin,
            vout,
        }
    }
    fn pub_key_hash_to_address(&self, pub_key_hash: &[u8]) -> String {
        use bitcoincash_addr::{Address, Scheme, HashType};
        
        let address = Address {
            body: pub_key_hash.to_vec(),
            scheme: Scheme::Base58,
            hash_type: HashType::Script,
            ..Default::default()
        };
        
        address.encode().unwrap_or_else(|_| {
            format!("[❌ 无效地址: {}...]", 
                if pub_key_hash.len() >= 8 {
                    hex::encode(&pub_key_hash[..4])
                } else {
                    hex::encode(pub_key_hash)
                }
            )
        })
    }
}
// impl TXInput {
//     /// UsesKey checks whether the address initiated the transaction
//     pub fn uses_key(&self, pub_key_hash: &[u8]) -> bool {
//         let mut pubkeyhash = self.pub_key.clone();
//         hash_pub_key(&mut pubkeyhash);
//         pubkeyhash == pub_key_hash
//     }
// }
impl TXOutput {
    /// IsLockedWithKey checks if the output can be used by the owner of the pubkey
    pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash == pub_key_hash
    }
    /// Lock signs the output
    fn lock(&mut self, address: &str) -> Result<()> {
        let pub_key_hash = Address::decode(address).unwrap().body;
        debug!("lock: {}", address);
        self.pub_key_hash = pub_key_hash;
        Ok(())
    }

    pub fn new(value: i32, address: String) -> Result<Self> {
        let mut txo = TXOutput {
            value,
            pub_key_hash: Vec::new(),
        };
        txo.lock(&address)?;
        Ok(txo)
    }
}

// impl TXOutput {
//     /// CanBeUnlockedWith checks if the output can be unlocked with the provided data
//     pub fn can_be_unlock_with(&self, unlockingData: &str) -> bool {
//         self.script_pub_key == unlockingData
//     }
// }

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "💰 交易ID: {}...{}", 
            if self.id.len() >= 16 { &self.id[..8] } else { &self.id },
            if self.id.len() >= 16 { &self.id[self.id.len()-8..] } else { "" }
        )?;
        
        if self.is_coinbase() {
            writeln!(f, "🏆 类型: Coinbase交易 (挖矿奖励)")?;
            writeln!(f, "✨ 输入: 新生成的币")?;
        } else {
            writeln!(f, "🔄 类型: 普通转账交易")?;
            writeln!(f, "📥 输入:")?;
            for (i, input) in self.vin.iter().enumerate() {
                writeln!(f, "  {}. 🔗 来源交易: {}...{}", 
                    i + 1, 
                    if input.txid.len() >= 16 { &input.txid[..8] } else { &input.txid },
                    if input.txid.len() >= 16 { &input.txid[input.txid.len()-8..] } else { "" }
                )?;
                writeln!(f, "     📍 输出索引: {}", input.vout)?;
                writeln!(f, "     🔐 签名长度: {} bytes", input.signature.len())?;
            }
        }
        
        writeln!(f, "📤 输出:")?;
        for (i, output) in self.vout.iter().enumerate() {
            writeln!(f, "  {}. 💎 金额: {} 币", i + 1, output.value)?;
            let address = self.pub_key_hash_to_address(&output.pub_key_hash);
            writeln!(f, "     🏠 接收地址: {}", address)?;
        }
        
        Ok(())
    }
}
