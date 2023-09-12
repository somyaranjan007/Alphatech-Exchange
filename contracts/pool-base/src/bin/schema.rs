use cosmwasm_schema::write_api;

use pool_base::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        migrate: MigrateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
