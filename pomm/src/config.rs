use serde::Deserialize;

/// This is what we're going to decode into. Each field is optional, meaning
/// that it doesn't have to be present in TOML.
#[derive(Debug, Deserialize)]
struct Config {
    /// Optionally include your keypair path. Defaults to your Solana CLI config file.
    keypair_path: String,
    /// Optionally include your RPC endpoint. Use "local", "dev", "main" for default endpoints. Defaults to your Solana CLI config file.
    rpc_endpoint: String,
    /// Optionally include a commitment level. Defaults to your Solana CLI config file.
    commitment: String,
    phoenix: PhoenixOnChainMMConfig,
}

/// Sub-structs are decoded from tables, so this will decode from the `[server]`
/// table.
///
/// Again, each field is optional, meaning they don't have to be present.
#[derive(Debug, Deserialize)]
struct PhoenixOnChainMMConfig {
    /// Market pubkey to provide on
    market: String,
    // The ticker is used to pull the price from the Coinbase API, and therefore should conform to the Coinbase ticker format.
    /// Note that for all USDC quoted markets, the price feed should use "USD" instead of "USDC".
    ticker: String,
    quote_refresh_frequency_in_ms: u64,
    quote_edge_in_bps: u64,
    quote_size: u64,
    price_improvement_behavior: String,
    post_only: bool,
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
