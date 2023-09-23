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
mod vault_tests {
    use cosmwasm_std::{to_binary, Addr, Empty, Uint128};
    use cw_multi_test::{App, ContractWrapper, Executor};

    #[test]
    fn execute_vault_test() {
        /* addresses for creating transactions */
        let vault_owner = Addr::unchecked("vault_owner");
        let usdc_owner = Addr::unchecked("usdc_owner");
        let usdc_minter = Addr::unchecked("usdc_minter");
        let usdt_owner = Addr::unchecked("usdt_owner");
        let usdt_minter = Addr::unchecked("usdt_minter");
        let factory_owner = Addr::unchecked("factory_owner");

        let liquidity_provider = Addr::unchecked("liquidity_provider");

        /* initializing app */
        let mut app = App::default();

        /* vault contract */
        let code = ContractWrapper::new(
            vault::contract::execute,
            vault::contract::instantiate,
            vault::contract::query,
        );

        let reply_with_code = code.with_reply(vault::contract::reply);

        let code_id = app.store_code(Box::new(reply_with_code));

        let vault_contract_address = app
            .instantiate_contract(
                code_id,
                vault_owner.clone(),
                &Empty {},
                &[],
                "vault contract",
                None,
            )
            .unwrap();

        // token_a usdc instantiated
        let usdc_code = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        let usdc_code_id = app.store_code(Box::new(usdc_code));

        let usdc20 = app
            .instantiate_contract(
                usdc_code_id,
                usdc_owner,
                &cw20_base::msg::InstantiateMsg {
                    name: "usdc".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(cw20::MinterResponse {
                        minter: "usdc_minter".to_string(),
                        cap: None,
                    }),
                    marketing: None,
                },
                &[],
                "usdc20",
                None,
            )
            .unwrap();

        // Minting 1000 usdt tokens to provider
        let _execute_mint = app.execute_contract(
            usdc_minter.clone(),
            usdc20.clone(),
            &cw20_base::msg::ExecuteMsg::Mint {
                recipient: "liquidity_provider".to_string(),
                amount: Uint128::from(1000000u128),
            },
            &[],
        );

        let usdc_query0: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                usdc20.clone(),
                &cw20_base::msg::QueryMsg::Balance {
                    address: "liquidity_provider".to_string(),
                },
            )
            .unwrap();

        // token_b usdt instantiated
        let usdt_code = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        let usdt_code_id = app.store_code(Box::new(usdt_code));

        let usdt20 = app
            .instantiate_contract(
                usdt_code_id,
                usdt_owner,
                &cw20_base::msg::InstantiateMsg {
                    name: "usdc".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(cw20::MinterResponse {
                        minter: "usdt_minter".to_string(),
                        cap: None,
                    }),
                    marketing: None,
                },
                &[],
                "usdc20",
                None,
            )
            .unwrap();

        // Minting 1000 usdt tokens to provier
        let _execute_mint = app.execute_contract(
            usdt_minter.clone(),
            usdt20.clone(),
            &cw20_base::msg::ExecuteMsg::Mint {
                recipient: "liquidity_provider".to_string(),
                amount: Uint128::from(1000000u128),
            },
            &[],
        );

        let usdt_query: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                usdt20.clone(),
                &cw20_base::msg::QueryMsg::Balance {
                    address: "liquidity_provider".to_string(),
                },
            )
            .unwrap();

        println!("usdt query: {:?}", usdt_query);

        /* Instantiating Factory Contract */

        // pool contract code and code_id
        let pool_code = ContractWrapper::new(
            uniswapv2_pool::contract::execute,
            uniswapv2_pool::contract::instantiate,
            uniswapv2_pool::contract::query,
        );

        let pool_code_id = app.store_code(Box::new(pool_code));

        let factory_code = ContractWrapper::new(
            factory::contract::execute,
            factory::contract::instantiate,
            factory::contract::query,
        );

        let factory_code_with_reply = factory_code.with_reply(factory::contract::reply);

        let factory_code_id = app.store_code(Box::new(factory_code_with_reply));

        let factory_contract_address = app
            .instantiate_contract(
                factory_code_id,
                factory_owner.clone(),
                &factory::msg::InstantiateMsg {
                    pool_contract_code_id: pool_code_id,
                    vault_contract: vault_contract_address.to_string().clone(),
                },
                &[],
                "factory contract",
                None,
            )
            .unwrap();

        println!("factory contract code id: {}", factory_code_id);
        println!("factory contract address: {}", factory_contract_address);

        // factory register
        let execute_register_factory = app
            .execute_contract(
                vault_owner.clone(),
                vault_contract_address.clone(),
                &vault::msg::ExecuteMsg::RegisterFactory {
                    factory_address: factory_contract_address.to_string().clone(),
                },
                &[],
            )
            .unwrap();

        // factory execute creating pool
        let execute_create_pool_tx = app.execute_contract(
            Addr::unchecked("fac"),
            factory_contract_address,
            &factory::msg::ExecuteMsg::CreatePool {
                token_a: usdc20.to_string(),
                token_b: usdt20.to_string(),
            },
            &[],
        );

        // println!("exe: {:?}", execute_create_pool_tx);

        match execute_create_pool_tx {
            Ok(data) => {
                let wasm = data.events.iter().find(|ev| {
                    ev.ty == "wasm"
                        && ev
                            .attributes
                            .iter()
                            .any(|attr| attr.key == "pool_contract_address")
                });

                match wasm {
                    Some(_data) => {
                        let attr = _data
                            .attributes
                            .iter()
                            .find(|at| at.key == "pool_contract_address");
                        match attr {
                            Some(data) => {
                                let _execute_approve_to_vault_usdc = app
                                    .execute_contract(
                                        liquidity_provider.clone(),
                                        usdc20.clone(),
                                        &cw20_base::msg::ExecuteMsg::IncreaseAllowance {
                                            spender: vault_contract_address.to_string().clone(),
                                            amount: Uint128::from(10000u128),
                                            expires: None,
                                        },
                                        &[],
                                    )
                                    .unwrap();

                                let _execute_approve_to_vault_usdt = app
                                    .execute_contract(
                                        liquidity_provider.clone(),
                                        usdt20.clone(),
                                        &cw20_base::msg::ExecuteMsg::IncreaseAllowance {
                                            spender: vault_contract_address.to_string().clone(),
                                            amount: Uint128::from(10000u128),
                                            expires: None,
                                        },
                                        &[],
                                    )
                                    .unwrap();

                                // println!(
                                //     "_execute_approve_to_vault_usdc : {:?}",
                                //     _execute_approve_to_vault_usdc
                                // );
                                // println!(
                                //     "_execute_approve_to_vault_usdt : {:?}",
                                //     _execute_approve_to_vault_usdt
                                // );

                                let execute_add_liquidity = app
                                    .execute_contract(
                                        liquidity_provider.clone(),
                                        vault_contract_address.clone(),
                                        &packages::vault_msg::VaultExecuteMsg::AddLiquidity(
                                            packages::vault_msg::AddLiquidityParams {
                                                pool_address: data.value.to_string().clone(),
                                                token_a: usdc20.to_string().clone(),
                                                token_b: usdt20.to_string().clone(),
                                                amount_a_desired: Uint128::from(10000u128),
                                                amount_b_desired: Uint128::from(9000u128),
                                                amount_a_min: Uint128::from(9999u128),
                                                amount_b_min: Uint128::from(8999u128),
                                                address_to: liquidity_provider.to_string().clone(),
                                                deadline: Uint128::from(1u128),
                                            },
                                        ),
                                        &[],
                                    )
                                    .unwrap();
                                println!("addrs - {}", data.value.to_string().clone());

                                let query_add_liquidity: Result<
                                    packages::pool_msg::PoolDataResponse,
                                    _,
                                > = app.wrap().query_wasm_smart(
                                    vault_contract_address.clone(),
                                    &vault::msg::QueryMsg::QueryPoolData {
                                        pool_address: data.value.to_string().clone(),
                                    },
                                );
                                println!("return {:?}", query_add_liquidity);

                                let query_user_lp_balance: Result<cw20::BalanceResponse, _> =
                                    app.wrap().query_wasm_smart(
                                        data.value.to_string().clone(),
                                        &uniswapv2_pool::msg::QueryMsg::Balance {
                                            address: liquidity_provider.to_string().clone(),
                                        },
                                    );
                                println!(
                                    "query_user_lp_balance first: {:?}",
                                    query_user_lp_balance.unwrap()
                                );

                                let _execute_approve_to_vault_usdc = app
                                    .execute_contract(
                                        liquidity_provider.clone(),
                                        usdc20.clone(),
                                        &cw20_base::msg::ExecuteMsg::IncreaseAllowance {
                                            spender: vault_contract_address.to_string().clone(),
                                            amount: Uint128::from(10000u128),
                                            expires: None,
                                        },
                                        &[],
                                    )
                                    .unwrap();

                                let _execute_approve_to_vault_usdt = app
                                    .execute_contract(
                                        liquidity_provider.clone(),
                                        usdt20.clone(),
                                        &cw20_base::msg::ExecuteMsg::IncreaseAllowance {
                                            spender: vault_contract_address.to_string().clone(),
                                            amount: Uint128::from(9000u128),
                                            expires: None,
                                        },
                                        &[],
                                    )
                                    .unwrap();

                                let query_add_liquidity: Result<
                                    packages::pool_msg::PoolDataResponse,
                                    _,
                                > = app.wrap().query_wasm_smart(
                                    vault_contract_address.clone(),
                                    &vault::msg::QueryMsg::QueryPoolData {
                                        pool_address: data.value.to_string().clone(),
                                    },
                                );

                                println!("return 2333 {:?}", query_add_liquidity);

                                // let query_total_supply: Result<>

                                let execute_add_liquidity = app
                                    .execute_contract(
                                        liquidity_provider.clone(),
                                        vault_contract_address.clone(),
                                        &packages::vault_msg::VaultExecuteMsg::AddLiquidity(
                                            packages::vault_msg::AddLiquidityParams {
                                                pool_address: data.value.to_string().clone(),
                                                token_a: usdc20.to_string().clone(),
                                                token_b: usdt20.to_string().clone(),
                                                amount_a_desired: Uint128::from(10000u128),
                                                amount_b_desired: Uint128::from(9000u128),
                                                amount_a_min: Uint128::from(100u128),
                                                amount_b_min: Uint128::from(100u128),
                                                address_to: liquidity_provider.to_string().clone(),
                                                deadline: Uint128::from(1u128),
                                            },
                                        ),
                                        &[],
                                    )
                                    .unwrap();
                                println!("addrs - {}", data.value.to_string().clone());

                                let query_add_liquidity: Result<
                                    packages::pool_msg::PoolDataResponse,
                                    _,
                                > = app.wrap().query_wasm_smart(
                                    vault_contract_address.clone(),
                                    &vault::msg::QueryMsg::QueryPoolData {
                                        pool_address: data.value.to_string().clone(),
                                    },
                                );

                                println!("return 2 {:?}", query_add_liquidity);

                                let query_user_lp_balance: Result<cw20::BalanceResponse, _> =
                                    app.wrap().query_wasm_smart(
                                        data.value.to_string().clone(),
                                        &uniswapv2_pool::msg::QueryMsg::Balance {
                                            address: liquidity_provider.to_string().clone(),
                                        },
                                    );
                                println!(
                                    "query_user_lp_balance second: {:?}",
                                    query_user_lp_balance.unwrap()
                                );

                                // let _liquidity_send = app
                                //     .execute_contract(
                                //         liquidity_provider.clone(),
                                //         Addr::unchecked(&data.value),
                                //         &cw20_base::msg::ExecuteMsg::Transfer { recipient: data.value.to_string().clone(), amount: Uint128::from(1) },
                                //         },
                                //         &[],
                                //     )
                                //     .unwrap();

                                let usdc_query0: cw20::BalanceResponse = app
                                    .wrap()
                                    .query_wasm_smart(
                                        usdc20.clone(),
                                        &cw20_base::msg::QueryMsg::Balance {
                                            address: "liquidity_provider".to_string(),
                                        },
                                    )
                                    .unwrap();

                                println!("usdc query: {:?}", usdc_query0);

                                let usdt_query: cw20::BalanceResponse = app
                                    .wrap()
                                    .query_wasm_smart(
                                        usdt20.clone(),
                                        &cw20_base::msg::QueryMsg::Balance {
                                            address: "liquidity_provider".to_string(),
                                        },
                                    )
                                    .unwrap();

                                println!("usdt query: {:?}", usdt_query);

                                let _liquidity_send = app
                                    .execute_contract(
                                        liquidity_provider.clone(),
                                        Addr::unchecked(&data.value),
                                        &cw20_base::msg::ExecuteMsg::Send {
                                            contract: data.value.to_string().clone(),
                                            amount: Uint128::from(100u128),
                                            msg: to_binary(
                                                &packages::pool_msg::RemoveLiquidityPoolParams {
                                                    vault_contract_addresss: vault_contract_address
                                                        .to_string(),
                                                    amount_a_min: Uint128::from(50u128),
                                                    amount_b_min: Uint128::from(50u128),
                                                    address_to: liquidity_provider.to_string(),
                                                },
                                            )
                                            .unwrap(),
                                        },
                                        &[],
                                    )
                                    .unwrap();

                                let query_user_lp_balance: Result<cw20::BalanceResponse, _> =
                                    app.wrap().query_wasm_smart(
                                        data.value.to_string().clone(),
                                        &uniswapv2_pool::msg::QueryMsg::Balance {
                                            address: liquidity_provider.to_string().clone(),
                                        },
                                    );
                                println!(
                                    "query_user_lp_balance after: {:?}",
                                    query_user_lp_balance.unwrap()
                                );

                                let usdc_query0: cw20::BalanceResponse = app
                                    .wrap()
                                    .query_wasm_smart(
                                        usdc20.clone(),
                                        &cw20_base::msg::QueryMsg::Balance {
                                            address: "liquidity_provider".to_string(),
                                        },
                                    )
                                    .unwrap();

                                println!("usdc query: {:?}", usdc_query0);

                                let usdt_query: cw20::BalanceResponse = app
                                    .wrap()
                                    .query_wasm_smart(
                                        usdt20.clone(),
                                        &cw20_base::msg::QueryMsg::Balance {
                                            address: "liquidity_provider".to_string(),
                                        },
                                    )
                                    .unwrap();

                                println!("usdt query: {:?}", usdt_query);

                                let usdc_transfer_vault = app.execute_contract(
                                    liquidity_provider.clone(),
                                    usdt20.clone(),
                                    &cw20_base::msg::ExecuteMsg::Send {
                                        contract: vault_contract_address.clone().to_string(),
                                        amount: Uint128::from(100u128),
                                        msg: to_binary(&packages::vault_msg::SwapTokensParams {
                                            message: String::from("execute_swap_tokens"),
                                            pool_address: data.value.clone().to_string(),
                                            amount_out_min: Uint128::from(5u128),
                                            token_in: usdt20.clone().to_string(),
                                            token_out: usdc20.clone().to_string(),
                                            address_to: liquidity_provider.to_string(),
                                        })
                                        .unwrap(),
                                    },
                                    &[],
                                );

                                // let execute_swap_token = app.execute_contract(
                                //     liquidity_provider.clone(),
                                //     vault_contract_address.clone(),
                                //     &packages::vault_msg::VaultExecuteMsg::SwapTokens(
                                //         packages::vault_msg::SwapTokensParams {
                                //             pool_address: data.value.clone().to_string(),
                                //             amount_in: Uint128::from(100u128),
                                //             amount_out_min: Uint128::from(5u128),
                                //             token_in: usdc20.clone().to_string(),
                                //             token_out: usdt20.clone().to_string(),
                                //             address_to: liquidity_provider.to_string(),
                                //         },
                                //     ),
                                //     &[],
                                // );

                                let usdc_query0: cw20::BalanceResponse = app
                                    .wrap()
                                    .query_wasm_smart(
                                        usdc20.clone(),
                                        &cw20_base::msg::QueryMsg::Balance {
                                            address: "liquidity_provider".to_string(),
                                        },
                                    )
                                    .unwrap();

                                println!("usdc query after swap: {:?}", usdc_query0);

                                let usdt_query: cw20::BalanceResponse = app
                                    .wrap()
                                    .query_wasm_smart(
                                        usdt20.clone(),
                                        &cw20_base::msg::QueryMsg::Balance {
                                            address: "liquidity_provider".to_string(),
                                        },
                                    )
                                    .unwrap();

                                println!("usdt query after swap: {:?}", usdt_query);
                            }
                            None => panic!("Attribute error"),
                        }
                    }
                    None => println!("NONE"),
                }
            }
            Err(err) => {
                panic!("Error at execute factory: {}", err)
            }
        }
    }
}
