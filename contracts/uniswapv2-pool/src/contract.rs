#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20_base::allowances::query_allowance;
use cw20_base::contract::{
    execute_burn, execute_mint, execute_send, query_balance, query_token_info,
};
use cw20_base::state::{MinterData, TokenInfo, BALANCES, TOKEN_INFO};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use packages::pool_msg::{
    AmountInParams, AmountOutParams, Cw20ReceiveMsg, MintRecieveParams, VaultMsgEnums,
};

use num::integer::Roots;
use std::cmp::min;

const CONTRACT_NAME: &str = "crates.io:uniswapv2-pool";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // store token info using cw20-base format
    let data = TokenInfo {
        name: _msg.name,
        symbol: _msg.symbol,
        decimals: _msg.decimals,
        total_supply: Uint128::zero(),
        // set self as minter, so we can properly execute mint and burn
        mint: Some(MinterData {
            minter: _env.contract.address,
            cap: None,
        }),
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint(mint_recieve_params) => {
            execute::execute_pool_mint(_deps, _env, _info, mint_recieve_params)
        }
        ExecuteMsg::Receive(cw20_receive_msg) => {
            execute::execute_burn_lp_tokens(_deps, _env, _info, cw20_receive_msg)
        }
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => execute_send(_deps, _env, _info, contract, amount, msg)
            .map_err(|_| ContractError::Unauthorized {}),
    }
}

pub mod execute {

    use packages::pool_msg::{PoolDataResponse, RemoveLiquidityPoolParams};
    use std::ops::{Add, Div, Mul, Sub};

    use super::*;
    use cosmwasm_std::{from_binary, QueryRequest, WasmMsg, WasmQuery};

    /**
     * Execute Burn LP Tokens
     *
     * This function is responsible for executing the burning of LP tokens, which involves
     *  burning of tokens.
     *
     * @param _deps            Mutable dependencies for the contract
     * @param _env             Environment information
     * @param _info            Information about the message sender
     * @param _cw20_receive_msg CW20 token receive message containing the amount and message
     *
     * @returns A Result containing a Response or a ContractError in case of failure.
     */
    pub fn execute_burn_lp_tokens(
        mut _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _cw20_receive_msg: Cw20ReceiveMsg,
    ) -> Result<Response, ContractError> {
        // Fetch the balance of LP tokens in Pool
        let pool_balance = match BALANCES.load(_deps.storage, &_env.contract.address) {
            Ok(pool_balance) => pool_balance,
            Err(_) => return Err(ContractError::FetchLiquidityFailed {}),
        };

        // Check if the received amount (amount_in) matches the pool balance
        if pool_balance != _cw20_receive_msg.amount {
            return Err(ContractError::CustomError {
                val: String::from("amount mismatch"),
            });
        }

        // Deserialize the received message into _remove_liquidity_pool_params
        let _remove_liquidity_pool_params: RemoveLiquidityPoolParams =
            from_binary(&_cw20_receive_msg.msg)?;

        // Query pool data from the vault contract
        let pool_data: Result<PoolDataResponse, _> =
            _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _remove_liquidity_pool_params
                    .vault_contract_addresss
                    .clone(),
                msg: to_binary(&VaultMsgEnums::QueryPoolData {
                    pool_address: _env.contract.address.to_string(),
                })?,
            }));

        match pool_data {
            // Fetch the total token supply
            Ok(data) => {
                let total_supply = match TOKEN_INFO.load(_deps.storage) {
                    Ok(supply) => supply.total_supply,
                    Err(_) => {
                        return Err(ContractError::CustomError {
                            val: "FetchTotalSupplyFailed".to_string(),
                        })
                    }
                };

                  // Calculate the amounts to be removed 
                let (amount0, amount1) = (
                    (pool_balance.mul(data.reserve0)).div(total_supply),
                    (pool_balance.mul(data.reserve1)).div(total_supply),
                );

                 // Check if the specified minimum amounts are met
                if _remove_liquidity_pool_params.amount_a_min > amount0
                    || _remove_liquidity_pool_params.amount_b_min > amount1
                {
                    return Err(ContractError::CustomError {
                        val: String::from("Insufficient A and B amount!"),
                    });
                }

                let information = MessageInfo {
                    sender: _env.contract.address.clone(),
                    funds: vec![],
                };
                // Execute the burn operation
                let response =
                    match execute_burn(_deps.branch(), _env.clone(), information, pool_balance) {
                        Ok(response) => response,
                        Err(_) => return Err(ContractError::BurnTokenFailed {}),
                    };
                // Calculate updated reserve values
                let (_reserve_a, _reserve_b) =
                    (data.reserve0.sub(amount0), data.reserve1.sub(amount1));
                
                // send msg to vault contract to REmoveLiquidity
                let _execute_vault_tx = WasmMsg::Execute {
                    contract_addr: _remove_liquidity_pool_params.vault_contract_addresss,
                    msg: to_binary(&packages::vault_msg::VaultExecuteMsg::RemoveLiquidity(
                        packages::vault_msg::RemoveLiquidityParams {
                            token_a: data.token0,
                            token_b: data.token1,
                            reserve_a: _reserve_a,
                            reserve_b: _reserve_b,
                            amount_a: amount0,
                            amount_b: amount1,
                            address_to: _remove_liquidity_pool_params.address_to,
                        },
                    ))?,
                    funds: vec![],
                };

                Ok(response.add_message(_execute_vault_tx))
            }
            Err(_) => {
                return Err(ContractError::CustomError {
                    val: "QueryFailed".to_string(),
                })
            }
        }
    }

/**
 * Execute Pool Mint-
 * This function handles the minting of LP tokens when adding liquidity to a pool.
 */
    pub fn execute_pool_mint(
        mut _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: MintRecieveParams,
    ) -> Result<Response, ContractError> {
        let pool_data: Result<PoolDataResponse, _> =
            _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _info.sender.to_string(),
                msg: to_binary(&VaultMsgEnums::QueryPoolData {
                    pool_address: _env.contract.address.to_string(),
                })?,
            }));

        match pool_data {
            Ok(data) => {
                let total_supply = match TOKEN_INFO.load(_deps.storage) {
                    Ok(data) => data.total_supply,
                    Err(_) => return Err(ContractError::FetchTotalSupplyFailed {}),
                };

                let amount0 = _msg.amount0;
                let amount1 = _msg.amount1;

                let liquidity;

                if total_supply.is_zero() {
                    let min_liquidity = Uint128::from(1000u128);
                    liquidity = Uint128::from(
                        u128::from(amount0.mul(amount1))
                            .sqrt()
                            .sub(u128::from(min_liquidity)),
                    );
                    // Mint tokens to the owner (self) since no one else is allowed
                    let information = MessageInfo {
                        sender: _env.contract.address.clone(),
                        funds: vec![],
                    };

                    let _ = execute_mint(
                        _deps.branch(),
                        _env.clone(),
                        information,
                        "undefined".to_string(),
                        min_liquidity,
                    );
                } else {
                    liquidity = min(
                        (amount0 * total_supply) / data.reserve0,
                        (amount1 * total_supply) / data.reserve1,
                    );
                }

                let (updated_reserve_a, updated_reserve_b) =
                    ((data.reserve0).add(amount0), (data.reserve1).add(amount1));

                if liquidity.le(&Uint128::from(0u128)) {
                    return Err(ContractError::InsufficientLiquidity {});
                }

                let information = MessageInfo {
                    sender: _env.contract.address.clone(),
                    funds: vec![],
                };

                match execute_mint(_deps, _env.clone(), information, _msg.to, liquidity) {
                    Ok(data) => Ok(data
                        .set_data(to_binary(&packages::vault_msg::ExecutePoolReplyData {
                            pool_contract_address: _env.contract.address.to_string(),
                            reserve_a: updated_reserve_a,
                            reserve_b: updated_reserve_b,
                        })?)
                        .add_attribute("minted_amount", liquidity)),
                    Err(_) => return Err(ContractError::MintTokenFailed {}),
                }
            }
            Err(_) => return Err(ContractError::QueryFailed {}),
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(_deps)?),
        QueryMsg::Balance { address } => to_binary(&query_balance(_deps, address)?),
        // TODO - REMOVE QUERY BAL IF NOT USED as we are using BALANCE state
        QueryMsg::GetAmountOut(amount_out_params) => {
            to_binary(&query::query_get_amountout(_deps, _env, amount_out_params)?)
        }
        QueryMsg::GetAmountIn(amount_in_params) => {
            to_binary(&query::query_get_amountin(_deps, _env, amount_in_params)?)
        }
        QueryMsg::GetAmountTransferToken { vault_address } => to_binary(
            &query::get_amount_token_transfer(_deps, _env, vault_address)?,
        ),
        QueryMsg::Allowance { owner, spender } => {
            to_binary(&query_allowance(_deps, owner, spender)?)
        }
    }
}

pub mod query {
    use cosmwasm_std::{QueryRequest, WasmQuery};
    use packages::pool_msg::{GetAmountTokenTransfer, PoolDataResponse};
    use std::ops::{Add, Div, Mul, Sub};

    use super::*;

    pub fn query_get_amountin(_deps: Deps, _env: Env, _msg: AmountInParams) -> StdResult<Uint128> {
        let amount_out = _msg.amount_out;
        let reserve_in = _msg.reserve_in;
        let reserve_out = _msg.reserve_out;

        //1.check amountIn, reserveIn and reserveOut should not be zero
        if amount_out.is_zero() {
            return Err(cosmwasm_std::StdError::GenericErr {
                msg: "InsufficientAmount".to_string(),
            });
        }

        if reserve_in.is_zero() || reserve_out.is_zero() {
            return Err(cosmwasm_std::StdError::GenericErr {
                msg: "InsufficientLiquidity".to_string(),
            });
        }

        let numerator = amount_out.mul(Uint128::from(1000u128)).mul(reserve_out);
        let denominator = reserve_out.sub(amount_out).mul(Uint128::from(997u128));
        let amount_in = numerator.div(denominator).add(Uint128::from(1u128));

        Ok(amount_in)
    }

    pub fn query_get_amountout(
        _deps: Deps,
        _env: Env,
        _msg: AmountOutParams,
    ) -> StdResult<Uint128> {
        let amount_in = _msg.amount_in;
        let reserve_in = _msg.reserve_in;
        let reserve_out = _msg.reserve_out;

        // 1.check amountIn, reserveIn and reserveOut should not be zero
        if amount_in.is_zero() {
            return Err(cosmwasm_std::StdError::GenericErr {
                msg: "InsufficientAmount".to_string(),
            });
        }

        if reserve_in.is_zero() || reserve_out.is_zero() {
            return Err(cosmwasm_std::StdError::GenericErr {
                msg: "InsufficientLiquidity".to_string(),
            });
        }

        let amount_in_with_fee = amount_in.mul(Uint128::from(997u128));
        let numerator = amount_in_with_fee.mul(reserve_out);
        let denominator = (reserve_in.mul(Uint128::from(1000u128))).add(amount_in_with_fee);
        let amountout = numerator.div(denominator);

        Ok(amountout)
    }

    pub fn get_amount_token_transfer(
        _deps: Deps,
        _env: Env,
        _vault_address: String,
    ) -> StdResult<GetAmountTokenTransfer> {
        let pool_data: Result<PoolDataResponse, _> =
            _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _vault_address,
                msg: to_binary(&VaultMsgEnums::QueryPoolData {
                    pool_address: _env.contract.address.to_string(),
                })?,
            }));

        let (balance0, balance1) = match pool_data {
            Ok(data) => (data.reserve0, data.reserve1),
            Err(_) => {
                return Err(cosmwasm_std::StdError::GenericErr {
                    msg: "QueryFailed".to_string(),
                })
            }
        };

        let total_supply = match TOKEN_INFO.load(_deps.storage) {
            Ok(data) => data.total_supply,
            Err(_) => {
                return Err(cosmwasm_std::StdError::GenericErr {
                    msg: "FetchTotalSupplyFailed".to_string(),
                })
            }
        };

        let balance_response: Result<cw20::BalanceResponse, _> =
            _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _env.contract.address.to_string(),
                msg: to_binary(&cw20_base::msg::QueryMsg::Balance {
                    address: _env.contract.address.to_string(),
                })?,
            }));

        let liquidity = match balance_response {
            Ok(data) => data.balance,
            Err(_) => {
                return Err(cosmwasm_std::StdError::GenericErr {
                    msg: "unable to fetch".to_string(),
                });
            }
        };

        // let liquidity= Uint128::from(1u128);

        let amount0 = (liquidity.mul(balance0)).div(total_supply);
        let amount1 = (liquidity.mul(balance1)).div(total_supply);

        Ok(GetAmountTokenTransfer {
            amount_a: amount0,
            amount_b: amount1,
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    todo!()
}
