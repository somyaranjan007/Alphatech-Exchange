#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

const CONTRACT_NAME: &str = "crates.io:testing";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    todo!()
}

#[cfg(test)]
mod vault_swap_tests {
    use cosmwasm_std::{Addr, Empty, Uint1280};
    use cw_multi_test::{App,ContractWrapper, Executor};

    #[test]
    fn execute_vault_swap_test(){
        let vault_owner=Addr::unchecked("vault_owner");
        let usdc_owner=Addr::unchecked("usdc_owner");
        let usdc_minter=Addr::unchecked("usdc_minter");
        let usdt_owner=Addr::unchecked("usdt_owner");
        let usdt_minter=Addr::unchecked("usdt_minter");
        let factory_owner=Addr::unchecked("factory_owner");

        let liquidity_provider=Addr::unchecked("liquidity_provider");


        //first initialize the app
        let mut app=App::default();

        //create a wrapper for for vault contract
        let code = ContractWrapper::new(vault::contract::execute,vault::contract::instantiate,vault::contract::query);

        //store this code
        let code_id=app.store_code(Box::new(code));

        //initialize vault contract to get vault_contract_address
        let vault_contract_address=app.instantiate_contract(code_id,vault_owner.clone(),&Empty{}, &[], "vault contract", None).unwrap();
    }




}