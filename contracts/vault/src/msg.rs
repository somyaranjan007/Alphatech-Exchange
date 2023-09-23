use cosmwasm_schema::cw_serde;

pub use packages::vault_msg::{VaultExecuteMsg as ExecuteMsg, VaultQueryMsg as QueryMsg};

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

// #[cw_serde]
// pub struct ExecutePoolReplyData {
//     pub pool_contract_address: String,
//     pub reserve_a: Uint128,
//     pub reserve_b: Uint128,
// }
