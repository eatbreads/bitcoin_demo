use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::blockchain::*;
use crate::transaction::*;
use crate::utxoset::*;
use crate::wallets::*;
use crate::server::Server;
use serde_json::json;
use bitcoincash_addr::Address;

// 请求和响应结构体定义
#[derive(Deserialize)]
pub struct SendRequest {
    pub from: String,
    pub to: String,
    pub amount: i32,
    pub mine_now: bool,
}

#[derive(Serialize)]
pub struct BalanceResponse {
    pub balance: i32,
}

// API处理函数
pub async fn create_wallet() -> impl Responder {
    let mut wallets = match Wallets::new() {
        Ok(w) => w,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };
    
    let address = wallets.create_wallet();
    if let Err(e) = wallets.save_all() {
        return HttpResponse::InternalServerError().body(e.to_string());
    }
    
    HttpResponse::Ok().json(json!({"address": address}))
}

pub async fn get_balance(address: web::Path<String>) -> impl Responder {
    let pub_key_hash = match Address::decode(&address).map_err(|e| e.0) {
        Ok(addr) => addr.body,
        Err(e) => return HttpResponse::BadRequest().body(format!("Invalid address format: {}", e))
    };

    let bc = match Blockchain::new() {
        Ok(bc) => bc,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    let utxo_set = UTXOSet { blockchain: bc };
    let utxos = match utxo_set.find_UTXO(&pub_key_hash) {
        Ok(utxos) => utxos,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    let balance = utxos.outputs.iter().fold(0, |acc, out| acc + out.value);
    HttpResponse::Ok().json(BalanceResponse { balance })
}

pub async fn send_transaction(req: web::Json<SendRequest>) -> impl Responder {
    let bc = match Blockchain::new() {
        Ok(bc) => bc,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    let mut utxo_set = UTXOSet { blockchain: bc };
    let wallets = match Wallets::new() {
        Ok(w) => w,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    let wallet = match wallets.get_wallet(&req.from) {
        Some(w) => w,
        None => return HttpResponse::BadRequest().body("Sender wallet not found")
    };

    let tx = match Transaction::new_UTXO(wallet, &req.to, req.amount, &utxo_set) {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    // 直接挖矿处理交易，不再使用Server发送
    let cbtx = match Transaction::new_coinbase(req.from.clone(), String::from("reward!")) {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    let new_block = match utxo_set.blockchain.mine_block(vec![cbtx, tx]) {
        Ok(block) => block,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    if let Err(e) = utxo_set.update(&new_block) {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    HttpResponse::Ok().json(json!({"status": "success"}))
}

pub async fn create_blockchain(address: web::Path<String>) -> impl Responder {
    let bc = match Blockchain::create_blockchain(address.to_string()) {
        Ok(bc) => bc,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    let utxo_set = UTXOSet { blockchain: bc };
    if let Err(e) = utxo_set.reindex() {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    HttpResponse::Ok().json(json!({"status": "success"}))
}

pub async fn print_chain() -> impl Responder {
    match Blockchain::new() {
        Ok(bc) => {
            let blocks: Vec<_> = bc.iter().collect();
            HttpResponse::Ok().json(json!({"blocks": blocks}))
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    }
}

pub async fn reindex() -> impl Responder {
    let bc = match Blockchain::new() {
        Ok(bc) => bc,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    let utxo_set = UTXOSet { blockchain: bc };
    if let Err(e) = utxo_set.reindex() {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    match utxo_set.count_transactions() {
        Ok(count) => HttpResponse::Ok().json(json!({"transaction_count": count})),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    }
}

pub async fn list_addresses() -> impl Responder {
    match Wallets::new() {
        Ok(wallets) => HttpResponse::Ok().json(json!({"addresses": wallets.get_all_addresses()})),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    }
}