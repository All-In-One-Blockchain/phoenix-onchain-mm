use serde::{Deserialize, Deserializer};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// This is what we're going to decode into. Each field is optional, meaning
/// that it doesn't have to be present in TOML.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Optionally include your keypair path. Defaults to your Solana CLI config file.
    pub keypair_path: Option<String>,
    /// Optionally include your RPC endpoint. Use "local", "dev", "main" for default endpoints. Defaults to your Solana CLI config file.
    pub rpc_endpoint: Option<String>,
    /// Optionally include a commitment level. Defaults to your Solana CLI config file.
    pub commitment: Option<String>,
    pub phoenix: PhoenixOnChainMMConfig,
}

fn parse_pubkey<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    let pubkey_str = String::deserialize(deserializer)?;
    Pubkey::from_str(&pubkey_str).map_err(serde::de::Error::custom)
}

/// Sub-structs are decoded from tables, so this will decode from the `[server]`
/// table.
///
/// Again, each field is optional, meaning they don't have to be present.
#[derive(Debug, Deserialize)]
pub struct PhoenixOnChainMMConfig {
    /// Market pubkey to provide on
    #[serde(deserialize_with = "parse_pubkey")]
    pub market: Pubkey,
    /// The ticker is used to pull the price from the Coinbase API, and therefore should conform to the Coinbase ticker format.
    /// Note that for all USDC quoted markets, the price feed should use "USD" instead of "USDC".
    pub ticker: String,
    pub quote_refresh_frequency_in_ms: u64,
    pub quote_edge_in_bps: u64,
    pub quote_size: u64,
    pub price_improvement_behavior: String,
    pub post_only: bool,
}

#[test]
fn test_read_config() {
    // 读取配置文件
    let config_str =
        std::fs::read_to_string("/Users/davirain/solana/hackhouse/phoenix-onchain-mm/config.toml")
            .unwrap();
    // 解析配置文件
    let config: Config = toml::from_str(&config_str).unwrap();

    println!("{:#?}", config);
}
