use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint128};
use cw20::{AllowanceResponse, BalanceResponse, TokenInfoResponse};
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

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
    Burn {
        vault_address: String,
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
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Cw20ReceiveMsg {
    pub sender: String,
    pub amount: Uint128,
    pub msg: Binary,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct RemoveLiquidityParams {
    pub token_a: String,
    pub token_b: String,
    pub reserve_a: Uint128,
    pub reserve_b: Uint128,
    pub amount_a: Uint128,
    pub amount_b: Uint128,
    pub address_to: String,
}

#[cw_serde] 
pub struct RemoveLiquidityPoolParams {
    pub vault_contract_addresss: String,
    pub amount_a_min: Uint128,
    pub amount_b_min: Uint128,
    pub address_to: String
}


impl fmt::Display for Cw20ReceiveMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "sender:{} amount:{} msg:{}",
            self.sender,
            self.amount,
            self.msg.to_string()
        )
    }
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

    #[returns(AllowanceResponse)]
    Allowance { owner: String, spender: String },

    #[returns(Uint128)]
    GetAmountOut(AmountOutParams),

    #[returns(Uint128)]
    GetAmountIn(AmountInParams),

    #[returns(GetAmountTokenTransfer)]
    GetAmountTransferToken { vault_address: String },
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
    pub amount_a: Uint128,
    pub amount_b: Uint128,
}
