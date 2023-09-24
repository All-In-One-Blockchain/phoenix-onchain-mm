use crate::config::Config as PhoenixConfig;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cancle {}

impl Cancle {
    pub async fn run(&self) -> anyhow::Result<()> {
        let home_path = dirs::home_dir().ok_or(anyhow::anyhow!("can't open home dir"))?;
        let pomm_config_path = home_path.join(".config").join("pomm");
        let config_path = pomm_config_path.join("config.toml");

        // 读取配置文件
        let config_str = std::fs::read_to_string(config_path).unwrap();
        // 解析配置文件
        let phoneix_config: PhoenixConfig = toml::from_str(&config_str).unwrap();

        let (_, payer, rpc_enpoint) = phoneix_config.read_global_config()?;

        let sdk = phoenix_sdk::sdk_client::SDKClient::new(&payer, &rpc_enpoint).await?;

        let (cancel_order_tx_sig, event) = sdk
            .send_cancel_all(&phoneix_config.phoenix.market)
            .await
            .ok_or(anyhow::anyhow!("cancel tx returen empty"))?;

        println!(
            "canceling all orders tx:  https://beta.solscan.io/tx/{}?cluster=devnet",
            cancel_order_tx_sig
        );
        println!("cancel event: {:?}", event);
        Ok(())
    }
}
