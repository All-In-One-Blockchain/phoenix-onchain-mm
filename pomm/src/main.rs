use structopt::StructOpt;

pub mod command;
pub mod config;
pub mod constant;
pub mod errors;
pub mod ids;
pub mod utils;

use command::PhoneixOnChainMMCli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = PhoneixOnChainMMCli::from_args();
    opt.run().await?;
    Ok(())
}
