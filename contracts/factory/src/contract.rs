#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdResult, SubMsg,
    WasmMsg,
};
use cw0::parse_reply_instantiate_data;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, PoolInstantiateMsg, QueryMsg};
use crate::state::{FactoryData, FACTORY_DATA};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let factory_data = FactoryData {
        vault_contract: _msg.vault_contract,
        pool_contract_code_id: _msg.pool_contact_code_id,
        token0: None,
        token1: None,
    };

    FACTORY_DATA.save(deps.storage, &factory_data)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {}
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePool { token_a, token_b } => {
            execute::execute_create_pool(_deps, _env, _info, token_a, token_b)
        }
    }
}

pub mod execute {
    use super::*;

    pub fn execute_create_pool(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _token_a: String,
        _token_b: String,
    ) -> Result<Response, ContractError> {
        if _token_a == _token_b {
            return Err(ContractError::IdenticalAddresses {});
        }

        let (token0, token1) = (_token_a, _token_b);

        if token0 == String::from("") && token1 == String::from("") {
            return Err(ContractError::EmptyAddresses {});
        }

        let fetch_factory_data = FACTORY_DATA.load(_deps.storage);
        match fetch_factory_data {
            Ok(data) => {
                let update_factory = FACTORY_DATA.update(
                    _deps.storage,
                    |mut factory_data| -> StdResult<FactoryData> {
                        factory_data.token0 = Some(token0);
                        factory_data.token1 = Some(token1);
                        Ok(factory_data)
                    },
                );

                match update_factory {
                    Ok(_) => {
                        let pool_instantiate_tx = WasmMsg::Instantiate {
                            admin: None,
                            code_id: data.pool_contract_code_id,
                            msg: to_binary(&uniswapv2_pool::msg::InstantiateMsg {
                                name: String::from("pool_lp"),
                                symbol: String::from("POOL_LP"),
                                decimals: 18,
                            })?,
                            funds: vec![],
                            label: "pool_contract".to_string(),
                        };

                        const POOL_INSTANTIATE_TX_ID: u64 = 1u64;

                        let submessage =
                            SubMsg::reply_on_success(pool_instantiate_tx, POOL_INSTANTIATE_TX_ID);

                        Ok(Response::new()
                            .add_submessage(submessage)
                            .add_attribute("function", "execute_create_pool"))
                    }
                    Err(_) => return Err(ContractError::FactoryDataUpdateError {}),
                }
            }
            Err(_) => return Err(ContractError::FactoryDataFetchError {}),
        }
    }
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    const POOL_INSTANTIATE_TX_ID: u64 = 1u64;
    match _msg.id {
        POOL_INSTANTIATE_TX_ID => handle_pool_instantiate(_deps, _msg),
        _id => return Err(ContractError::ReplyIdError {}),
    }
}

pub fn handle_pool_instantiate(_deps: DepsMut, _msg: Reply) -> Result<Response, ContractError> {
    let res = parse_reply_instantiate_data(_msg);

    match res {
        Ok(data) => {
            let fetch_factory_data = FACTORY_DATA.load(_deps.storage);

            match fetch_factory_data {
                Ok(factory_data) => {
                    let register_pool_params = vault::msg::RegisterPoolParams {
                        pool_address: data.contract_address,
                        token0: match factory_data.token0 {
                            Some(data) => data,
                            None => return Err(ContractError::TokenNotFound {})
                        },
                        token1: match factory_data.token1 {
                            Some(data) => data,
                            None => return Err(ContractError::TokenNotFound {})
                        },
                    };

                    let vault_execute_tx = WasmMsg::Execute {
                        contract_addr: factory_data.vault_contract,
                        msg: to_binary(&vault::msg::ExecuteMsg::RegisterPool(
                            register_pool_params,
                        ))?,
                        funds: vec![],
                    };

                    Ok(Response::new().add_message(vault_execute_tx))
                }
                Err(_) => return Err(ContractError::FactoryDataFetchError {}),
            }
        }
        Err(_) => return Err(ContractError::ReplyDataError {}),
    }
}
