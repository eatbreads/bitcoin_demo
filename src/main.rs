mod block;
mod blockchain;
mod cli;


#[macro_use]
extern crate log;

//这个地方声明了Result，故mod里面可以直接使用
pub type Result<T> = std::result::Result<T, failure::Error>;

use blockchain::*;
use std::thread::sleep;
use std::time::Duration;
use crate::cli::Cli;
use env_logger::Env;


fn main() -> Result<()> {
    env_logger::from_env(Env::default().default_filter_or("warning")).init();

    let mut cli = Cli::new()?;
    cli.run()?;

    Ok(())
}
