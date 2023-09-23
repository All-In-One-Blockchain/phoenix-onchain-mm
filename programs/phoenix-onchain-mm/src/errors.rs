use anchor_lang::prelude::*;

// An enum for custom error codes
#[error_code]
pub enum StrategyError {
    NoReturnData,
    InvalidStrategyParams,
    EdgeMustBeNonZero,
    InvalidPhoenixProgram,
    FailedToDeserializePhoenixMarket,
}
