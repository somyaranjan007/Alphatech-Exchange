use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw20::{BalanceResponse,  TokenInfoResponse};
use cw_utils::Expiration;

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
    Mint(MintRecieveParams),
    Burn,
    BurnLpToken {
        amount: Uint128,
    },
    MintLpToken {
        recipient: String,
        amount: Uint128,
    },
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    Transfer {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
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

    #[returns(Uint128)]
    GetAmountOut(AmountOutParams),

    #[returns(Uint128)]
    GetAmountIn(AmountInParams),

    #[returns(GetAmountTokenTransfer)]
    GetAmountTransferToken,
}

#[cw_serde]
pub enum VaultMsgEnums {
    QueryPoolData { pool_address: String },
}

#[cw_serde]
pub struct PoolDataResponse {
    pub registered: bool,
    pub token0: String,
    pub token1: String,
    pub reserve0: Uint128,
    pub reserve1: Uint128,
}

#[cw_serde]
pub struct MintRecieveParams {
    pub to: String,
    pub amount0: Uint128,
    pub amount1: Uint128,
}

#[cw_serde]
pub struct AmountOutParams {
    pub amountIn: Uint128,
    pub reserveIn: Uint128,
    pub reserveOut: Uint128,
}
#[cw_serde]
pub struct AmountInParams {
    pub amountOut: Uint128,
    pub reserveIn: Uint128,
    pub reserveOut: Uint128,
}

#[cw_serde]
pub struct GetAmountTokenTransfer {
    pub amount0: Uint128,
    pub amount1: Uint128,
}
