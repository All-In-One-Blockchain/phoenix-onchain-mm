use crate::config::PhoenixOnChainMMConfig;
use crate::utils::get_pomm_config;
use ellipsis_client::EllipsisClient;
use phoenix_sdk::sdk_client::SDKClient;
use solana_client::nonblocking::rpc_client::RpcClient;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ViewStateOrderBook {
    #[structopt(long, default_value = "5")]
    pub levels: usize,
    #[structopt(long, default_value = "4")]
    pub precision: usize,
}

impl ViewStateOrderBook {
    pub async fn run(&self) -> anyhow::Result<()> {
        let phoneix_config = get_pomm_config()?;

        let (commitment, payer, rpc_enpoint) = phoneix_config.read_global_config()?;

        let PhoenixOnChainMMConfig { market, .. } = phoneix_config.phoenix;

        let client = EllipsisClient::from_rpc(
            RpcClient::new_with_commitment(rpc_enpoint, commitment),
            &payer,
        )?;
        let sdk_client = SDKClient::new_from_ellipsis_client(client).await?;

        let orderbook = sdk_client.get_market_orderbook(&market).await?;
        orderbook.print_ladder(self.levels, self.precision);

        Ok(())
    }
}
