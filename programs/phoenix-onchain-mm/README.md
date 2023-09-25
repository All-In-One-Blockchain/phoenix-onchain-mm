# Phoenix On-chain Market Maker

This is a reference implementation of a smart contract for an on-chain market making bot.

It stores a quote width in basis points and a quote size and uses this information to update its current quotes.

The goal is to be able to write a client that looks like this:

```python
while True:
  price = await get_fair_price_by_oracle(TOKEN)
  await market_maker.update_orders(price)
  time.sleep(SLEEP_DURATION)
```

## env requirement

- anchor-cli 0.26.0
    - use avm install 0.26.0
- solana-cli 1.14.14
    - use solana-install init 1.14.14
