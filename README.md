# Phoenix On-chain Market Maker

This is a reference implementation of a smart contract for an on-chain market making bot.

It stores a quote width in basis points and a quote size and uses this information to update its current quotes.

The goal is to be able to write a client that looks like this:

```python
while True:
  price = await get_fair_price(TOKEN)
  await market_maker.update_orders(price)
  time.sleep(SLEEP_DURATION)
```

## Pomm usage

```
pomm 0.1.0

USAGE:
    pomm <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    auto             auto generate config.toml file to ~/.config/pomm/config.toml
    cancle           cancle all orders
    help             Prints this message or the help of the given subcommand(s)
    init             initialize Phoenix onchain Maket Maker and Claim Market Sate
    update-quotes    update quotes
```


## env requirement

- anchor-cli 0.26.0
    - use avm install 0.26.0
- solana-cli 1.14.14
    - use solana-install init 1.14.14


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
