use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;


/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {}


/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    Mint { to: String },

    Burn { from: String },
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetPoolDataResponse)]
    GetPoolData {},
}
#[cw_serde]
pub struct GetPoolDataResponse {
    pub token0: String,
    pub token1: String,
    pub reserve0: Uint128,
    pub reserve1: Uint128,
}

#[cw_serde]
pub struct PoolDataResponse {
    pub amount0: String,
    pub amount1: String
}