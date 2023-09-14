use cw_storage_plus::Item;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct FactoryData {
    pub vault_contract: String,
    pub pool_contract_code_id: u64,
    pub token0: Option<String>,
    pub token1: Option<String>
}

pub const FACTORY_DATA: Item<FactoryData> = Item::new("pool_contract_code_id");