use crate::config::Config as PhoenixConfig;
use crate::utils::get_pomm_config;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cancle {}

impl Cancle {
    pub async fn run(&self) -> anyhow::Result<()> {
        let phoneix_config = get_pomm_config()?;

        let (_, payer, rpc_enpoint) = phoneix_config.read_global_config()?;

        let sdk = phoenix_sdk::sdk_client::SDKClient::new(&payer, &rpc_enpoint).await?;

        let (cancel_order_tx_sig, event) = sdk
            .send_cancel_all(&phoneix_config.phoenix.market)
            .await
            .ok_or(anyhow::anyhow!("cancel tx returen empty"))?;

        println!(
            "canceling all orders tx: https://explorer.solana.com/{}?cluster=devnet",
            cancel_order_tx_sig
        );
        println!("cancel event: {:?}", event);
        Ok(())
    }
}
