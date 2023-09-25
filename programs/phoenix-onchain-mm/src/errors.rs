use anchor_lang::prelude::*;

// An enum for custom error codes
#[error_code]
pub enum StrategyError {
    #[msg("invalid strategy params")]
    InvalidStrategyParams,
    #[msg("edge must be non-zero")]
    EdgeMustBeNonZero,
    #[msg("invalid phoenix program")]
    InvalidPhoenixProgram,
    #[msg("failed to deserialize phoenix market")]
    FailedToDeserializePhoenixMarket,
    #[msg("Argument is invalid.")]
    InvalidArgument,
    #[msg("Pyth has an internal error.")]
    PythError,
    #[msg("Pyth price oracle is offline.")]
    PythOffline,
    #[msg("Program should not try to serialize a price account.")]
    TryToSerializePriceAccount,
}
