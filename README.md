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
    pomm [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config-path <config-path>    config path for Phoenix onchain Maket Maker
```
