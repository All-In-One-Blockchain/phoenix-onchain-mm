use crate::config::PhoenixOnChainMMConfig;
use crate::utils::get_pomm_config;
use pyth_sdk_solana::load_price_feed_from_account;
use solana_client::nonblocking::rpc_client::RpcClient;
// use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct GetPrice {}

impl GetPrice {
    pub async fn run(&self) -> anyhow::Result<()> {
        let phoneix_config = get_pomm_config()?;

        let (commitment, payer, rpc_enpoint) = phoneix_config.read_global_config()?;

        let client = RpcClient::new_with_commitment(rpc_enpoint.to_string(), commitment);

        let _sdk = phoenix_sdk::sdk_client::SDKClient::new(&payer, &rpc_enpoint).await?;

        let PhoenixOnChainMMConfig {
            base_account,
            quote_account,
            ..
        } = phoneix_config.phoenix;

        // get price data from key
        let mut base_price_account = client.get_account(&base_account).await?;
        let base_price_feed = load_price_feed_from_account(&base_account, &mut base_price_account)?;

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        let base_price = base_price_feed
            .get_price_no_older_than(current_time, 60)
            .ok_or(anyhow::anyhow!("base price is unavaiable"))?;

        let result = base_price.price as f64 * 10.0f64.powi(base_price.expo);

        println!(
            "Base price ........... {} x 10^{} = {}",
            base_price.price, base_price.expo, result
        );

        // get price data from key
        let mut quote_price_account = client.get_account(&quote_account).await?;
        let quote_price_feed =
            load_price_feed_from_account(&quote_account, &mut quote_price_account)?;

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        let quote_price = quote_price_feed
            .get_price_no_older_than(current_time, 60)
            .ok_or(anyhow::anyhow!("base price is unavaiable"))?;

        let result = quote_price.price as f64 * 10.0f64.powi(quote_price.expo);

        println!(
            "Quote price ........... {} x 10^{} = {}",
            quote_price.price, quote_price.expo, result
        );

        Ok(())
    }
}