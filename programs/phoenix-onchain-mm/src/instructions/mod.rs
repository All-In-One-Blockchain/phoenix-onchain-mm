use anchor_lang::prelude::*;

mod initialize;
mod update_quotes;

pub use initialize::*;
pub use update_quotes::*;

use crate::oracle::OracleConfig;

// TODO
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum PriceImprovementBehavior {
    Join,
    Dime,
    Ignore,
}

impl PriceImprovementBehavior {
    pub fn to_u8(&self) -> u8 {
        match self {
            PriceImprovementBehavior::Join => 0,
            PriceImprovementBehavior::Dime => 1,
            PriceImprovementBehavior::Ignore => 2,
        }
    }

    pub fn from_u8(byte: u8) -> Self {
        match byte {
            0 => PriceImprovementBehavior::Join,
            1 => PriceImprovementBehavior::Dime,
            2 => PriceImprovementBehavior::Ignore,
            _ => panic!("Invalid PriceImprovementBehavior"),
        }
    }
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct OrderParams {
    pub strategy_params: StrategyParams,
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct StrategyParams {
    pub quote_edge_in_bps: Option<u64>,
    pub quote_size_in_quote_atoms: Option<u64>,
    pub price_improvement_behavior: Option<PriceImprovementBehavior>,
    pub post_only: Option<bool>,
    pub oracle_account_config: OracleConfig,
}
