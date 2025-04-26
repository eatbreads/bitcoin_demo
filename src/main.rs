mod block;
mod blockchain;
//这个地方声明了Result，故mod里面可以直接使用
pub type Result<T> = std::result::Result<T, failure::Error>;

use blockchain::*;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<()> {
    let mut bc = Blockchain::new();
    sleep(Duration::from_millis(10));
    bc.add_block(String::from("Send 1 BTC to Ivan"))?;
    sleep(Duration::from_millis(30));
    bc.add_block(String::from("Send 2 more BTC to Ivan"))?;

   // println!("Blockchain: {:#?}", bc);
    // 修改这行，从 {:#?} 改为 {}
    println!("{}", bc);
    Ok(())
}
