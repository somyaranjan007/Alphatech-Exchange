#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

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
        ExecuteMsg::Mint { to } => execute::execute_mint(_deps, _env, _info, to),
        ExecuteMsg::Burn { from } => execute::execute_burn(_deps, _env, _info, from),
    }
}

pub mod execute {
    use cosmwasm_std::{QueryRequest, StdError, Uint128, Uint256, WasmQuery, WasmMsg};
    use cw20::Cw20QueryMsg::TokenInfo;
    use cw20::TokenInfoResponse;

    use super::*;

    pub fn execute_mint(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        to: String,
    ) -> Result<Response, ContractError> {
        //1. first we need how much token0 and token1 has in specific pool contract.
        // let token0_in_pool=match get_pool_data(deps, env){
        // }
        //2. we need token0 and token1 reserve
        //3. find amount deposited
        //4. get total supply of lp tokens
        let lp_token_data: Result<TokenInfoResponse, StdError> =
            _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: lpContractAddress.to_string(),
                msg: to_binary(&TokenInfo {})?,
            }));

        let total_supply = match lp_token_data {
            Ok(data) => data.total_supply,
            Err(_) => {
                return Err(ContractError::CustomError {
                    val: "unable to fetch liquidity".to_string(),
                })
            }
        };

        //5. if total supply is zero. then call the lptoken_mint with some min_amount else calculate
        let liquidity: Uint128;
        if total_supply == Uint128::zero() {
            let min_liq = Uint128::from(10000);

            //execute mint function of lptoken contract
            let execute_mint_tx = WasmMsg::Execute {
                contract_addr: lp_contract_addr,
                msg: to_binary(&cw20::Cw20ExecuteMsg::Mint {
                    recipient: to,
                    amount: min_liq,
                })?,
                funds: vec![],
            };
        }

        //6. chck L should not be zero
        //7. mint lptoken to user
        //8. update new reserve of token0 and token1
        unimplemented!();
    }
    pub fn execute_burn(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        from: String,
    ) -> Result<Response, ContractError> {
        unimplemented!();
    }
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _info: MessageInfo, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPoolData {} => to_binary(&query::get_pool_data(_deps, _env, _info)),
    }
}

pub mod query {
    use cosmwasm_std::{QueryRequest, StdError, WasmQuery};

    use crate::msg::GetPoolDataResponse;

    use super::*;

    pub fn get_pool_data(
        _deps: Deps,
        _env: Env,
        _info: MessageInfo,
    ) -> Result<GetPoolDataResponse, ContractError> {
        let pool_data: Result<PoolDataResponse, StdError> =
            _deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _info.sender.to_string(),
                msg: to_binary(&pool_data_info)?,
            }));
        unimplemented!()
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    todo!()
}
