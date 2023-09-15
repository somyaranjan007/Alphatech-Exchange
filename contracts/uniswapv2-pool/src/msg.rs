use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw20:: {TokenInfoResponse, BalanceResponse};


/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
     /// name of the derivative token
     pub name: String,
     /// symbol / ticker of the derivative token
     pub symbol: String,
     /// decimal places of the derivative token (for UI)
     pub decimals: u8,
}


/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    Mint (MintRecieveParams),
    Burn (BurnRecieveParams),
    GetAmountOut (AmountOutParams),
    GetAmountIn (AmountInParams)

}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(TokenInfoResponse)]
    TokenInfo {},

    #[returns(BalanceResponse)]
    Balance { address: String },
}

#[cw_serde]
pub struct PoolDataResponse {
    pub reserve0: String,
    pub reserve1: String,
}

#[cw_serde]
pub struct MintRecieveParams {
    pub to: String,
    pub amount0: Uint128,
    pub amount1: Uint128
}

#[cw_serde]
pub struct BurnRecieveParams {
    pub to: String,
    pub amount0: Uint128,
    pub amount1: Uint128
}

#[cw_serde]
pub struct AmountOutParams{
    pub amountIn: Uint128,
    pub reserveIn: Uint128,
    pub reserveOut: Uint128,
}
#[cw_serde]
pub struct AmountInParams{
    pub amountOut: Uint128,
    pub reserveIn: Uint128,
    pub reserveOut: Uint128,
}

