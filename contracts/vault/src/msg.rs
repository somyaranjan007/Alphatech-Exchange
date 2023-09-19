use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::PoolData;

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    /**
     * 1. RegisterFactory: This function allows the owner of the vault contract to register a factory contract.
     * Only the owner of the vault contract can call this function.
     *
     * Parameters:
     * - `factory_address`: The address of the factory contract to be registered.
     */
    RegisterFactory {
        factory_address: String,
    },

    /**
     * 2. RegisterPool: This function is used to register a pool contract in the vault.
     *
     * Parameters are defined in RegisterPoolParams:
     * - `pool_address`: The address of the pool contract to be registered.
     * - `token0`: The address or identifier of the first token in the token pair managed by the pool.
     * - `token1`: The address or identifier of the second token in the token pair managed by the pool.
     * - `lp_token_contract`: The address of the CW20 contract responsible for minting LP tokens when users provide liquidity to the pool.
     *
     * In a liquidity pool, `token0` and `token1` represent a token pair that the pool manages. Users can provide liquidity in the form of both `token0` and `token1`, and in return, they receive LP (Liquidity Provider) tokens from the `lp_token_contract`.
     */
    RegisterPool(RegisterPoolParams),

    /**
     * 3. AddLiquidity: This function allows a user to add liquidity to a specific pool contract.
     *
     * Parameters are defined in AddLiquidityParams:
     * - `pool_address`: The address of the pool contract where liquidity will be added.
     * - `token_a`: The address or identifier of the first token to be contributed.
     * - `token_b`: The address or identifier of the second token to be contributed.
     * - `amount_a_desired`: The desired amount of `token_a` to be contributed by the user.
     * - `amount_b_desired`: The desired amount of `token_b` to be contributed by the user.
     * - `amount_a_min`: The minimum amount of `token_a` acceptable for the contribution.
     * - `amount_b_min`: The minimum amount of `token_b` acceptable for the contribution.
     * - `address_to`: The recipient's address for receiving LP tokens.
     * - `deadline`: The deadline by which the liquidity addition must occur.
     *
     * This function allows users to provide liquidity to a pool by specifying the tokens they want to
     * contribute, the desired amounts, and minimum acceptable amounts. It also specifies the recipient's
     * address for receiving LP (Liquidity Provider) tokens and a deadline for the operation.
     */
    AddLiquidity(AddLiquidityParams),

    /**
     * 3. RemoveLiquidity: This function allows a user to remove liquidity from a specific pool contract.
     *
     * Parameters are defined in RemoveLiquidityParams:
     * - `pool_address`: The address of the pool contract from which liquidity will be removed.
     * - `token_a`: The address or identifier of the first token in the liquidity pool.
     * - `token_b`: The address or identifier of the second token in the liquidity pool.
     * - `amount_a_min`: The minimum amount of `token_a` that the user is willing to receive.
     * - `amount_b_min`: The minimum amount of `token_b` that the user is willing to receive.
     * - `address_to`: The recipient's address for receiving the tokens withdrawn from the liquidity pool.
     * - `deadline`: The deadline by which the liquidity removal must occur.
     *
     * This function allows users to remove liquidity from a pool by specifying the pool address,
     * the tokens they want to withdraw, the minimum acceptable amounts of each token, the recipient's
     * address for receiving the tokens, and a deadline for the operation.
     */
    RemoveLiquidity(RemoveLiquidityParams),

    /**
     * 5. UpdateReserves: This function allows the owner to update the liquidity reserves of a specific pool contract.
     *
     * Parameters are defined in UpdateLiquidityParams:
     * - `pool_address`: The address of the pool contract to update reserves for.
     * - `amount_a`: The amount of the first token (token A) to add or remove from the liquidity reserves.
     * - `amount_b`: The amount of the second token (token B) to add or remove from the liquidity reserves.
     * - `feature`: An optional feature or description related to the liquidity update.
     *
     * This function is used by the owner to adjust the liquidity reserves of a pool contract, which can affect the pool's behavior and pricing.
     */
    UpdateReserves(UpdateLiquidiyParams),

    SwapTokens(SwapTokensParams),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct RegisterPoolParams {
    pub pool_address: String,
    pub token0: String,
    pub token1: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct AddLiquidityParams {
    pub pool_address: String,
    pub token_a: String,
    pub token_b: String,
    pub amount_a_desired: Uint128,
    pub amount_b_desired: Uint128,
    pub amount_a_min: Uint128,
    pub amount_b_min: Uint128,
    pub address_to: String,
    pub deadline: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct RemoveLiquidityParams {
    pub pool_address: String,
    pub token_a: String,
    pub token_b: String,
    pub liquidity: Uint128,
    pub amount_a_min: Uint128,
    pub amount_b_min: Uint128,
    pub address_to: String,
    pub deadline: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct UpdateLiquidiyParams {
    pub pool_address: String,
    pub amount_a: Uint128,
    pub amount_b: Uint128
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SwapTokensParams {
    pub pool_address: String,
    pub amount_in: Uint128,
    pub amount_out_min: Uint128,
    pub token_in: String,
    pub token_out: String,
    pub address_to: String,
}

#[cw_serde]
pub struct LiquidityAmounts {
    pub amount_a: Uint128,
    pub amount_b: Uint128,
}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // This example query variant indicates that any client can query the contract
    // using `YourQuery` and it will return `YourQueryResponse`
    // This `returns` information will be included in contract's schema
    // which is used for client code generation.
    //
    #[returns(PoolData)]
    QueryPoolData { pool_address: String },
}

#[cw_serde]
pub struct TransferFrom {
    pub owner: String,
    pub recipient: String,
    pub amount: Uint128,
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ContractMsg {
    pub contract_address: String,
    pub contract_msg: Binary,
}
