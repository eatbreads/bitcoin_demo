//! cli process

use super::*;
use crate::blockchain::*;
use crate::server::*;
use crate::transaction::*;
use crate::utxoset::*;
use crate::wallets::*;
use bitcoincash_addr::Address;
use clap::{App, Arg};
use std::process::exit;

pub struct Cli {}

impl Cli {
    pub fn new() -> Cli {
        Cli {}
    }

    pub fn run(&mut self) -> Result<()> {
        info!("run app");
        let matches = App::new("区块链比特币示例")
            .version("0.1")
            .author("小面包. 1852611363@qq.com")
            .about("用rust语言实现区块链比特币示例")
            .subcommand(App::new("printchain").about("打印整个区块链"))
            .subcommand(App::new("createwallet").about("创建一个新钱包"))
            .subcommand(App::new("listaddresses").about("列出所有钱包地址"))
            .subcommand(App::new("reindex").about("重建UTXO集合"))
            .subcommand(
                App::new("startnode")
                    .about("启动节点服务器")
                    .arg(Arg::from_usage("<port> '服务器本地绑定的端口'")),
            )
            .subcommand(
                App::new("startminer")
                    .about("启动挖矿节点服务器")
                    .arg(Arg::from_usage("<port> '服务器本地绑定的端口'"))
                    .arg(Arg::from_usage("<address> '挖矿奖励接收地址'")),
            )
            .subcommand(
                App::new("getbalance")
                    .about("获取地址余额")
                    .arg(Arg::from_usage(
                        "<address> '要查询余额的钱包地址'",
                    )),
            )
            .subcommand(App::new("createblockchain").about("创建新的区块链").arg(
                Arg::from_usage("<address> '创世区块奖励接收地址'"),
            ))
            .subcommand(
                App::new("send")
                    .about("发送交易")
                    .arg(Arg::from_usage("<from> '发送方钱包地址'"))
                    .arg(Arg::from_usage("<to> '接收方钱包地址'"))
                    .arg(Arg::from_usage("<amount> '发送金额'"))
                    .arg(Arg::from_usage(
                        "-m --mine '立即由发送方挖矿'",
                    )),
            )
            .get_matches();

        if let Some(ref matches) = matches.subcommand_matches("getbalance") {
            if let Some(address) = matches.value_of("address") {
                let balance = cmd_get_balance(address)?;
                println!("Balance: {}\n", balance);
            }
        } else if let Some(_) = matches.subcommand_matches("createwallet") {
            println!("address: {}", cmd_create_wallet()?);
        } else if let Some(_) = matches.subcommand_matches("printchain") {
            cmd_print_chain()?;
        } else if let Some(_) = matches.subcommand_matches("reindex") {
            let count = cmd_reindex()?;
            println!("Done! There are {} transactions in the UTXO set.", count);
        } else if let Some(_) = matches.subcommand_matches("listaddresses") {
            cmd_list_address()?;
        } else if let Some(ref matches) = matches.subcommand_matches("createblockchain") {
            if let Some(address) = matches.value_of("address") {
                cmd_create_blockchain(address)?;
            }
        } else if let Some(ref matches) = matches.subcommand_matches("send") {
            let from = if let Some(address) = matches.value_of("from") {
                address
            } else {
                println!("from not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            let to = if let Some(address) = matches.value_of("to") {
                address
            } else {
                println!("to not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            let amount: i32 = if let Some(amount) = matches.value_of("amount") {
                amount.parse()?
            } else {
                println!("amount in send not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            if matches.is_present("mine") {
                cmd_send(from, to, amount, true)?;
            } else {
                cmd_send(from, to, amount, false)?;
            }
        } else if let Some(ref matches) = matches.subcommand_matches("startnode") {
            if let Some(port) = matches.value_of("port") {
                println!("Start node...");
                let bc = Blockchain::new()?;
                let utxo_set = UTXOSet { blockchain: bc };
                let server = Server::new(port, "", utxo_set)?;
                server.start_server()?;
            }
        } else if let Some(ref matches) = matches.subcommand_matches("startminer") {
            let address = if let Some(address) = matches.value_of("address") {
                address
            } else {
                println!("address not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            let port = if let Some(port) = matches.value_of("port") {
                port
            } else {
                println!("port not supply!: usage\n{}", matches.usage());
                exit(1)
            };
            println!("Start miner node...");
            let bc = Blockchain::new()?;
            let utxo_set = UTXOSet { blockchain: bc };
            let server = Server::new(port, address, utxo_set)?;
            server.start_server()?;
        }

        Ok(())
    }
}

fn cmd_send(from: &str, to: &str, amount: i32, mine_now: bool) -> Result<()> {
    let bc = Blockchain::new()?;
    let mut utxo_set = UTXOSet { blockchain: bc };
    let wallets = Wallets::new()?;
    let wallet = wallets.get_wallet(from).unwrap();
    let tx = Transaction::new_UTXO(wallet, to, amount, &utxo_set)?;
    if mine_now {
        let cbtx = Transaction::new_coinbase(from.to_string(), String::from("reward!"))?;
        let new_block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;

        utxo_set.update(&new_block)?;
    } else {
        Server::send_transaction(&tx, utxo_set)?;
    }

    println!("success!");
    Ok(())
}

fn cmd_create_wallet() -> Result<String> {
    let mut ws = Wallets::new()?;
    let address = ws.create_wallet();
    ws.save_all()?;
    Ok(address)
}

fn cmd_reindex() -> Result<i32> {
    let bc = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };
    utxo_set.reindex()?;
    utxo_set.count_transactions()
}

fn cmd_create_blockchain(address: &str) -> Result<()> {
    let address = String::from(address);
    let bc = Blockchain::create_blockchain(address)?;

    let utxo_set = UTXOSet { blockchain: bc };
    utxo_set.reindex()?;
    println!("create blockchain");
    Ok(())
}

fn cmd_get_balance(address: &str) -> Result<i32> {
    let pub_key_hash = Address::decode(address).unwrap().body;
    let bc = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };
    let utxos = utxo_set.find_UTXO(&pub_key_hash)?;

    let mut balance = 0;
    for out in utxos.outputs {
        balance += out.value;
    }
    Ok(balance)
}

fn cmd_print_chain() -> Result<()> {
    let bc = Blockchain::new()?;
    for b in bc.iter() {
        println!("{:#?}", b);
    }
    Ok(())
}

fn cmd_list_address() -> Result<()> {
    let ws = Wallets::new()?;
    let addresses = ws.get_all_addresses();
    println!("addresses: ");
    for ad in addresses {
        println!("{}", ad);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_locally() {
        let addr1 = cmd_create_wallet().unwrap();
        let addr2 = cmd_create_wallet().unwrap();
        cmd_create_blockchain(&addr1).unwrap();

        let b1 = cmd_get_balance(&addr1).unwrap();
        let b2 = cmd_get_balance(&addr2).unwrap();
        assert_eq!(b1, 10);
        assert_eq!(b2, 0);

        cmd_send(&addr1, &addr2, 5, true).unwrap();

        let b1 = cmd_get_balance(&addr1).unwrap();
        let b2 = cmd_get_balance(&addr2).unwrap();
        assert_eq!(b1, 15);
        assert_eq!(b2, 5);

        cmd_send(&addr2, &addr1, 15, true).unwrap_err();
        let b1 = cmd_get_balance(&addr1).unwrap();
        let b2 = cmd_get_balance(&addr2).unwrap();
        assert_eq!(b1, 15);
        assert_eq!(b2, 5);
    }
}
