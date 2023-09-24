use anyhow::anyhow;
use serde::{Deserialize, Deserializer};
use solana_cli_config::{Config as SolanaConfig, ConfigInput, CONFIG_FILE};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::signature::Keypair;
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

impl Config {
    pub fn read_global_config(&self) -> anyhow::Result<(CommitmentConfig, Keypair, String)> {
        let (commitment, keypair_path, rpc_enpoint) =
            if let (Some(commitment), Some(keypair_path), Some(rpc_endpoint)) = (
                self.commitment.clone(),
                self.keypair_path.clone(),
                self.rpc_endpoint.clone(),
            ) {
                (commitment, keypair_path, rpc_endpoint)
            } else {
                let config = match CONFIG_FILE.as_ref() {
                    Some(config_file) => SolanaConfig::load(config_file).unwrap_or_else(|_| {
                        println!("Failed to load config file: {}", config_file);
                        SolanaConfig::default()
                    }),
                    None => SolanaConfig::default(),
                };
                (config.commitment, config.keypair_path, config.json_rpc_url)
            };
        let commitment = ConfigInput::compute_commitment_config("", &commitment).1;
        let payer = get_payer_keypair_from_path(&keypair_path)?;
        let network_url = get_network(&rpc_enpoint).to_string();
        Ok((commitment, payer, network_url))
    }
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

pub fn get_network(network_str: &str) -> &str {
    match network_str {
        "devnet" | "dev" | "d" => "https://api.devnet.solana.com",
        "mainnet" | "main" | "m" | "mainnet-beta" => "https://api.mainnet-beta.solana.com",
        "localnet" | "localhost" | "l" | "local" => "http://localhost:8899",
        _ => network_str,
    }
}

pub fn get_payer_keypair_from_path(path: &str) -> anyhow::Result<Keypair> {
    read_keypair_file(&*shellexpand::tilde(path)).map_err(|e| anyhow!(e.to_string()))
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