#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20_base::contract::{execute_burn, execute_mint, query_token_info, query_balance};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO, BALANCES};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, MintRecieveParams, QueryMsg, SwapRecieveParams};
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
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
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
        ExecuteMsg::Mint(MintRecieveParams) => execute::execute_pool_mint(_deps, _env, _info, MintRecieveParams),
        ExecuteMsg::Burn (BurnRecieveParams) => execute::execute_pool_burn(_deps, _env, _info, BurnRecieveParams),
        ExecuteMsg::Swap(SwapRecieveParams) => execute:: execute_pool_swap(_deps,_env,_info,SwapRecieveParams),
    }
}

pub mod execute {
    use std::ops::Mul;

    use super::*;
    use crate::{msg::{PoolDataResponse, BurnRecieveParams}, ContractError};
    use cosmwasm_std::{QueryRequest, StdError, WasmQuery};

    //mint functionality
    pub fn execute_pool_mint(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: MintRecieveParams,
    ) -> Result<Response, ContractError> {
       
        //1. first we need how much token0 and token1 has in specific pool contract.
        //2. we need token0 and token1 reserve in sorted order
        //3. find amount deposited
        let pool_data: Result<PoolDataResponse, StdError> =
            match _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _info.sender.to_string(),
                msg: to_binary(&pool_data_info)?,
            })) {
                Ok(data) => Ok(data),
                Err(_) => {
                    return Err(ContractError::CustomError {
                        val: "Unable to fetch reserves".to_string(),
                    });
                }
            };

        //4. get total supply of lp tokens
        let total_supply = match TOKEN_INFO.load(_deps.storage) {
            Ok(data) => data.total_supply,
            Err(_) => {
                return Err(ContractError::CustomError {
                    val: "unable to fetch total supply".to_string(),
                })
            }
        };

        //5. if total supply is zero. then call the lptoken_mint with some min_amount else calculate
        let amount0 = _msg.amount0;
        let amount1 = _msg.amount1;

        let liquidity;
        if total_supply.is_zero() {
            let min_liquidity = Uint128::from(1000);
            liquidity =
                Uint128::from(u128::from(amount0.mul(amount1)).sqrt() - u128::from(min_liquidity));
            // call into cw20-base to mint tokens to owner, call as self as no one else is allowed
            match execute_mint(
                _deps.branch(),
                _env.clone(),
                _info,
                "0".to_string(),
                min_liquidity,
            ) {
                Ok(data) => data.add_attribute("minted_to", "adress0x"),
                Err(_) => {
                    return Err(ContractError::CustomError {
                        val: "unable to mint tokens".to_string(),
                    })
                }
            };
        } else {
            liquidity = min(
                (amount0 * total_supply) / pool_data.reserve0,
                (amount1 * total_supply) / pool_data.reserve1,
            )
        }

        //6. chck L should not be zero
        if liquidity.le(&Uint128::from(0)) {
            return Err(ContractError::CustomError {
                val: "liquiity is insufficient".to_string(),
            });
        }

        //7. mint lptoken to user
        match execute_mint(_deps.branch(), _env.clone(), _info, _msg.to, liquidity) {
            Ok(data) => data
                .add_attribute("minted_to", _msg.to)
                .add_attribute("minted_amount", liquidity),
            Err(_) => {
                return Err(ContractError::CustomError {
                    val: "unable to mint tokens to user".to_string(),
                })
            }
        };

        //8. update new reserve of token0 and token1
        // _update(balance0, balance1);  this will be updated from vault contract

        // Ok(Response::new());
        unimplemented!();
    }

    //burn functionality
    pub fn execute_pool_burn(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: BurnRecieveParams,
    ) -> Result<Response, ContractError> {
        
        //1.get amount0 and amount1
        let amount0 = _msg.amount0;
        let amount1 = _msg.amount1;
        //2.get liquidity amount to be burned that user have sent to pool
        let liquidity = match BALANCES.load(_deps.storage, &_env.contract.address) {
            Ok(poolBalance) => poolBalance,
            Err(_) => return Err(ContractError::CustomError { val: "unable to fetch liquiity".to_string() })
        };
        //3. check any amount0 or amount1 should not be zero
        if amount0.le(&Uint128::from(0u128)) || amount1.le(&Uint128::from(0u128)) {
            return Err(ContractError::CustomError { val: " InsufficientLiquidityBurned".to_string() });
        } 
        
        //4. burn liquidity from pool 
        match execute_burn(_deps, _env.clone(), _info, liquidity) {
            Ok(data) => data
                .add_attribute("burnt_amount", liquidity),
            Err(_) => {
                return Err(ContractError::CustomError {
                    val: "unable to burn tokens of user".to_string(),
                })
            }
        };
        
        //5. transfer amount0 and amount1 to user in vault
        //6. update token balance and pool reserves in sorted order in vault
        // Ok(Response::new());
        unimplemented!();
    }

    pub fn execute_pool_swap(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: SwapRecieveParams,
    )->Result<Response,ContractError>{
        
        //1. chck amount0Out and amount1Out should not be zero
        //2. get  reserves
        //3. check availaible liquidity amount0Out or  amount1Out should less than reserves
        //4. transfer token0 and token1 to user if amount0Out and amount1Out more than zero
        //5. 
        unimplemented!()
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _info: MessageInfo, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(_deps)?),
        QueryMsg::Balance { address }=> to_binary(&query_balance(_deps, address)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    todo!()
}
