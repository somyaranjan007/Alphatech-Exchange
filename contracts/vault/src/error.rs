use cosmwasm_std::StdError;
use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("pool doesn't existed")]
    PoolNotExisted {},

    #[error("Unable to fetch pool data")]
    FetchPoolDataError {},

    #[error("Insufficient amount")]
    InsufficientAmount {},

    #[error("Insufficient liquidity")]
    InsufficientLiquidity {},

    #[error("Calculation overflow")]
    CalculationOverflow {},

    #[error("Insufficient b amount")]
    InsufficientBAmount {},

    #[error("Insufficient a amount")]
    InsufficientAAmount {},

    #[error("Calculation amount error")]
    CalculationAmountError {},

    #[error("Adding liquidity failed")]
    AddingLiquidityFailed {},

    #[error("Unable to update liquidity")]
    UpateLiquidityFailed {},

    #[error("Overflow error")]
    OverflowError {},
    
    #[error("Unable to swap")]
    SwapFailed {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

impl Serialize for ContractError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str("ContractError")
    }
}
