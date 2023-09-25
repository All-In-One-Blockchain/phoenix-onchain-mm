use crate::utils::get_pomm_config;
use phoenix::program::MarketHeader;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address;
use std::io;
use std::io::Write;
use structopt::StructOpt;
use tokio::time::sleep;
use tokio::time::Duration;

#[derive(Debug, StructOpt)]
pub struct ListenBalance {}

impl ListenBalance {
    pub async fn run(&self) -> anyhow::Result<()> {
        let phoneix_config = get_pomm_config()?;

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
            "Base Balance: {}, QuoteBalance: {}\n",
            base_start_balance, quote_start_balance
        );

        io::stdout().flush()?; // 刷新标准输出缓冲区

        loop {
            let quote_balance = client
                .get_token_account_balance(&quote_token_account)
                .await?
                .ui_amount_string;

            let base_balance = client
                .get_token_account_balance(&base_token_account)
                .await?
                .ui_amount_string;

            println!(
                "\tCurrent Base Balance: {}, Current Quote Balance: {}",
                base_balance, quote_balance
            );

            io::stdout().flush()?; // 刷新标准输出缓冲区

            // 休眠一段时间，以控制更新速度
            sleep(Duration::from_secs(1)).await;
        }
    }
}
