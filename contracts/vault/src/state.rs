use cw_storage_plus::{Item, Map};
use packages::vault_msg::PoolDataResponse;

// VAULT_OWNER is used to store the address of the vault owner in the state.
pub const VAULT_OWNER: Item<String> = Item::new("vault_owner");

/**
 * FACTORY_REGISTER: This constant represents a mapping used to track the registration
 * status of factory contracts in the system. Each key in the mapping corresponds to a
 * factory address, and the associated boolean value indicates whether the factory is
 * registered (true) or not registered (false).
 *
 * Example:
 * - Key: FactoryContractAddress1, Value: true (Registered2222222222222222222222)
 * - Key: FactoryContractAddress2, Value: false (Not Registered)
 * - Key: FactoryContractAddress3, Value: true (Registered)
 *
 * This mapping allows efficient look-up to determine whether a specific factory contract
 * has been registered or not.
 */

pub const FACTORY_REGISTER: Map<String, bool> = Map::new("factory_register");

/**
 * `POOL_REGISTER` is a mapping used to store information about registered pool contracts in the vault.
 *
 * When a pool contract is registered in the vault contract, all the relevant data is saved in this mapping.
 * The key is the pool contract's address, and the associated value is an instance of the `PoolData` struct,
 * containing details about the pool's token pair, reserves, and LP token contract.
 * 
 * note: we are sending pool addresses in the events so that you can find addressess and store them for fetching pool data
 */
pub const POOL_REGISTER: Map<String, PoolDataResponse> = Map::new("pool_register");