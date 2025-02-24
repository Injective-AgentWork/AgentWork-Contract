#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Addr};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::*;

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:agent-work";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let token_info = TokenInfo {
        token_denom: msg.token_symbol,
        token_address: msg.token_contract_addr,
    };
    TOKEN_INFO.save(deps.storage, &token_info)?;

    Ok(Response::new().add_attribute("method", "instantiate"))

}

// user stake, unstake
// agent stake, unstake
// distribute rewards: send stake amount to agent 
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UserStake { amount } => execute::user_stake(deps, env, info, amount),
        ExecuteMsg::UserUnstake { amount } => execute::user_unstake(deps, info, amount),
        ExecuteMsg::AgentStake { amount } => execute::agent_stake(deps, env, info, amount),
        ExecuteMsg::AgentUnstake { amount } => execute::agent_unstake(deps, info, amount),
        ExecuteMsg::DistributeRewards { rewards_owner_addr, agent_addr_list} => execute::distribute_rewards(deps, rewards_owner_addr, agent_addr_list),
    }
}

pub mod execute {
    use cosmwasm_std::{to_json_binary, CosmosMsg, WasmMsg};
    use super::*;

    pub fn user_stake(
        deps: DepsMut, 
        env: Env,
        info: MessageInfo,
        amount: Uint128
    ) -> Result<Response, ContractError> {
        let token_info = TOKEN_INFO.load(deps.storage)?;
        let mut user_stake_amount = USER_STAKE.load(deps.storage, info.sender.clone()).unwrap_or(Uint128::zero());
        user_stake_amount += amount;
        USER_STAKE.save(deps.storage, info.sender.clone(), &user_stake_amount)?;
        
        let transfer_from_msg = cw20::Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount
        };

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_info.token_address.to_string(),
            msg: to_json_binary(&transfer_from_msg)?,
            funds: info.funds
        });

        Ok(Response::new().add_attribute("action", "user stake").add_message(msg))
    }
    
    pub fn user_unstake(
        deps: DepsMut, 
        info: MessageInfo,
        amount: Uint128
    ) -> Result<Response, ContractError> {
        let token_info = TOKEN_INFO.load(deps.storage)?;
        let mut user_stake_amount = USER_STAKE.load(deps.storage, info.sender.clone()).unwrap_or(Uint128::zero());
        if user_stake_amount < amount {
            return Err(ContractError::InsufficientStake {})
        } else {
            user_stake_amount -= amount;
        };
        USER_STAKE.save(deps.storage, info.sender.clone(), &user_stake_amount)?;
        
        let transfer_msg = cw20::Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount,
        };

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_info.token_address.to_string(),
            msg: to_json_binary(&transfer_msg)?,
            funds: info.funds
        });

        Ok(Response::new().add_attribute("action", "user unstake").add_message(msg))
    }

    pub fn agent_stake(
        deps: DepsMut, 
        env: Env,
        info: MessageInfo,
        amount: Uint128
    ) -> Result<Response, ContractError> {
        let token_info = TOKEN_INFO.load(deps.storage)?;
        let mut agent_stake_amount = AGENT_STAKE.load(deps.storage, info.sender.clone()).unwrap_or(Uint128::zero());
        agent_stake_amount += amount;
        AGENT_STAKE.save(deps.storage, info.sender.clone(), &agent_stake_amount)?;
        
        let transfer_from_msg = cw20::Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount
        };

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_info.token_address.to_string(),
            msg: to_json_binary(&transfer_from_msg)?,
            funds: info.funds
        });

        Ok(Response::new().add_attribute("action", "agent stake").add_message(msg))
    }

    pub fn agent_unstake(
        deps: DepsMut, 
        info: MessageInfo,
        amount: Uint128
    ) -> Result<Response, ContractError> {
        let token_info = TOKEN_INFO.load(deps.storage)?;
        let mut agent_stake_amount = AGENT_STAKE.load(deps.storage, info.sender.clone()).unwrap_or(Uint128::zero());
        if agent_stake_amount < amount {
            return Err(ContractError::InsufficientStake {})
        } else {
            agent_stake_amount -= amount;
        };
        AGENT_STAKE.save(deps.storage, info.sender.clone(), &agent_stake_amount)?;
        
        let transfer_msg = cw20::Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount,
        };

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_info.token_address.to_string(),
            msg: to_json_binary(&transfer_msg)?,
            funds: info.funds
        });

        Ok(Response::new().add_attribute("action", "agent unstake").add_message(msg))
    }

    pub fn distribute_rewards(
        deps: DepsMut,
        rewards_owner_addr: Addr,
        agent_addr_list: Vec<Addr>,
    ) -> Result<Response, ContractError> {
        let token_info = TOKEN_INFO.load(deps.storage)?;
        let rewards_owner_stake_amount = USER_STAKE.load(deps.storage, rewards_owner_addr.clone()).unwrap_or(Uint128::zero());
        let mut rewards_per_agent = rewards_owner_stake_amount / Uint128::from(agent_addr_list.len() as u128);
        let mut messages: Vec<CosmosMsg> = vec![];
        for agent_addr in agent_addr_list {
            // repay staked amount for agent
            let mut agent_stake_amount = AGENT_STAKE.load(deps.storage, agent_addr.clone()).unwrap_or(Uint128::zero());
            rewards_per_agent += agent_stake_amount;
            agent_stake_amount = Uint128::zero();
            AGENT_STAKE.save(deps.storage, agent_addr.clone(), &agent_stake_amount)?;

            // send rewards to agent
            let transfer_msg = cw20::Cw20ExecuteMsg::Transfer {
                recipient: agent_addr.to_string(),
                amount: rewards_per_agent,
            };

            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: token_info.token_address.to_string(),
                msg: to_json_binary(&transfer_msg)?,
                funds: vec![]
            });
            messages.push(msg);
        }
        Ok(Response::new().add_attribute("action", "distribute rewards").add_messages(messages))
    }    

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use std::usize;

    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::{App, BankKeeper, ContractWrapper, Executor, IntoAddr};

    use cw20::{Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg};
    use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
 
    use super::*;

    fn setup_cw20_contract(app: &mut App, admin: Addr) -> Addr {
        let cw20_code = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        let cw20_code_id = app.store_code(Box::new(cw20_code));

        let cw20_addr = app
            .instantiate_contract(
                cw20_code_id,
                admin.clone(),
                &Cw20InstantiateMsg {
                    name: "Test Token".to_string(),
                    symbol: "TTK".to_string(),
                    decimals: 6,
                    initial_balances: vec![Cw20Coin {
                        address: admin.to_string(),
                        amount: Uint128::new(1000000),
                    }],
                    mint: None,
                    marketing: None,
                },
                &[],
                "CW20 Test Token",
                None
            ).unwrap();

        cw20_addr
    }

    fn setup_agent_work_contract(app: &mut App, admin: Addr, cw20_addr: Addr) -> Addr {
        let agent_work_code = ContractWrapper::new(
            execute,
            instantiate,
            query,
        );
        let agent_work_code_id = app.store_code(Box::new(agent_work_code));
        
        let agent_work_addr = app
            .instantiate_contract(
                agent_work_code_id,
                admin.clone(),
                &InstantiateMsg {
                    token_symbol: "TTK".to_string(),
                    token_contract_addr: cw20_addr.clone(),
                },
                &[],
                "Agent Work",
                None
            ).unwrap();

        agent_work_addr
    }
 
    #[test]
    fn test_user_stake() {
        let mut app = App::default();
        let admin = app.api().addr_make("admin");
        let user1 = app.api().addr_make("user1");
        let user2 = app.api().addr_make("user2");
        let agent1 = app.api().addr_make("agent1");
        let agent2 = app.api().addr_make("agent2");
        let agent3 = app.api().addr_make("agent3");

        // set up cw20 contract
        let cw20_addr = setup_cw20_contract(&mut app, admin.clone());
        // set up agent work contract
        let agent_work_addr = setup_agent_work_contract(&mut app, admin.clone(), cw20_addr.clone());

    
        // send TTK to user1 and user2
        app.execute_contract(
            admin.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::Transfer { 
                recipient: user1.to_string(), 
                amount: Uint128::new(500), 
            } ,
            &[]
        ).unwrap();
        app.execute_contract(
            admin.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::Transfer { 
                recipient: user2.to_string(), 
                amount: Uint128::new(500), 
            } ,
            &[]
        ).unwrap();

        // user1 give allowance and stake 100 TTK
        app.execute_contract(
            user1.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: agent_work_addr.to_string(),
                amount: Uint128::new(100),
                expires: None,
            },
            &[]
        ).unwrap();
        let response = app.execute_contract(
            user1.clone(),
            agent_work_addr.clone(),
            &ExecuteMsg::UserStake { 
                amount: Uint128::new(100),
            },
            &[]
        ).unwrap();        
        assert!(response.events.iter().any(|e| e.ty == "wasm" && e.attributes.iter().any(|attr| attr.key == "action" && attr.value == "user stake")));


    }
}
