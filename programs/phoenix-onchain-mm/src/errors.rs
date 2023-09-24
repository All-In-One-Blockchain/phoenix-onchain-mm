use anchor_lang::prelude::*;

// An enum for custom error codes
#[error_code]
pub enum StrategyError {
    NoReturnData,
    InvalidStrategyParams,
    EdgeMustBeNonZero,
    InvalidPhoenixProgram,
    FailedToDeserializePhoenixMarket,
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
    #[msg("The config has already been initialized.")]
    ReInitialize,
    #[msg("The config has not been initialized.")]
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
    PythValidSlot,
    PythStatus,
    PythNegativePrice,
    PythConfidence,
}
