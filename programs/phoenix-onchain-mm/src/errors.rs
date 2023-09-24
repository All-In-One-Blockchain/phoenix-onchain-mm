use anchor_lang::prelude::*;

// An enum for custom error codes
#[error_code]
pub enum StrategyError {
    #[msg("no return data")]
    NoReturnData,
    #[msg("invalid strategy params")]
    InvalidStrategyParams,
    #[msg("edge must be non-zero")]
    EdgeMustBeNonZero,
    #[msg("invalid phoenix program")]
    InvalidPhoenixProgram,
    #[msg("failed to deserialize phoenix market")]
    FailedToDeserializePhoenixMarket,
    #[msg("unauthorized")]
    Unauthorized,
    #[msg("re-initialize")]
    ReInitialize,
    #[msg("un-initialize")]
    UnInitialize,
    #[msg("Argument is invalid.")]
    InvalidArgument,
    #[msg("An overflow occurs.")]
    Overflow,
    #[msg("Pyth has an internal error.")]
    PythError,
    #[msg("Pyth price oracle is offline.")]
    PythOffline,
    #[msg("The loan value is higher than the collateral value.")]
    LoanValueTooHigh,
    #[msg("Program should not try to serialize a price account.")]
    TryToSerializePriceAccount,
    #[msg("pyth valid slot")]
    PythValidSlot,
    #[msg("pyth status")]
    PythStatus,
    #[msg("pyth negative price")]
    PythNegativePrice,
    #[msg("pyth confidence")]
    PythConfidence,
}
