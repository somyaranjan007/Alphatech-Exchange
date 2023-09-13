use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};
use serde::{Serialize, Deserialize};

// VAULT_OWNER is used to store the address of the vault owner in the state.
pub const VAULT_OWNER: Item<String> = Item::new("vault_owner");

/**
 * FACTORY_REGISTER: This constant represents a mapping used to track the registration
 * status of factory contracts in the system. Each key in the mapping corresponds to a
 * factory address, and the associated boolean value indicates whether the factory is
 * registered (true) or not registered (false).
 *
 * Example:
 * - Key: FactoryContractAddress1, Value: true (Registered)
 * - Key: FactoryContractAddress2, Value: false (Not Registered)
 * - Key: FactoryContractAddress3, Value: true (Registered)
 *
 * This mapping allows efficient look-up to determine whether a specific factory contract
 * has been registered or not.
 */

pub const FACTORY_REGISTER: Map<String, bool> = Map::new("factory_register");

/**
 * `PoolData` is a struct used to store information about a registered pool contract in the vault.
 * - `registered`: when true (Registered) and when false (Not Registered)
 * - `token0`: The address or identifier of the first token in the pool's token pair.
 * - `token1`: The address or identifier of the second token in the pool's token pair.
 * - `reserve0`: The current reserve amount of `token0` held in the pool.
 * - `reserve1`: The current reserve amount of `token1` held in the pool.
 * - `lp_token_contract`: The address of the CW20 contract responsible for minting LP (Liquidity Provider) tokens
 *    when users provide liquidity to the pool.
 */
#[derive(Serialize, Deserialize)]
pub struct PoolData {
    pub registered: bool,
    pub token0: String,
    pub token1: String,
    pub reserve0: Uint128,
    pub reserve1: Uint128,
    pub lp_token_contract: String,
}

/**
 * `POOL_REGISTER` is a mapping used to store information about registered pool contracts in the vault.
 *
 * When a pool contract is registered in the vault contract, all the relevant data is saved in this mapping.
 * The key is the pool contract's address, and the associated value is an instance of the `PoolData` struct,
 * containing details about the pool's token pair, reserves, and LP token contract.
 */
pub const POOL_REGISTER: Map<String, PoolData> = Map::new("pool_register");
