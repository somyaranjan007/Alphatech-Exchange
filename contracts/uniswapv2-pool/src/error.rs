use cosmwasm_std::StdError;
use thiserror::Error;
use serde::{Serialize, Serializer};

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Unable to Fetch Reserve")]
    FetchReserveFailed {},
    
    #[error("Unable to Fetch TotalSupply")]
    FetchTotalSupplyFailed {},

    #[error("Liquiity id not sufficient")]
    FetchLiquidityFailed {},
    
    #[error("Unable to Mint Lp-Tokens")]
    MintTokenFailed {},
    
    #[error("Liquiity id not sufficient")]
    InsufficientLiquidity {},

    #[error("Unable to Burn user Lp-Tokens")]
    BurnTokenFailed {},
    
    #[error("Insufficient amount entered")]
    InsufficientAmount {},
    

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}

impl Serialize for ContractError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer {
        serializer.serialize_str("ContractError")   
    }
}