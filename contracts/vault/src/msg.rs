use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint256;
use schemars::JsonSchema;
use serde::{ Serialize, Deserialize };

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
    RegisterFactory { factory_address: String },

    /**
     * 2. RegisterPool: This function is used to register a pool contract in the vault.
     *
     * Parameters:
     * - `pool_address`: The address of the pool contract to be registered.
     * - `token0`: The address or identifier of the first token in the token pair managed by the pool.
     * - `token1`: The address or identifier of the second token in the token pair managed by the pool.
     * - `lp_token_contract`: The address of the CW20 contract responsible for minting LP tokens when users provide liquidity to the pool.
     *
     * In a liquidity pool, `token0` and `token1` represent a token pair that the pool manages. Users can provide liquidity in the form of both `token0` and `token1`, and in return, they receive LP (Liquidity Provider) tokens from the `lp_token_contract`.
     */
    RegisterPool {
        pool_address: String,
        token0: String,
        token1: String,
        lp_token_contract: String,
    },

    /**
     * AddLiquidity: This function allows a user to add liquidity to a specific pool contract.
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
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct AddLiquidityParams {
    pub pool_address: String,
    pub token_a: String,
    pub token_b: String,
    pub amount_a_desired: Uint256,
    pub amount_b_desired: Uint256,
    pub amount_a_min: Uint256,
    pub amount_b_min: Uint256,
    pub address_to: String,
    pub deadline: Uint256,
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // This example query variant indicates that any client can query the contract
    // using `YourQuery` and it will return `YourQueryResponse`
    // This `returns` information will be included in contract's schema
    // which is used for client code generation.
    //
    // #[returns(YourQueryResponse)]
    // YourQuery {},
}

// We define a custom struct for each query response
// #[cw_serde]
// pub struct YourQueryResponse {}
