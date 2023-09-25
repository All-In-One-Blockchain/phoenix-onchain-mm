use anchor_lang::{prelude::*, solana_program::program::invoke};
use phoenix::program::{
    new_order::{CondensedOrder, MultipleOrderPacket},
    CancelMultipleOrdersByIdParams, CancelOrderParams, MarketHeader,
};
use phoenix::{
    quantities::WrapperU64,
    state::{markets::FIFOOrderId, OrderPacket, Side},
};

use super::{OrderParams, PriceImprovementBehavior};
use crate::constant::{BASE, BIG_NUMBER};
use crate::errors::StrategyError;
use crate::oracle::{OracleConfig, PriceFeed};
use crate::phoenix_v1::*;
use crate::state::PhoenixStrategyState;

pub fn update_quotes_instruction(ctx: Context<UpdateQuotes>, params: OrderParams) -> Result<()> {
    let UpdateQuotes {
        phoenix_strategy,
        user,
        phoenix_program,
        log_authority,
        market: market_account,
        seat,
        quote_account,
        base_account,
        quote_vault,
        base_vault,
        token_program,
        ..
    } = ctx.accounts;

    let mut phoenix_strategy = phoenix_strategy.load_mut()?;

    // Update timestamps
    let clock = Clock::get()?;
    phoenix_strategy.last_update_slot = clock.slot;
    phoenix_strategy.last_update_unix_timestamp = clock.unix_timestamp;

    // Update the strategy parameters
    if let Some(edge) = params.strategy_params.quote_edge_in_bps {
        if edge > 0 {
            phoenix_strategy.quote_edge_in_bps = edge;
        }
    }
    if let Some(size) = params.strategy_params.quote_size_in_quote_atoms {
        phoenix_strategy.quote_size_in_quote_atoms = size;
    }
    if let Some(post_only) = params.strategy_params.post_only {
        phoenix_strategy.post_only = post_only;
    }
    if let Some(price_improvement_behavior) = params.strategy_params.price_improvement_behavior {
        phoenix_strategy.price_improvement_behavior = price_improvement_behavior.to_u8();
    }

    // Load market
    let header = load_header(market_account)?;
    let market_data = market_account.data.borrow();
    let (_, market_bytes) = market_data.split_at(std::mem::size_of::<MarketHeader>());
    let market = phoenix::program::load_with_dispatch(&header.market_size_params, market_bytes)
        .map_err(|_| {
            msg!("Failed to deserialize market");
            StrategyError::FailedToDeserializePhoenixMarket
        })?
        .inner;

    msg!("Using oracle to calculate the fair price");

    let load_base_feed = &ctx.accounts.oracle_base_price;

    // With high confidence, the maximum value of the loan is
    // (price + conf) * loan_qty * 10 ^ (expo).
    // Here is more explanation on confidence interval in Pyth:
    // https://docs.pyth.network/consume-data/best-practices
    let current_timestamp1 = Clock::get()?.unix_timestamp;
    let base_oracle_price = load_base_feed
        .get_price_no_older_than(current_timestamp1, 60)
        .ok_or(StrategyError::PythOffline)?;
    msg!(
        "oracle price = {}, oracle expo = {}",
        base_oracle_price.price,
        base_oracle_price.expo
    );
    // calculating the price by multiplying oracle price on 10^6 and dividing it on 10^expo
    let base_fair_price = BIG_NUMBER * base_oracle_price.price as u128
        / (u64::pow(BASE, (-base_oracle_price.expo) as u32) as u128);

    let load_quote_feed = &ctx.accounts.oracle_quote_price;

    // With high confidence, the maximum value of the loan is
    // (price + conf) * loan_qty * 10 ^ (expo).
    // Here is more explanation on confidence interval in Pyth:
    // https://docs.pyth.network/consume-data/best-practices
    let current_timestamp1 = Clock::get()?.unix_timestamp;
    let quote_oracle_price = load_quote_feed
        .get_price_no_older_than(current_timestamp1, 60)
        .ok_or(StrategyError::PythOffline)?;

    msg!(
        "oracle price = {}, oracle expo = {}",
        quote_oracle_price.price,
        quote_oracle_price.expo
    );

    let quote_fair_price = BIG_NUMBER * quote_oracle_price.price as u128
        / (u64::pow(BASE, (-quote_oracle_price.expo) as u32) as u128);

    msg!(
        "Base price = {}, quote price = {}",
        base_fair_price,
        quote_fair_price
    );

    let fair_price_in_ticks = get_fair_price_in_ticks(base_fair_price, quote_fair_price, &header);

    // Compute quote prices
    let mut bid_price_in_ticks =
        get_bid_price_in_ticks(fair_price_in_ticks, phoenix_strategy.quote_edge_in_bps);

    let mut ask_price_in_ticks =
        get_ask_price_in_ticks(fair_price_in_ticks, phoenix_strategy.quote_edge_in_bps);

    // Returns the best bid and ask prices that are not placed by the trader
    let trader_index = market.get_trader_index(&user.key()).unwrap_or(u32::MAX) as u64;
    let (best_bid, best_ask) = get_best_bid_and_ask(market, trader_index);

    msg!("Current market: {} @ {}", best_bid, best_ask);

    let price_improvement_behavior =
        PriceImprovementBehavior::from_u8(phoenix_strategy.price_improvement_behavior);
    match price_improvement_behavior {
        PriceImprovementBehavior::Join => {
            // If price_improvement_behavior is set to Join, we will always join the best bid and ask
            // if our quote prices are within the spread
            ask_price_in_ticks = ask_price_in_ticks.max(best_ask);
            bid_price_in_ticks = bid_price_in_ticks.min(best_bid);
        }
        PriceImprovementBehavior::Dime => {
            // If price_improvement_behavior is set to Dime, we will never price improve by more than 1 tick
            ask_price_in_ticks = ask_price_in_ticks.max(best_ask - 1);
            bid_price_in_ticks = bid_price_in_ticks.min(best_bid + 1);
        }
        PriceImprovementBehavior::Ignore => {
            // If price_improvement_behavior is set to Ignore, we will not update our quotes based off the current
            // market prices
        }
    }

    // Compute quote amounts in base lots
    let size_in_quote_lots =
        phoenix_strategy.quote_size_in_quote_atoms / header.get_quote_lot_size().as_u64();

    let bid_size_in_base_lots = size_in_quote_lots * market.get_base_lots_per_base_unit().as_u64()
        / (bid_price_in_ticks * market.get_tick_size().as_u64());
    let ask_size_in_base_lots = size_in_quote_lots * market.get_base_lots_per_base_unit().as_u64()
        / (ask_price_in_ticks * market.get_tick_size().as_u64());

    msg!(
        "Our market: {} {} @ {} {}",
        bid_size_in_base_lots,
        bid_price_in_ticks,
        ask_price_in_ticks,
        ask_size_in_base_lots
    );

    let mut update_bid = true;
    let mut update_ask = true;
    let orders_to_cancel = [
        (
            Side::Bid,
            bid_price_in_ticks,
            FIFOOrderId::new_from_untyped(
                phoenix_strategy.bid_price_in_ticks,
                phoenix_strategy.bid_order_sequence_number,
            ),
            phoenix_strategy.initial_bid_size_in_base_lots,
        ),
        (
            Side::Ask,
            ask_price_in_ticks,
            FIFOOrderId::new_from_untyped(
                phoenix_strategy.ask_price_in_ticks,
                phoenix_strategy.ask_order_sequence_number,
            ),
            phoenix_strategy.initial_ask_size_in_base_lots,
        ),
    ]
    .iter()
    .filter_map(|(side, price, order_id, initial_size)| {
        if let Some(resting_order) = market.get_book(*side).get(order_id) {
            // The order is 100% identical, do not cancel it
            if resting_order.num_base_lots == *initial_size
                && order_id.price_in_ticks.as_u64() == *price
            {
                msg!("Resting order is identical: {:?}", order_id);
                match side {
                    Side::Bid => update_bid = false,
                    Side::Ask => update_ask = false,
                }
                return None;
            }
            msg!("Found partially filled resting order: {:?}", order_id);
            // The order has been partially filled or reduced
            return Some(*order_id);
        }
        msg!("Failed to find resting order: {:?}", order_id);
        // The order has been fully filled
        None
    })
    .collect::<Vec<FIFOOrderId>>();

    // Drop reference prior to invoking
    drop(market_data);

    // Cancel the old orders
    if !orders_to_cancel.is_empty() {
        invoke(
            &phoenix::program::create_cancel_multiple_orders_by_id_with_free_funds_instruction(
                &market_account.key(),
                &user.key(),
                &CancelMultipleOrdersByIdParams {
                    orders: orders_to_cancel
                        .iter()
                        .map(|o_id| CancelOrderParams {
                            order_sequence_number: o_id.order_sequence_number,
                            price_in_ticks: o_id.price_in_ticks.as_u64(),
                            side: Side::from_order_sequence_number(o_id.order_sequence_number),
                        })
                        .collect::<Vec<_>>(),
                },
            ),
            &[
                phoenix_program.to_account_info(),
                log_authority.to_account_info(),
                user.to_account_info(),
                market_account.to_account_info(),
            ],
        )?;
    }

    // Don't update quotes if the price is invalid or if the sizes are 0
    update_bid &= bid_price_in_ticks > 1 && bid_size_in_base_lots > 0;
    update_ask &= ask_price_in_ticks < u64::MAX && ask_size_in_base_lots > 0;

    let client_order_id = u128::from_le_bytes(user.key().to_bytes()[..16].try_into().unwrap());
    if !update_ask && !update_bid && orders_to_cancel.is_empty() {
        msg!("No orders to update");
        return Ok(());
    }
    let mut order_ids = vec![];
    if phoenix_strategy.post_only
        || !matches!(price_improvement_behavior, PriceImprovementBehavior::Join)
    {
        // Send multiple post-only orders in a single instruction
        let multiple_order_packet = MultipleOrderPacket::new(
            if update_bid {
                vec![CondensedOrder::new_default(
                    bid_price_in_ticks,
                    bid_size_in_base_lots,
                )]
            } else {
                vec![]
            },
            if update_ask {
                vec![CondensedOrder::new_default(
                    ask_price_in_ticks,
                    ask_size_in_base_lots,
                )]
            } else {
                vec![]
            },
            Some(client_order_id),
            false,
        );
        invoke(
            &phoenix::program::create_new_multiple_order_instruction_with_custom_token_accounts(
                &market_account.key(),
                &user.key(),
                &base_account.key(),
                &quote_account.key(),
                &header.base_params.mint_key,
                &header.quote_params.mint_key,
                &multiple_order_packet,
            ),
            &[
                phoenix_program.to_account_info(),
                log_authority.to_account_info(),
                user.to_account_info(),
                market_account.to_account_info(),
                seat.to_account_info(),
                quote_account.to_account_info(),
                base_account.to_account_info(),
                quote_vault.to_account_info(),
                base_vault.to_account_info(),
                token_program.to_account_info(),
            ],
        )?;
        parse_order_ids_from_return_data(&mut order_ids)?;
    } else {
        if update_bid {
            invoke(
                &phoenix::program::create_new_order_instruction_with_custom_token_accounts(
                    &market_account.key(),
                    &user.key(),
                    &base_account.key(),
                    &quote_account.key(),
                    &header.base_params.mint_key,
                    &header.quote_params.mint_key,
                    &OrderPacket::new_limit_order_default_with_client_order_id(
                        Side::Bid,
                        bid_price_in_ticks,
                        bid_size_in_base_lots,
                        client_order_id,
                    ),
                ),
                &[
                    phoenix_program.to_account_info(),
                    log_authority.to_account_info(),
                    user.to_account_info(),
                    market_account.to_account_info(),
                    seat.to_account_info(),
                    quote_account.to_account_info(),
                    base_account.to_account_info(),
                    quote_vault.to_account_info(),
                    base_vault.to_account_info(),
                    token_program.to_account_info(),
                ],
            )?;
            parse_order_ids_from_return_data(&mut order_ids)?;
        }
        if update_ask {
            invoke(
                &phoenix::program::create_new_order_instruction_with_custom_token_accounts(
                    &market_account.key(),
                    &user.key(),
                    &base_account.key(),
                    &quote_account.key(),
                    &header.base_params.mint_key,
                    &header.quote_params.mint_key,
                    &OrderPacket::new_limit_order_default_with_client_order_id(
                        Side::Ask,
                        ask_price_in_ticks,
                        ask_size_in_base_lots,
                        client_order_id,
                    ),
                ),
                &[
                    phoenix_program.to_account_info(),
                    log_authority.to_account_info(),
                    user.to_account_info(),
                    market_account.to_account_info(),
                    seat.to_account_info(),
                    quote_account.to_account_info(),
                    base_account.to_account_info(),
                    quote_vault.to_account_info(),
                    base_vault.to_account_info(),
                    token_program.to_account_info(),
                ],
            )?;
            parse_order_ids_from_return_data(&mut order_ids)?;
        }
    }

    let market_data = market_account.data.borrow();
    let (_, market_bytes) = market_data.split_at(std::mem::size_of::<MarketHeader>());
    let market = phoenix::program::load_with_dispatch(&header.market_size_params, market_bytes)
        .map_err(|_| {
            msg!("Failed to deserialize market");
            StrategyError::FailedToDeserializePhoenixMarket
        })?
        .inner;

    for order_id in order_ids.iter() {
        let side = Side::from_order_sequence_number(order_id.order_sequence_number);
        match side {
            Side::Ask => {
                market
                    .get_book(Side::Ask)
                    .get(order_id)
                    .map(|order| {
                        msg!("Placed Ask Order: {:?}", order_id);
                        phoenix_strategy.ask_price_in_ticks = order_id.price_in_ticks.as_u64();
                        phoenix_strategy.ask_order_sequence_number = order_id.order_sequence_number;
                        phoenix_strategy.initial_ask_size_in_base_lots =
                            order.num_base_lots.as_u64();
                    })
                    .unwrap_or_else(|| {
                        msg!("Ask order not found");
                    });
            }
            Side::Bid => {
                market
                    .get_book(Side::Bid)
                    .get(order_id)
                    .map(|order| {
                        msg!("Placed Bid Order: {:?}", order_id);
                        phoenix_strategy.bid_price_in_ticks = order_id.price_in_ticks.as_u64();
                        phoenix_strategy.bid_order_sequence_number = order_id.order_sequence_number;
                        phoenix_strategy.initial_bid_size_in_base_lots =
                            order.num_base_lots.as_u64();
                    })
                    .unwrap_or_else(|| {
                        msg!("Bid order not found");
                    });
            }
        }
    }

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateQuotes<'info> {
    #[account(
        mut,
        seeds=[b"phoenix".as_ref(), user.key.as_ref(), market.key.as_ref()],
        bump,
    )]
    pub phoenix_strategy: AccountLoader<'info, PhoenixStrategyState>,
    #[account(
            seeds = [b"oracle", user.key.as_ref(), market.key.as_ref()],
            bump
    )]
    pub oracle_account: Account<'info, OracleConfig>,
    #[account(
        address = oracle_account.oracle_base_account @ StrategyError::InvalidArgument
    )]
    pub oracle_base_price: Account<'info, PriceFeed>,
    #[account(
        address = oracle_account.oracle_quote_account @ StrategyError::InvalidArgument
    )]
    pub oracle_quote_price: Account<'info, PriceFeed>,
    pub user: Signer<'info>,
    pub phoenix_program: Program<'info, PhoenixV1>,
    /// CHECK: Checked in CPI
    pub log_authority: UncheckedAccount<'info>,
    /// CHECK: Checked in instruction and CPI
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// CHECK: Checked in CPI
    pub seat: UncheckedAccount<'info>,
    /// CHECK: Checked in CPI
    #[account(mut)]
    pub quote_account: UncheckedAccount<'info>,
    /// CHECK: Checked in CPI
    #[account(mut)]
    pub base_account: UncheckedAccount<'info>,
    /// CHECK: Checked in CPI
    #[account(mut)]
    pub quote_vault: UncheckedAccount<'info>,
    /// CHECK: Checked in CPI
    #[account(mut)]
    pub base_vault: UncheckedAccount<'info>,
    /// CHECK: Checked in CPI
    pub token_program: UncheckedAccount<'info>,
}
