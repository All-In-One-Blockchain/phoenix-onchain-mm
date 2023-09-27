pub const PHOENIX_ONCHAIN_MM_STRATEGY_SEED: &[u8] = b"phoenix";
pub const PHOENIX_ONCHAIN_MM_ORACLE_SEED: &[u8] = b"oracle";
pub const BASE: f64 = 10.0;

pub const DEFAULT_CONFIG_FILE: &str = r#"
# Optionally include your keypair path. Defaults to your Solana CLI config file.
keypair_path = "~/.config/solana/id.json"
# Optionally include your RPC endpoint. Use "local", "dev", "main" for default endpoints. Defaults to your Solana CLI config file.
rpc_endpoint = "https://api.devnet.solana.com"
# Optionally include a commitment level. Defaults to your Solana CLI config file.
commitment = "confirmed"

[phoenix]
market = "CS2H8nbAVVEUHWPF5extCSymqheQdkd4d7thik6eet9N"
ticker = "SOL/USDC"
quote_refresh_frequency_in_ms = 2000
quote_edge_in_bps = 3
quote_size = 100000000
price_improvement_behavior = "ignore"
post_only = true"#;
