#![allow(non_snake_case)]
mod block;
mod blockchain;
mod transaction;
mod utxoset;
mod server;
mod wallets;
mod api;

#[macro_use]
extern crate log;

use actix_web::{web, App, HttpServer};
use env_logger::Env;

pub type Result<T> = std::result::Result<T, failure::Error>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 配置日志
    env_logger::from_env(Env::default().default_filter_or("warning")).init();
    
    println!("HTTP server starting at http://localhost:8080");

    // 启动HTTP服务器
    HttpServer::new(|| {
        App::new()
            .route("/wallet/create", web::post().to(api::create_wallet))
            .route("/wallet/balance/{address}", web::get().to(api::get_balance))
            .route("/transaction/send", web::post().to(api::send_transaction))
            .route("/blockchain/create/{address}", web::post().to(api::create_blockchain))
            .route("/blockchain/print", web::get().to(api::print_chain))
            .route("/utxo/reindex", web::post().to(api::reindex))
            .route("/wallet/list", web::get().to(api::list_addresses))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}