# Pomm Client

## Pomm usage

```bash
pomm 0.1.1

USAGE:
    pomm <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    airdrop                  airdrop base and quote token
    auto                     auto generate config.toml file to ~/.config/pomm/config.toml
    cancle                   cancle all orders
    fetch-market-event       fetch market event
    get-market-address       get market address
    get-price                get base and quote price
    grpc                     grpc
    help                     Prints this message or the help of the given subcommand(s)
    init                     initialize Phoenix onchain Maket Maker and Claim Market Sate
    list-all-market          list all market
    listen-balance           listen balance
    update-quotes            update quotes
    view-state-order-book    view state order book
```

## Config setting

```toml
# Optionally include your keypair path. Defaults to your Solana CLI config file.
keypair_path = "/Users/davirain/.config/solana/id.json"
# Optionally include your RPC endpoint. Use "local", "dev", "main" for default endpoints. Defaults to your Solana CLI config file.
rpc_endpoint = "https://api.devnet.solana.com"
# Optionally include a commitment level. Defaults to your Solana CLI config file.
commitment = "confirmed"

[phoenix]
# this is phoneix market address
market = "CS2H8nbAVVEUHWPF5extCSymqheQdkd4d7thik6eet9N"
# devnet: https://pyth.network/developers/price-feed-ids#solana-devnet
# mainnet: https://pyth.network/developers/price-feed-ids#solana-mainnet-beta
# Crypto.SOL/USD J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix
# base account is oracle account in pyth
base_account = "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix"
# devnet: https://pyth.network/developers/price-feed-ids#solana-devnet
# mainnet: https://pyth.network/developers/price-feed-ids#solana-mainnet-beta
# Crypto.USDC/USD 5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7
# quote account is oracle account in pyth
quote_account = "5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7"
ticker = "SOL-USDC"
quote_refresh_frequency_in_ms = 2000
quote_edge_in_bps = 3
quote_size = 100000000
price_improvement_behavior = "ignore"
post_only = true
```
