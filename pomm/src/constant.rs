pub const DEFAULT_CONFIG_FILE: &str = r#"
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
post_only = true"#;
