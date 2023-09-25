pub const DEFAULT_CONFIG_FILE: &str = r#"
# Optionally include your keypair path. Defaults to your Solana CLI config file.
keypair_path = "/Users/davirain/.config/solana/id.json"
# Optionally include your RPC endpoint. Use "local", "dev", "main" for default endpoints. Defaults to your Solana CLI config file.
rpc_endpoint = "https://api.devnet.solana.com"
# Optionally include a commitment level. Defaults to your Solana CLI config file.
commitment = "confirmed"

[phoenix]
market = "CS2H8nbAVVEUHWPF5extCSymqheQdkd4d7thik6eet9N"
# https://pyth.network/developers/price-feed-ids#solana-devnet
# Crypto.SOL/USD J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix
base_account = "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix"
# https://pyth.network/developers/price-feed-ids#solana-devnet
# Crypto.USDC/USD 5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7
quote_account = "5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7"
ticker = "SOL-USDC"
quote_refresh_frequency_in_ms = 2000
quote_edge_in_bps = 3
quote_size = 100000000
price_improvement_behavior = "ignore"
post_only = true"#;
