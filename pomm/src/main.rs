#![allow(unused_imports)]

use structopt::StructOpt;

pub mod command;
pub mod config;
pub mod constant;
pub mod utils;

use command::PhoneixOnChainMMCli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _opt = PhoneixOnChainMMCli::from_args();
    Ok(())
}
