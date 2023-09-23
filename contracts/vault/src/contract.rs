use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{FACTORY_REGISTER, POOL_REGISTER, VAULT_OWNER};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, QueryRequest, Reply, Response,
    StdError, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use packages::vault_msg::{
    AddLiquidityParams, ContractMsg, Cw20ReceiveMsg, ExecutePoolReplyData, PoolDataResponse,
    RegisterPoolParams, RemoveLiquidityParams, SwapTokensParams, UpdateLiquidiyParams,
};

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
        ExecuteMsg::Receive(cw_receive_msg) => {
            execute::execute_swap_tokens(_deps, _env, _info, cw_receive_msg)
        }
    }
}

pub mod execute {
    use cosmwasm_std::from_binary;

    use super::*;
    use std::ops::{AddAssign, SubAssign};

    /**
     * Internal Functions
     *
     * 1. execute_wasm_execute: This function generates multiple `Execute` messages to interact with other CosmWasm contracts.
     *
     * @param _contract_msg A vector of `ContractMsg` objects, each containing information about the target contract
     * address and the binary message to be sent to that contract.
     *
     * @returns A vector of `WasmMsg` containing the generated `Execute` messages.
     */
    fn execute_wasm_execute(_contract_msg: Vec<ContractMsg>) -> Vec<WasmMsg> {
        let execute_messages = _contract_msg
            .iter()
            .map(|contract_msg| WasmMsg::Execute {
                contract_addr: contract_msg.contract_address.clone(),
                msg: contract_msg.contract_msg.clone(),
                funds: vec![],
            })
            .collect();

        execute_messages
    }

    /**
     * 2. Calculate Amount: This function calculates the value of `amount_b` based on the provided parameters.
     *
     * @param _amount_a The input amount.
     * @param _reserve_a The reserve of token A in the liquidity pool.
     * @param _reserve_b The reserve of token B in the liquidity pool.
     *
     * @returns Result<Uint128, ContractError> A Result containing the calculated `amount_b` or an error
     * if the calculation cannot be performed due to insufficient amounts or potential overflow.
     */
    fn calculate_amount(
        _amount_a: Uint128,
        _reserve_a: Uint128,
        _reserve_b: Uint128,
    ) -> Result<Uint128, ContractError> {
        // Check if _amount_a is zero; in that case, it's impossible to perform the calculation.
        if _amount_a == Uint128::zero() {
            return Err(ContractError::InsufficientAmount {});
        }

        // Check if both reserves are greater than zero, as it's required for the calculation.
        if _reserve_a > Uint128::zero() && _reserve_b > Uint128::zero() {
            let _amount_b = match _amount_a.checked_mul(_reserve_b) {
                Ok(data) => match data.checked_div(_reserve_a) {
                    Ok(data) => data,
                    Err(_) => return Err(ContractError::CalculationOverflow {}),
                },
                Err(_) => return Err(ContractError::CalculationOverflow {}),
            };

            Ok(_amount_b)
        } else {
            return Err(ContractError::InsufficientLiquidity {});
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
            Err(err) => return Err(err),
        }
    }

    fn execute_liquidity(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _params: AddLiquidityParams,
        mut _reserve_a: Uint128,
        mut _reserve_b: Uint128,
    ) -> Result<Vec<WasmMsg>, ContractError> {
        let (_amount_a, _amount_b) =
            if _reserve_a == Uint128::zero() && _reserve_b == Uint128::zero() {
                (_params.amount_a_desired, _params.amount_b_desired)
            } else {
                match calculate_amounts(
                    _params.amount_a_desired,
                    _params.amount_b_desired,
                    _params.amount_a_min,
                    _params.amount_b_min,
                    _reserve_a,
                    _reserve_b,
                ) {
                    Ok(data) => (data.0, data.1),
                    Err(err) => return Err(err),
                }
            };

        let execute_messages = execute_wasm_execute(vec![
            ContractMsg {
                contract_address: _params.token_a,
                contract_msg: to_binary(&cw20_base::msg::ExecuteMsg::TransferFrom {
                    owner: _info.sender.to_string(),
                    recipient: _env.contract.address.to_string(),
                    amount: _amount_a,
                })?,
            },
            ContractMsg {
                contract_address: _params.token_b,
                contract_msg: to_binary(&cw20_base::msg::ExecuteMsg::TransferFrom {
                    owner: _info.sender.to_string(),
                    recipient: _env.contract.address.to_string(),
                    amount: _amount_b,
                })?,
            },
            ContractMsg {
                contract_address: _params.pool_address,
                contract_msg: to_binary(&packages::pool_msg::PoolExecuteMsg::Mint(
                    packages::pool_msg::MintRecieveParams {
                        to: _info.sender.to_string(),
                        amount0: _amount_a,
                        amount1: _amount_b,
                    },
                ))?,
            },
        ]);

        Ok(execute_messages)
    }

    /* internal functions */
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
                    let pool_data = PoolDataResponse {
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
                let _params = if data.token0 == _add_liquidity_params.token_a {
                    AddLiquidityParams {
                        pool_address: _add_liquidity_params.pool_address,
                        token_a: _add_liquidity_params.token_a,
                        token_b: _add_liquidity_params.token_b,
                        amount_a_desired: _add_liquidity_params.amount_a_desired,
                        amount_b_desired: _add_liquidity_params.amount_b_desired,
                        amount_a_min: _add_liquidity_params.amount_a_min,
                        amount_b_min: _add_liquidity_params.amount_b_min,
                        address_to: _add_liquidity_params.address_to,
                        deadline: _add_liquidity_params.deadline,
                    }
                } else {
                    AddLiquidityParams {
                        pool_address: _add_liquidity_params.pool_address,
                        token_a: _add_liquidity_params.token_b,
                        token_b: _add_liquidity_params.token_a,
                        amount_a_desired: _add_liquidity_params.amount_b_desired,
                        amount_b_desired: _add_liquidity_params.amount_a_desired,
                        amount_a_min: _add_liquidity_params.amount_b_min,
                        amount_b_min: _add_liquidity_params.amount_a_min,
                        address_to: _add_liquidity_params.address_to,
                        deadline: _add_liquidity_params.deadline,
                    }
                };

                let mut execute_messages = match execute_liquidity(
                    _deps,
                    _env,
                    _info,
                    _params,
                    data.reserve0,
                    data.reserve1,
                ) {
                    Ok(messages) => messages,
                    Err(err) => return Err(err),
                };

                let submessage_execute = execute_messages.pop();
                match submessage_execute {
                    Some(submessage) => {
                        const EXECUTE_REPLY_ID: u64 = 1u64;

                        let sub_msg: SubMsg<Empty> =
                            SubMsg::reply_on_success(submessage, EXECUTE_REPLY_ID);

                        Ok(Response::new()
                            .add_messages(execute_messages)
                            .add_submessage(sub_msg))
                    }
                    None => {
                        return Err(ContractError::CustomError {
                            val: String::from("Unable to pop message"),
                        })
                    }
                }
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
        match execute_update_liquidity(
            _deps,
            _env,
            UpdateLiquidiyParams {
                pool_address: _info.sender.to_string(),
                amount_a: _remove_liquidity_params.reserve_a,
                amount_b: _remove_liquidity_params.reserve_b,
            },
        ) {
            Ok(data) => {
                let _execute_messages = execute_wasm_execute(vec![
                    ContractMsg {
                        contract_address: _remove_liquidity_params.token_a,
                        contract_msg: to_binary(&cw20_base::msg::ExecuteMsg::Transfer {
                            recipient: _remove_liquidity_params.address_to.clone(),
                            amount: _remove_liquidity_params.amount_a,
                        })?,
                    },
                    ContractMsg {
                        contract_address: _remove_liquidity_params.token_b,
                        contract_msg: to_binary(&cw20_base::msg::ExecuteMsg::Transfer {
                            recipient: _remove_liquidity_params.address_to,
                            amount: _remove_liquidity_params.amount_b,
                        })?,
                    },
                ]);

                Ok(data.add_messages(_execute_messages))
            }
            Err(_) => {
                return Err(ContractError::CustomError {
                    val: String::from("Update Reserve failed"),
                })
            }
        }
    }

    pub fn execute_update_liquidity(
        _deps: DepsMut,
        _env: Env,
        _update_liquidity_params: UpdateLiquidiyParams,
    ) -> Result<Response, ContractError> {
        let _update_pool_register = POOL_REGISTER.update(
            _deps.storage,
            _update_liquidity_params.pool_address,
            |pool_data| -> Result<PoolDataResponse, ContractError> {
                match pool_data {
                    Some(mut pool) => {
                        pool.reserve0 = _update_liquidity_params.amount_a;
                        pool.reserve1 = _update_liquidity_params.amount_b;
                        Ok(pool)
                    }
                    None => return Err(ContractError::PoolNotExisted {}),
                }
            },
        );

        match _update_pool_register {
            Ok(_) => Ok(Response::new().add_attribute("function", "execute_update_liquidity")),
            Err(_) => return Err(ContractError::UpateLiquidityFailed {}),
        }
    }

    pub fn execute_swap_tokens(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _cw20_receive_msg: Cw20ReceiveMsg,
    ) -> Result<Response, ContractError> {
        let message = String::from("execute_swap_tokens");
        let _swap_token_params: SwapTokensParams = from_binary(&_cw20_receive_msg.msg)?;

        if message != _swap_token_params.message {
            return Err(ContractError::SwapFailed {});
        }

        let pool_exist = POOL_REGISTER.load(_deps.storage, _swap_token_params.pool_address.clone());

        match pool_exist {
            Ok(data) => {
                let mut updated_amount_a = data.reserve0;
                let mut updated_amount_b = data.reserve1;

                let _amount_out = if _swap_token_params.token_in == data.token0 {
                    let amount_out: Result<Uint128, _> =
                        _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                            contract_addr: _swap_token_params.pool_address.clone(),
                            msg: to_binary(&packages::pool_msg::PoolQueryMsg::GetAmountOut(
                                packages::pool_msg::AmountOutParams {
                                    amount_in: _cw20_receive_msg.amount,
                                    reserve_in: data.reserve0,
                                    reserve_out: data.reserve1,
                                },
                            ))?,
                        }));

                    match amount_out {
                        Ok(_amount_out) => {
                            updated_amount_a.add_assign(_cw20_receive_msg.amount);
                            updated_amount_b.sub_assign(_amount_out);
                            _amount_out
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
                            contract_addr: _swap_token_params.pool_address.clone(),
                            msg: to_binary(&packages::pool_msg::PoolQueryMsg::GetAmountOut(
                                packages::pool_msg::AmountOutParams {
                                    amount_in: _cw20_receive_msg.amount,
                                    reserve_in: data.reserve1,
                                    reserve_out: data.reserve0,
                                },
                            ))?,
                        }));

                    match amount_out {
                        Ok(_amount_out) => {
                            updated_amount_a.sub_assign(_amount_out);
                            updated_amount_b.add_assign(_cw20_receive_msg.amount);
                            _amount_out
                        }
                        Err(_) => {
                            return Err(ContractError::CustomError {
                                val: "Unable to find amount out".to_string(),
                            })
                        }
                    }
                };

                match execute_update_liquidity(
                    _deps,
                    _env.clone(),
                    UpdateLiquidiyParams {
                        pool_address: _swap_token_params.pool_address,
                        amount_a: updated_amount_a,
                        amount_b: updated_amount_b,
                    },
                ) {
                    Ok(response) => {
                        let execute_message = WasmMsg::Execute {
                            contract_addr: _swap_token_params.token_out,
                            msg: to_binary(&cw20_base::msg::ExecuteMsg::Transfer {
                                recipient: _swap_token_params.address_to,
                                amount: _amount_out,
                            })?,
                            funds: vec![],
                        };

                        Ok(response.add_message(execute_message))
                    }
                    Err(_) => {
                        return Err(ContractError::CustomError {
                            val: String::from("Update Reserve failed"),
                        })
                    }
                }
            }
            Err(_) => {
                return Err(ContractError::PoolNotExisted {});
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryPoolData { pool_address } => {
            to_binary(&query::query_pool_data(_deps, _env, pool_address)?)
        }
    }
}

pub mod query {
    use super::*;

    pub fn query_pool_data(
        _deps: Deps,
        _env: Env,
        _pool_address: String,
    ) -> StdResult<PoolDataResponse> {
        let pool_data = POOL_REGISTER.load(_deps.storage, _pool_address);

        match pool_data {
            Ok(data) => Ok(data),
            Err(_) => {
                return Err(StdError::GenericErr {
                    msg: "Pool does not exist".to_string(),
                })
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    const EXECUTE_REPLY_ID: u64 = 1u64;

    match _msg.id {
        EXECUTE_REPLY_ID => reply::update_pool_reserve(_deps, _env, _msg),
        _id => {
            return Err(ContractError::CustomError {
                val: String::from("Id doesn't match"),
            })
        }
    }
}

pub mod reply {
    use super::*;
    use cosmwasm_std::from_binary;
    use cw0::parse_reply_execute_data;

    pub fn update_pool_reserve(
        _deps: DepsMut,
        _env: Env,
        _msg: Reply,
    ) -> Result<Response, ContractError> {
        let execute_reply_data = parse_reply_execute_data(_msg);

        match execute_reply_data {
            Ok(data) => {
                let reply_data = data.data;

                let response = match reply_data {
                    Some(binary_data) => {
                        let update_data: ExecutePoolReplyData = from_binary(&binary_data)?;

                        let res = match execute::execute_update_liquidity(
                            _deps,
                            _env,
                            UpdateLiquidiyParams {
                                pool_address: update_data.pool_contract_address,
                                amount_a: update_data.reserve_a,
                                amount_b: update_data.reserve_b,
                            },
                        ) {
                            Ok(data) => data,
                            Err(_) => return Err(ContractError::UpateLiquidityFailed {}),
                        };

                        res
                    }
                    None => {
                        return Err(ContractError::CustomError {
                            val: String::from("Unable to find data!"),
                        })
                    }
                };

                Ok(response.add_attribute("function", "value"))
            }
            Err(_) => {
                return Err(ContractError::CustomError {
                    val: String::from("Unable to find data!"),
                })
            }
        }
    }
}
