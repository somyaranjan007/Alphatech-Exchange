#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, Reply, Response, StdError,
    StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;

use crate::error::ContractError;

use crate::msg::{
    AddLiquidityParams, ExecuteMsg, InstantiateMsg, LiquidityAmounts, QueryMsg, RegisterPoolParams,
    RemoveLiquidityParams, SwapTokensParams, TransferFrom, UpdateLiquidiyParams,
};

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
        ExecuteMsg::RegisterPool(register_pool_params) => {
            execute::execute_register_pool(_deps, _env, _info, register_pool_params)
        }
        ExecuteMsg::AddLiquidity(add_liquidity_params) => {
            execute::execute_add_liquidity(_deps, _env, _info, add_liquidity_params)
        }
        ExecuteMsg::RemoveLiquidity(remove_liquidity_params) => {
            execute::execute_remove_liquidity(_deps, _env, _info, remove_liquidity_params)
        }
        ExecuteMsg::UpdateReserves(update_liquidiy_params) => {
            execute::execute_update_liquidity(_deps, _env, _info, update_liquidiy_params)
        }
        ExecuteMsg::SwapTokens(swap_token_params) => {
            execute::execute_swap_tokens(_deps, _env, _info, swap_token_params)
        }
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
        _register_pool_params: RegisterPoolParams,
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
                        token0: _register_pool_params.token0,
                        token1: _register_pool_params.token1,
                        reserve0: Uint128::zero(),
                        reserve1: Uint128::zero(),
                    };

                    // Save the pool registration data in the `POOL_REGISTER` mapping
                    POOL_REGISTER.save(
                        _deps.storage,
                        _register_pool_params.pool_address.clone(),
                        &pool_data,
                    )?;

                    // Return a successful response with attributes
                    Ok(Response::new()
                        .add_attribute("function", "execute_register_pool")
                        .add_attribute("pool_contract_address", _register_pool_params.pool_address))
                }
            }
            Err(_) => {
                // Return an error if the factory contract is not found in storage
                return Err(ContractError::CustomError {
                    val: "Unable to find factory contract!".to_string(),
                });
            }
        }
    }

    fn calculate_amount(
        _amount_a: Uint128,
        _reserve_a: Uint128,
        _reserve_b: Uint128,
    ) -> Result<Uint128, ContractError> {
        if _amount_a > Uint128::zero() {
            return Err(ContractError::InsufficientAmount {});
        }

        match (_reserve_a, _reserve_b) {
            (_reserve_a, _reserve_b)
                if _reserve_a > Uint128::zero() && _reserve_b > Uint128::zero() =>
            {
                let _amount_b = match _amount_a.checked_mul(_reserve_b) {
                    Ok(data) => match data.checked_div(_reserve_a) {
                        Ok(data) => data,
                        Err(_) => return Err(ContractError::CalculationOverflow {}),
                    },
                    Err(_) => return Err(ContractError::CalculationOverflow {}),
                };

                Ok(_amount_b)
            }

            _ => return Err(ContractError::InsufficientLiquidity {}),
        }
    }

    fn calculate_amounts(
        _amount_a_desired: Uint128,
        _amount_b_desired: Uint128,
        _amount_a_min: Uint128,
        _amount_b_min: Uint128,
        _reserve_a: Uint128,
        _reserve_b: Uint128,
    ) -> Result<(Uint128, Uint128), ContractError> {
        match calculate_amount(_amount_a_desired, _reserve_a, _reserve_b) {
            Ok(_amount_b_optimal) => {
                if _amount_b_optimal <= _amount_b_desired {
                    if _amount_b_optimal >= _amount_b_min {
                        Ok((_amount_a_desired, _amount_b_optimal))
                    } else {
                        return Err(ContractError::InsufficientBAmount {});
                    }
                } else {
                    match calculate_amount(_amount_b_desired, _reserve_b, _reserve_a) {
                        Ok(amount_a_optimal) => {
                            if amount_a_optimal <= _amount_a_desired {
                                if amount_a_optimal >= _amount_a_min {
                                    Ok((amount_a_optimal, _amount_b_desired))
                                } else {
                                    return Err(ContractError::InsufficientAAmount {});
                                }
                            } else {
                                return Err(ContractError::AddingLiquidityFailed {});
                            }
                        }
                        Err(_) => return Err(ContractError::CalculationAmountError {}),
                    }
                }
            }
            Err(_) => return Err(ContractError::CalculationAmountError {}),
        }
    }

    fn execute_liquidity(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _pool_address: String,
        _token_a: String,
        _token_b: String,
        _amount_a_desired: Uint128,
        _amount_b_desired: Uint128,
        _amount_a_min: Uint128,
        _amount_b_min: Uint128,
        _reserve_a: Uint128,
        _reserve_b: Uint128,
        _address_to: String,
    ) -> Result<(WasmMsg, WasmMsg, WasmMsg, WasmMsg), ContractError> {
        let (_amount_a, _amount_b) =
            if _reserve_a == Uint128::zero() && _reserve_b == Uint128::zero() {
                (_amount_a_desired, _amount_b_desired)
            } else {
                match calculate_amounts(
                    _amount_a_desired,
                    _amount_b_desired,
                    _amount_a_min,
                    _amount_b_min,
                    _reserve_a,
                    _reserve_b,
                ) {
                    Ok(data) => (data.0, data.1),
                    Err(_) => return Err(ContractError::CalculationAmountError {}),
                }
            };

        let excute_transfer_token_a = WasmMsg::Execute {
            contract_addr: _token_a,
            msg: to_binary(&cw20_base::msg::ExecuteMsg::TransferFrom {
                owner: _info.sender.to_string(),
                recipient: _env.contract.address.to_string(),
                amount: _amount_a,
            })?,
            funds: vec![],
        };

        let excute_transfer_token_b = WasmMsg::Execute {
            contract_addr: _token_b,
            msg: to_binary(&cw20_base::msg::ExecuteMsg::TransferFrom {
                owner: _info.sender.to_string(),
                recipient: _env.contract.address.to_string(),
                amount: _amount_b,
            })?,
            funds: vec![],
        };

        let execute_pool = WasmMsg::Execute {
            contract_addr: _pool_address.clone(),
            msg: to_binary(&uniswapv2_pool::msg::ExecuteMsg::Mint(
                uniswapv2_pool::msg::MintRecieveParams {
                    to: _info.sender.to_string(),
                    amount0: _amount_a,
                    amount1: _amount_b,
                },
            ))?,
            funds: vec![],
        };



        let execute_update_liquidity = match _update_liquidity(
            _env,
            _pool_address,
            _amount_a,
            _amount_b,
            String::from("AddLiquidity"),
        ) {
            Ok(data) => data,
            Err(_) => return Err(ContractError::UpateLiquidityFailed {}),
        };

        Ok((
            excute_transfer_token_a,
            excute_transfer_token_b,
            execute_pool,
            execute_update_liquidity,
        ))
    }

    pub fn execute_add_liquidity(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _add_liquidity_params: AddLiquidityParams,
    ) -> Result<Response, ContractError> {
        let fetch_pool_data =
            POOL_REGISTER.load(_deps.storage, _add_liquidity_params.pool_address.clone());

        match fetch_pool_data {
            Ok(data) => {
                let (
                    execute_token_a,
                    execute_token_b,
                    execute_token_pool,
                    execute_update_liquidity,
                ) = if data.token0 == _add_liquidity_params.token_a {
                    let result = execute_liquidity(
                        _deps,
                        _env,
                        _info,
                        _add_liquidity_params.pool_address,
                        _add_liquidity_params.token_a,
                        _add_liquidity_params.token_b,
                        _add_liquidity_params.amount_a_desired,
                        _add_liquidity_params.amount_b_desired,
                        _add_liquidity_params.amount_a_min,
                        _add_liquidity_params.amount_b_min,
                        data.reserve0,
                        data.reserve1,
                        _add_liquidity_params.address_to,
                    );
                    match result {
                        Ok(data) => (data.0, data.1, data.2, data.3),
                        Err(_) => {
                            return Err(ContractError::CustomError {
                                val: "execute liquidity failed!".to_string(),
                            })
                        }
                    }
                } else {
                    match execute_liquidity(
                        _deps,
                        _env,
                        _info,
                        _add_liquidity_params.pool_address,
                        _add_liquidity_params.token_b,
                        _add_liquidity_params.token_a,
                        _add_liquidity_params.amount_b_desired,
                        _add_liquidity_params.amount_a_desired,
                        _add_liquidity_params.amount_b_min,
                        _add_liquidity_params.amount_a_min,
                        data.reserve1,
                        data.reserve0,
                        _add_liquidity_params.address_to,
                    ) {
                        Ok(data) => (data.0, data.1, data.2, data.3),
                        Err(_) => {
                            return Err(ContractError::CustomError {
                                val: "execute liquidity failed!".to_string(),
                            })
                        }
                    }
                };
                Ok(Response::new()
                    .add_message(execute_token_a)
                    .add_message(execute_token_b)
                    .add_message(execute_token_pool)
                    .add_message(execute_update_liquidity))
            }
            Err(_) => return Err(ContractError::FetchPoolDataError {}),
        }
    }

    /**
     * 3. execute_remove_liquidity: This function allows users to remove liquidity from a pool by specifying
     * the pool address, the tokens they want to withdraw, the minimum acceptable amounts of each token, the recipient's
     * address for receiving the tokens, and a deadline for the operation.
     */
    pub fn execute_remove_liquidity(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _remove_liquidity_params: RemoveLiquidityParams,
    ) -> Result<Response, ContractError> {
        let pool_exist =
            POOL_REGISTER.load(_deps.storage, _remove_liquidity_params.pool_address.clone());

        match pool_exist {
            Ok(_data) => {
                let execute_pool_liquidity = WasmMsg::Execute {
                    contract_addr: _remove_liquidity_params.pool_address.clone(),
                    msg: to_binary(&uniswapv2_pool::msg::ExecuteMsg::Transfer {
                        owner: _info.sender.to_string(),
                        recipient: _remove_liquidity_params.pool_address,
                        amount: _remove_liquidity_params.liquidity,
                    })?,
                    funds: vec![],
                };

                let liquidity_amounts: Result<LiquidityAmounts, _> =
                    _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                        contract_addr: _remove_liquidity_params.pool_address.clone(),
                        msg: to_binary(&uniswapv2_pool::msg::QueryMsg::GetAmountTransferToken)?,
                    }));

                let liquidity_amount_burn = WasmMsg::Execute {
                    contract_addr: _remove_liquidity_params.pool_address.clone(),
                    msg: to_binary(&uniswapv2_pool::msg::ExecuteMsg::Burn)?,
                    funds: vec![],
                };

                match liquidity_amounts {
                    Ok(amounts) => {
                        let excute_transfer_token_a = WasmMsg::Execute {
                            contract_addr: _data.token0,
                            msg: to_binary(&cw20_base::msg::ExecuteMsg::TransferFrom {
                                owner: _env.contract.address.to_string(),
                                recipient: _remove_liquidity_params.address_to,
                                amount: amounts.amount_a,
                            })?,
                            funds: vec![],
                        };

                        let excute_transfer_token_b = WasmMsg::Execute {
                            contract_addr: _data.token1,
                            msg: to_binary(&cw20_base::msg::ExecuteMsg::TransferFrom {
                                owner: _env.contract.address.to_string(),
                                recipient: _remove_liquidity_params.address_to,
                                amount: amounts.amount_b,
                            })?,
                            funds: vec![],
                        };

                        let execute_update_liquidity = match _update_liquidity(
                            _env,
                            _remove_liquidity_params.pool_address,
                            amounts.amount_a,
                            amounts.amount_b,
                            String::from("RemoveLiquidity"),
                        ) {
                            Ok(execute_msg) => execute_msg,
                            Err(_) => return Err(ContractError::UpateLiquidityFailed {}),
                        };

                        Ok(Response::new()
                            .add_message(execute_pool_liquidity)
                            .add_message(liquidity_amount_burn)
                            .add_message(excute_transfer_token_a)
                            .add_message(excute_transfer_token_b)
                            .add_message(execute_update_liquidity))
                    }
                    Err(_) => return Err(ContractError::PoolNotExisted {}),
                }
            }
            Err(_) => return Err(ContractError::PoolNotExisted {}),
        }
    }

    fn _update_liquidity(
        _env: Env,
        _pool_address: String,
        _amount_a: Uint128,
        _amount_b: Uint128,
        _feature: String,
    ) -> Result<WasmMsg, ContractError> {
        let _execute_update_tx = WasmMsg::Execute {
            contract_addr: _env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::UpdateReserves(UpdateLiquidiyParams {
                pool_address: _pool_address,
                amount_a: _amount_a,
                amount_b: _amount_b,
                feature: _feature,
            }))?,
            funds: vec![],
        };

        Ok(_execute_update_tx)
    }

    pub fn execute_update_liquidity(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _update_liquidity_params: UpdateLiquidiyParams,
    ) -> Result<Response, ContractError> {
        let _update_pool_register = POOL_REGISTER.update(
            _deps.storage,
            _update_liquidity_params.pool_address,
            |pool_data| -> Result<PoolData, ContractError> {
                match pool_data {
                    Some(mut pool) => {
                        if _update_liquidity_params.feature == String::from("AddLiquidity") {
                            pool.reserve0.add_assign(_update_liquidity_params.amount_a);
                            pool.reserve1.add_assign(_update_liquidity_params.amount_b);
                        } else if _update_liquidity_params.feature
                            == String::from("RemoveLiquidity")
                        {
                            pool.reserve0.sub_assign(_update_liquidity_params.amount_a);
                            pool.reserve1.sub_assign(_update_liquidity_params.amount_b);
                        }
                        // else for swapping tokens

                        Ok(pool)
                    }
                    None => return Err(ContractError::PoolNotExisted {}),
                }
            },
        );

        Ok(Response::new().add_attribute("function", "execute_update_liquidity"))
    }

    pub fn execute_swap_tokens(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _swap_token_params: SwapTokensParams,
    ) -> Result<Response, ContractError> {
        let pool_exist = POOL_REGISTER.load(_deps.storage, _swap_token_params.pool_address);
        match pool_exist {
            Ok(data) => {
                if _swap_token_params.token_in == data.token0 {
                    let amount_out: Result<Uint128, _> =
                        _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                            contract_addr: _swap_token_params.pool_address,
                            msg: to_binary(&uniswapv2_pool::msg::QueryMsg::GetAmountOut(
                                uniswapv2_pool::msg::AmountOutParams {
                                    amountIn: _swap_token_params.amount_in,
                                    reserveIn: data.reserve0,
                                    reserveOut: data.reserve1,
                                },
                            ))?,
                        }));

                    match amount_out {
                        Ok(_amount_out) => {
                            if _amount_out >= data.reserve1 {
                                return Err(ContractError::CustomError {
                                    val: "Insufficient Balance!".to_string(),
                                });
                            }

                            let transfer_amount_out = WasmMsg::Execute {
                                contract_addr: _swap_token_params.token_out,
                                msg: to_binary(&cw20_base::msg::ExecuteMsg::Transfer {
                                    recipient: _swap_token_params.address_to,
                                    amount: _amount_out,
                                })?,
                                funds: vec![],
                            };

                            // pending update reserves of pool

                            Ok(Response::new().add_message(transfer_amount_out))
                        }
                        Err(_) => {
                            return Err(ContractError::CustomError {
                                val: "Unable to find amount out".to_string(),
                            })
                        }
                    }
                } else {
                    let amount_out: Result<Uint128, _> =
                        _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                            contract_addr: _swap_token_params.pool_address,
                            msg: to_binary(&uniswapv2_pool::msg::QueryMsg::GetAmountOut(
                                uniswapv2_pool::msg::AmountOutParams {
                                    amountIn: _swap_token_params.amount_in,
                                    reserveIn: data.reserve1,
                                    reserveOut: data.reserve0,
                                },
                            ))?,
                        }));

                    match amount_out {
                        Ok(_amount_out) => {
                            if _amount_out >= data.reserve0 {
                                return Err(ContractError::CustomError {
                                    val: "Insufficient Balance!".to_string(),
                                });
                            }

                            let transfer_amount_out = WasmMsg::Execute {
                                contract_addr: _swap_token_params.token_out,
                                msg: to_binary(&cw20_base::msg::ExecuteMsg::Transfer {
                                    recipient: _swap_token_params.address_to,
                                    amount: _amount_out,
                                })?,
                                funds: vec![],
                            };

                            // pending update reserves of pool

                            Ok(Response::new().add_message(transfer_amount_out))
                        }
                        Err(_) => {
                            return Err(ContractError::CustomError {
                                val: "Unable to find amount out".to_string(),
                            })
                        }
                    }
                }
            }
            Err(_) => {
                return Err(ContractError::PoolNotExisted {});
            }
        }

        todo!()
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryPoolData { pool_address } => {
            to_binary(&query::query_pool_data(_deps, _env, pool_address))
        }
    }
}

pub mod query {
    use super::*;

    pub fn query_pool_data(
        _deps: Deps,
        _env: Env,
        _pool_address: String,
    ) -> Result<PoolData, ContractError> {
        let pool_data = POOL_REGISTER.load(_deps.storage, _pool_address);

        match pool_data {
            Ok(data) => Ok(data),
            Err(_) => return Err(ContractError::PoolNotExisted {}),
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    todo!()
}
