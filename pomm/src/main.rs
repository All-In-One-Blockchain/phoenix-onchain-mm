use solana_sdk::pubkey::Pubkey;
use structopt::StructOpt;

pub mod command;
pub mod config;
pub mod logic;

use command::PhoneixOnChainMMCli;

#[derive(Debug, Clone, Copy, Default)]
pub struct FaucetMetadata {
    pub spec_pubkey: Pubkey,
    pub faucet_pubkey: Pubkey,
    pub difficulty: u8,
    pub amount: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _opt = PhoneixOnChainMMCli::from_args();
    Ok(())
}
