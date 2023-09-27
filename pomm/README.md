# Pomm Client

> Notice this Phoenix On-chain Market Maker only support Devnet

## Per

1. install [Rust](https://www.rust-lang.org/tools/install)
2. install [Solana-cli](https://docs.solana.com/cli/install-solana-cli-tools)
    - `sh -c "$(curl -sSfL https://release.solana.com/v1.14.14/install)"`
3. install pomm by Cargo
    - `cargo install pomm`
4. generate keypair
    - `solana-keygen new` this will generate keypair to `~/.config/solana/id.json`

## Usage Step

1. generate default config
    - `pomm auto` this will generate default config.toml file to `~/.config/pomm/config.toml`, you can modify it. to change market and base and quote coin account, for you interest market.
2. validate generate config
    - `pomm validate`
3. airdrop base and quote coin
    - `pomm airdrop`
4. init phoenix market
    - `pomm init`
5. update quotes
    - `pomm update-quotes`
6. listen balance
    - `pomm listen-balance`

Below this is pomm command support.

## Pomm usage

```bash
pomm 0.1.6

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
    validate                 validate config
    view-state-order-book    view state order book
```

## Config setting

```toml
# Optionally include your keypair path. Defaults to your Solana CLI config file.
keypair_path = "~/.config/solana/id.json"
# Optionally include your RPC endpoint. Use "local", "dev", "main" for default endpoints. Defaults to your Solana CLI config file.
rpc_endpoint = "https://api.devnet.solana.com"
# Optionally include a commitment level. Defaults to your Solana CLI config file.
commitment = "confirmed"

[phoenix]
# this is phoneix market address
market = "CS2H8nbAVVEUHWPF5extCSymqheQdkd4d7thik6eet9N" # you can change it to you interest market
ticker = "SOL/USDC" # you interest market
## Below is you Maket Maker Strategy Param
quote_refresh_frequency_in_ms = 2000
quote_edge_in_bps = 3
quote_size = 100000000
price_improvement_behavior = "ignore"
post_only = true
```
