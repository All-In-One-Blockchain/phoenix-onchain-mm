use std::path::PathBuf;
use structopt::StructOpt;

use crate::config::Config as PhoenixConfig;
#[derive(Debug, StructOpt)]
#[structopt(name = "phoneix-mm-cli")]
pub struct PhoneixOnChainMMCli {
    config_path: Option<PathBuf>,
}

impl PhoneixOnChainMMCli {
    pub fn get_config_path(&self) -> anyhow::Result<PathBuf> {
        if let Some(config_path) = self.config_path.clone() {
            let config_str = std::fs::read_to_string(config_path.clone())?;
            toml::from_str::<PhoenixConfig>(&config_str)?;
            Ok(config_path)
        } else {
            // open  config file path is  ~/.config/pomm/config.toml
            let home_path = dirs::home_dir().ok_or(anyhow::anyhow!("can't open home dir"))?;
            let pomm_config_path = home_path.join(".config").join("pomm");
            let config_path = pomm_config_path.join("config.toml");
            if std::fs::read_to_string(config_path.clone()).is_ok() {
                Ok(config_path)
            } else {
                std::fs::create_dir_all(pomm_config_path.clone())?;
                let config_path = pomm_config_path.join("config.toml");
                std::fs::write(
                    config_path.clone(),
                    r#"
# Optionally include your keypair path. Defaults to your Solana CLI config file.
keypair_path = "/home/davirain/.config/solana/id.json"
# Optionally include your RPC endpoint. Use "local", "dev", "main" for default endpoints. Defaults to your Solana CLI config file.
rpc_endpoint = "https://api.devnet.solana.com"
# Optionally include a commitment level. Defaults to your Solana CLI config file.
commitment = "confirmed"

[phoenix]
market = "78ehDnHgbkFxqXZwdFxa8HK7saX58GymeX2wNGdkqYLp"
ticker = "SOL-USD"
quote_refresh_frequency_in_ms = 2000
quote_edge_in_bps = 3
quote_size = 100000000
price_improvement_behavior = "ignore"
post_only = true"#,
                )?;
                let config_str = std::fs::read_to_string(config_path.clone())?;
                toml::from_str::<PhoenixConfig>(&config_str)?;
                Ok(config_path)
            }
        }
    }
}
