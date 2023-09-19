#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:testing";
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

    // With `Response` type, it is possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

/// Handling contract migration
/// To make a contract migratable, you need
/// - this entry_point implemented
/// - only contract admin can migrate, so admin has to be set at contract initiation time
/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        // Find matched incoming message variant and execute them with your custom logic.
        //
        // With `Response` type, it is possible to dispatch message to invoke external logic.
        // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    }
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
        // Find matched incoming message variant and execute them with your custom logic.
        //
        // With `Response` type, it is possible to dispatch message to invoke external logic.
        // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    }
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // Find matched incoming message variant and query them your custom logic
        // and then construct your query response with the type usually defined
        // `msg.rs` alongside with the query message itself.
        //
        // use `cosmwasm_std::to_binary` to serialize query response to json binary.
    }
}

/// Handling submessage reply.
/// For more info on submessage and reply, see https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#submessages
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    // With `Response` type, it is still possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages

    todo!()
}

#[cfg(test)]
mod vault_tests {
    use super::*;
    use cosmwasm_std::{Addr, Empty,};
    use cw_multi_test::{App, ContractWrapper, Executor, BankKeeper, Contract};
    use cosmwasm_std::testing::{ mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR };
    use crate::contract::{ execute, instantiate, query };
    use crate::msg::*;

    #[test]
    fn execute_vault_test() {

        fn mock_app() -> App {
            let env = mock_env();
            let api = Box::new(MockApi::default());
            let bank = BankKeeper::new();

            struct AppConfig {
                api: Box<MockApi>,
                env_block: BlockInfo, // Assuming env.block is of type Block
                bank: BankKeeper,
                storage: Box<MockStorage>,
            }

            let app_config = AppConfig {
                api,
                env_block: env.block,
                bank,
                storage: Box::new(MockStorage::new()),
            };

            App::new(|router, api, storage| {
                
            }) 
        }

        let mut app = App::default();
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let vault_contract_address = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("vault_owner"),
                &Empty {},
                &[],
                "Vault Contract",
                None,
            )
            .unwrap();

        println!("vault contract code id: {}", code_id);
        println!("vault contract address: {}", vault_contract_address);

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
                Addr::unchecked("usdc owner"),
                &cw20_base::msg::InstantiateMsg {
                    name: "usdc".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(cw20::MinterResponse {
                        minter: "usdc_executor".to_string(),
                        cap: None,
                    }),
                    marketing: None,
                },
                &[],
                "usdc20",
                None,
            )
            .unwrap();

        println!("usdc20 contract address: {}", usdc20);

        // Minting 1000 usdt tokens to provier      
        let _execute_mint = app.execute_contract(
            Addr::unchecked("usdc_executor"),
            usdc20.clone(),
            &cw20_base::msg::ExecuteMsg::Mint {
                recipient: "provider".to_string(),
                amount: Uint128::from(1000u128),
            },
            &[],
        );

        let usdc_query: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                usdc20,
                &cw20_base::msg::QueryMsg::Balance {
                    address: "provider".to_string(),
                },
            )
            .unwrap();

        println!("provider balance: {:?}", usdc_query);

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
                Addr::unchecked("usdc owner"),
                &cw20_base::msg::InstantiateMsg {
                    name: "usdc".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(cw20::MinterResponse {
                        minter: "usdt_executor".to_string(),
                        cap: None,
                    }),
                    marketing: None,
                },
                &[],
                "usdc20",
                None,
            )
            .unwrap();

        println!("usdt20 contract address: {}", usdt20);

        // Minting 1000 usdt tokens to provier            
        let _execute_mint = app.execute_contract(
            Addr::unchecked("usdt_executor"),
            usdt20.clone(),
            &cw20_base::msg::ExecuteMsg::Mint {
                recipient: "provider".to_string(),
                amount: Uint128::from(1000u128),
            },
            &[],
        );

        let usdt_query: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                usdt20,
                &cw20_base::msg::QueryMsg::Balance {
                    address: "provider".to_string(),
                },
            )
            .unwrap();

        println!("provider balance: {:?}", usdt_query);

        // Providing Liquiity to vault

        let factory_code = ContractWrapper::new()
    }
}
