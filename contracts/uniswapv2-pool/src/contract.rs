#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20_base::contract::{execute_burn, execute_mint, query_balance, query_token_info};
use cw20_base::state::{MinterData, TokenInfo, BALANCES, TOKEN_INFO};

use crate::error::ContractError;
use crate::msg::{
    AmountInParams, AmountOutParams, ExecuteMsg, InstantiateMsg, MigrateMsg, MintRecieveParams,
    QueryMsg, VaultMsgEnums,
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
        ExecuteMsg::Burn {} => execute::execute_pool_burn(_deps, _env, _info),
        ExecuteMsg::BurnLpToken { amount } => {
            execute::execute_burn_lptokens(_deps, _env, _info, amount)
        }
        ExecuteMsg::MintLpToken { recipient, amount } => {
            execute::execute_mint_lptokens(_deps, _env, _info, recipient, amount)
        }
        ExecuteMsg::Transfer {
            owner,
            recipient,
            amount,
        } => execute::execute_transfer_lptoken(_deps, _env, _info, owner, recipient, amount),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => execute::execute_inc_allowance(_deps, _env, _info, spender, amount, expires),
    }
}

pub mod execute {

    use std::ops::{Mul, Sub};

    use crate::msg::PoolDataResponse;

    use super::*;
    use cosmwasm_std::{QueryRequest, WasmMsg, WasmQuery};
    use cw20::Expiration;

    use cw20_base::allowances::{execute_increase_allowance, execute_transfer_from};

    // mint functionality
    pub fn execute_pool_mint(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: MintRecieveParams,
    ) -> Result<Response, ContractError> {
        // 1. get total supply of lp tokens
        let total_supply = match TOKEN_INFO.load(_deps.storage) {
            Ok(data) => data.total_supply,
            Err(_) => return Err(ContractError::FetchTotalSupplyFailed {}),
        };

        // 2. if total supply is zero. then call the lptoken_mint with some min_amount else calculate
        let amount0 = _msg.amount0;
        let amount1 = _msg.amount1;

        let liquidity;
        let mut min_liquidity_execute = None;
        if total_supply.is_zero() {
            let min_liquidity = Uint128::from(1000u128);
            liquidity = Uint128::from(
                u128::from(amount0.mul(amount1))
                    .sqrt()
                    .sub(u128::from(min_liquidity)),
            );
            // 3. call into cw20-base to mint tokens to owner, call as self as no one else is allowed
            min_liquidity_execute = Some(WasmMsg::Execute {
                contract_addr: _env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::MintLpToken {
                    recipient: "undefined".to_string(),
                    amount: min_liquidity,
                })?,
                funds: vec![],
            });
        } else {
            // 4. first we need how much token0 and token1 has in specific pool contract.
            // 5. we need token0 and token1 reserve in sorted order
            // 6. find amount deposite
            let pool_data: Result<PoolDataResponse, _> =
                _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: _info.sender.to_string(),
                    msg: to_binary(&VaultMsgEnums::QueryPoolData {
                        pool_address: _env.contract.address.to_string(),
                    })?,
                }));

            match pool_data {
                Ok(data) => {
                    liquidity = min(
                        (amount0 * total_supply) / data.reserve0,
                        (amount1 * total_supply) / data.reserve1,
                    );
                }
                Err(_) => return Err(ContractError::QueryFailed {}),
            }
        }

        // 6. chck L should not be zero
        if liquidity.le(&Uint128::from(0u128)) {
            return Err(ContractError::InsufficientLiquidity {});
        }

        // 7. mint lptoken to user
        // match execute_mint(_deps.branch(), _env.clone(), _info, _msg.to, liquidity) {
        //     Ok(data) => data
        //         .add_attribute("minted_to", _msg.to)
        //         .add_attribute("minted_amount", liquidity),
        //     Err(_) => return Err(ContractError::MintTokenFailed {}),
        // };
        let mint_execute_tx = WasmMsg::Execute {
            contract_addr: _env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::MintLpToken {
                recipient: _msg.to,
                amount: liquidity,
            })?,
            funds: vec![],
        };

        if total_supply.is_zero() {
            Ok(Response::new()
                .add_message(min_liquidity_execute.unwrap())
                .add_message(mint_execute_tx))
        } else {
            Ok(Response::new().add_message(mint_execute_tx))
        }

        //8. update new reserve of token0 and token1
        // _update(balance0, balance1);  this will be updated from vault contract
    }

    //burn functionality
    pub fn execute_pool_burn(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
    ) -> Result<Response, ContractError> {
        //1.get liquidity amount to be burned that user have sent to pool
        let liquidity = match BALANCES.load(_deps.storage, &_env.contract.address) {
            Ok(pool_balance) => pool_balance,
            Err(_) => return Err(ContractError::FetchLiquidityFailed {}),
        };

        //2. burn liquidity from pool
        // match execute_burn(_deps, _env.clone(), _info, liquidity) {
        //     Ok(data) => data.add_attribute("burnt_amount", liquidity),
        //     Err(_) => return Err(ContractError::BurnTokenFailed {}),
        // };
        let burn_execute_tx = WasmMsg::Execute {
            contract_addr: _env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::BurnLpToken { amount: liquidity })?,
            funds: vec![],
        };
        Ok(Response::new().add_message(burn_execute_tx))

        //5. transfer amount0 and amount1 to user in vault
        //6. update token balance and pool reserves in sorted order in vault
    }

    pub fn execute_mint_lptokens(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        match execute_mint(_deps, _env, _info, recipient, amount) {
            Ok(data) => Ok(data),
            Err(_) => return Err(ContractError::MintTokenFailed {}),
        }
    }
    pub fn execute_burn_lptokens(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        match execute_burn(_deps, _env, _info, amount) {
            Ok(data) => Ok(data),
            Err(_) => return Err(ContractError::BurnTokenFailed {}),
        }
    }

    pub fn execute_transfer_lptoken(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        owner: String,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        match execute_transfer_from(_deps, _env, _info, owner, recipient, amount) {
            Ok(data) => Ok(data),
            Err(err) => return Err(ContractError::CustomError { val: err.to_string() } ), 
        }
    }

    pub fn execute_inc_allowance(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        spender: String,
        amount: Uint128,
        _expires: Option<Expiration>,
    ) -> Result<Response, ContractError> {
        match execute_increase_allowance(_deps, _env, _info, spender, amount, None) {
            Ok(data) => Ok(data),
            Err(_) => {
                return Err(ContractError::CustomError {
                    val: "increase allowance failed".to_string(),
                })
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(_deps)?),
        QueryMsg::Balance { address } => to_binary(&query_balance(_deps, address)?),
        //TODO - REMOVE QUERY BAL IF NOT USED as we are using BALANCE state
        QueryMsg::GetAmountOut(amount_out_params) => {
            to_binary(&query::query_get_amountout(_deps, _env, amount_out_params)?)
        }
        QueryMsg::GetAmountIn(amount_in_params) => {
            to_binary(&query::query_get_amountin(_deps, _env, amount_in_params)?)
        }
        QueryMsg::GetAmountTransferToken { vault_address } => to_binary(
            &query::get_amount_token_transfer(_deps, _env, vault_address)?,
        ),
    }
}

pub mod query {
    use std::ops::{Add, Div, Mul, Sub};

    use cosmwasm_std::{QueryRequest, WasmQuery};

    use crate::msg::{GetAmountTokenTransfer, PoolDataResponse};

    use super::*;

    pub fn query_get_amountin(_deps: Deps, _env: Env, _msg: AmountInParams) -> StdResult<Uint128> {
        let amount_out = _msg.amountOut;
        let reserve_in = _msg.reserveIn;
        let reserve_out = _msg.reserveOut;

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
        let amount_in = _msg.amountIn;
        let reserve_in = _msg.reserveIn;
        let reserve_out = _msg.reserveOut;

        //1.check amountIn, reserveIn and reserveOut should not be zero
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

        // let liquidity = match BALANCES.load(_deps.storage, &_env.contract.address) {
        //     Ok(pool_balance) => pool_balance,
        //     Err(_) => return Err(cosmwasm_std::StdError::GenericErr { msg: "FetchLiquidityFailed".to_string() })
        // };

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
