#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, Uint128};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, AddLiquidityParams};
use crate::state::{PoolData, FACTORY_REGISTER, POOL_REGISTER, VAULT_OWNER};

const CONTRACT_NAME: &str = "crates.io:vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Save the vault owner's address in the VAULT_OWNER
    VAULT_OWNER.save(deps.storage, &info.sender.to_string())?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
//     match msg {}
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterFactory { factory_address } => {
            execute::execute_register_factory(_deps, _env, _info, factory_address)
        }
        ExecuteMsg::RegisterPool {
            pool_address,
            token0,
            token1,
            lp_token_contract,
        } => execute::execute_register_pool(
            _deps,
            _env,
            _info,
            pool_address,
            token0,
            token1,
            lp_token_contract,
        ),
        ExecuteMsg::AddLiquidity(add_liquidity_params) => execute::execute_add_liquidity(_deps, _env, _info, add_liquidity_params)
    }
}

pub mod execute {
    use super::*;

    pub fn execute_register_factory(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _factory: String,
    ) -> Result<Response, ContractError> {
        // Load the vault owner from storage
        let vault_owner = VAULT_OWNER.load(_deps.storage);

        match vault_owner {
            Ok(owner) => {
                // Check if the sender is authorized to register the factory
                if owner != _info.sender {
                    // Return an error if the sender is not authorized
                    return Err(ContractError::CustomError {
                        val: "Unauthorized Caller!".to_string(),
                    });
                } else {
                    // Save the factory registration status as true in storage
                    FACTORY_REGISTER.save(_deps.storage, _factory.clone(), &true)?;

                    // Return a successful response with attributes
                    Ok(Response::new()
                        .add_attribute("function", "value")
                        .add_attribute("factory_contract_address", _factory.to_string()))
                }
            }
            Err(_) => {
                // Return an error if the vault owner is not found in storage
                return Err(ContractError::CustomError {
                    val: "Unable to find vault owner".to_string(),
                });
            }
        }
    }

    pub fn execute_register_pool(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _pool: String,
        _token0: String,
        _token1: String,
        _lp_token_contract: String,
    ) -> Result<Response, ContractError> {
        // Check if the factory contract is registered
        let factory_registered = FACTORY_REGISTER.load(_deps.storage, _info.sender.to_string());

        match factory_registered {
            Ok(registered) => {
                // If factory contract is not registered, return an error
                if registered != true {
                    return Err(ContractError::CustomError {
                        val: "Unauthorized factory contract!".to_string(),
                    });
                } else {
                    // Create a new `PoolData` instance to store pool registration information
                    let pool_data = PoolData {
                        registered: true,
                        token0: _token0,
                        token1: _token1,
                        reserve0: Uint128::zero(),
                        reserve1: Uint128::zero(),
                        lp_token_contract: _lp_token_contract,
                    };

                    // Save the pool registration data in the `POOL_REGISTER` mapping
                    POOL_REGISTER.save(_deps.storage, _pool.clone(), &pool_data)?;

                    // Return a successful response with attributes
                    Ok(Response::new()
                        .add_attribute("function", "execute_register_pool")
                        .add_attribute("pool_contract_address", _pool))
                }
            }
            Err(_) => {
                return Err(ContractError::CustomError {
                    val: "Unable to find factory contract!".to_string(),
                })
            }
        }
    }

    pub fn execute_add_liquidity(_deps: DepsMut, _env: Env, _info: MessageInfo, _add_liquidity_params: AddLiquidityParams) -> Result<Response, ContractError> {
        todo!()
    }
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//     match msg {}
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    todo!()
}
