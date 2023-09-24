use crate::config::{Config as PhoenixConfig, PhoenixOnChainMMConfig};
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use phoenix::program::MarketHeader;
use phoenix_onchain_mm::accounts::Initialize as InitializeAccounts;
use phoenix_onchain_mm::instruction::Initialize as InitializeInstruction;
use phoenix_onchain_mm::PriceImprovementBehavior;
use phoenix_onchain_mm::StrategyParams;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ListenBalance {}

impl ListenBalance {
    pub async fn run(&self) -> anyhow::Result<()> {
        let home_path = dirs::home_dir().ok_or(anyhow::anyhow!("can't open home dir"))?;
        let pomm_config_path = home_path.join(".config").join("pomm");
        let config_path = pomm_config_path.join("config.toml");

        // 读取配置文件
        let config_str = std::fs::read_to_string(config_path).unwrap();
        // 解析配置文件
        let phoneix_config: PhoenixConfig = toml::from_str(&config_str).unwrap();

        let (commitment, payer, rpc_enpoint) = phoneix_config.read_global_config()?;

        let client = RpcClient::new_with_commitment(rpc_enpoint.to_string(), commitment);

        let _sdk = phoenix_sdk::sdk_client::SDKClient::new(&payer, &rpc_enpoint).await?;

        let data = client
            .get_account_data(&phoneix_config.phoenix.market)
            .await?;
        let header =
            bytemuck::try_from_bytes::<MarketHeader>(&data[..std::mem::size_of::<MarketHeader>()])
                .map_err(|_| anyhow::Error::msg("Failed to parse Phoenix market header"))?;

        let _base_decimals = u64::pow(10, header.base_params.decimals);
        let _quote_decimals = u64::pow(10, header.quote_params.decimals);

        let quote_token_account =
            get_associated_token_address(&payer.pubkey(), &header.quote_params.mint_key);
        let base_token_account =
            get_associated_token_address(&payer.pubkey(), &header.base_params.mint_key);

        let quote_start_balance = client
            .get_token_account_balance(&quote_token_account)
            .await?
            .ui_amount_string;

        let base_start_balance = client
            .get_token_account_balance(&base_token_account)
            .await?
            .ui_amount_string;

        println!(
            "Base Balance: {}, QuoteBalance: {}",
            base_start_balance, quote_start_balance
        );

        Ok(())
    }
}
