#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

pub mod constant;
pub mod errors;
pub mod instructions;
pub mod oracle;
pub mod phoenix_v1;
pub mod state;

pub use instructions::*;

declare_id!("Exz7z8HpBjS7trD6ZbdWABdQyhK5ZvGkuV4UYoUiSTQQ");

#[program]
pub mod phoenix_onchain_mm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: StrategyParams) -> Result<()> {
        initialize_instruction(ctx, params)
    }

    pub fn update_quotes(ctx: Context<UpdateQuotes>, params: OrderParams) -> Result<()> {
        update_quotes_instruction(ctx, params)
    }
}
