use anchor_lang::prelude::*;

use super::StrategyParams;
use crate::errors::StrategyError;
use crate::phoenix_v1::load_header;
use crate::state::PhoenixStrategyState;

pub fn initialize_instruction(ctx: Context<Initialize>, params: StrategyParams) -> Result<()> {
    require!(
        params.quote_edge_in_bps.is_some()
            && params.quote_size_in_quote_atoms.is_some()
            && params.price_improvement_behavior.is_some(),
        StrategyError::InvalidStrategyParams
    );
    require!(
        params.quote_edge_in_bps.unwrap() > 0,
        StrategyError::EdgeMustBeNonZero
    );
    load_header(&ctx.accounts.market)?;
    let clock = Clock::get()?;
    msg!("Initializing Phoenix Strategy with params: {:?}", params);
    let mut phoenix_strategy = ctx.accounts.phoenix_strategy.load_init()?;
    *phoenix_strategy = PhoenixStrategyState {
        trader: *ctx.accounts.user.key,
        market: *ctx.accounts.market.key,
        bid_order_sequence_number: 0,
        bid_price_in_ticks: 0,
        initial_bid_size_in_base_lots: 0,
        ask_order_sequence_number: 0,
        ask_price_in_ticks: 0,
        initial_ask_size_in_base_lots: 0,
        last_update_slot: clock.slot,
        last_update_unix_timestamp: clock.unix_timestamp,
        quote_edge_in_bps: params.quote_edge_in_bps.unwrap(),
        quote_size_in_quote_atoms: params.quote_size_in_quote_atoms.unwrap(),
        post_only: params.post_only.unwrap_or(false),
        price_improvement_behavior: params.price_improvement_behavior.unwrap().to_u8(),
        padding: [0; 6],
    };
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds=[b"phoenix".as_ref(), user.key.as_ref(), market.key.as_ref()],
        bump,
        payer = user,
        space = 8 + std::mem::size_of::<PhoenixStrategyState>(),
    )]
    pub phoenix_strategy: AccountLoader<'info, PhoenixStrategyState>,
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: Checked in instruction
    pub market: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
