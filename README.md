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
