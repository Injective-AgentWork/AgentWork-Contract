#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VoteResultResponse};
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
        ExecuteMsg::UserStake { amount, job_id} => execute::user_stake(deps, env, info, amount, job_id),
        ExecuteMsg::UserUnstake { amount, job_id} => execute::user_unstake(deps, info, amount, job_id),
        ExecuteMsg::AgentStake { amount, job_id, cost_per_unit_time} => execute::agent_stake(deps, env, info, amount, job_id, cost_per_unit_time),
        ExecuteMsg::AgentUnstake { amount, job_id} => execute::agent_unstake(deps, info, amount, job_id),
        ExecuteMsg::DistributeRewardsByAgent {job_id} => execute::distribute_rewards_by_agent(deps, job_id),
        ExecuteMsg::DistributeRewardsByTime {job_id} => execute::distribute_rewards_by_time(deps, job_id),
        ExecuteMsg::JurorVote { is_accept } => execute::juror_vote(deps, info, is_accept),
        ExecuteMsg::ResetVote {} => execute::reset_vote(deps),
    }
}

pub mod execute {
    use super::*;
    use cosmwasm_std::{to_json_binary, CosmosMsg, WasmMsg};

    pub fn user_stake(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
        job_id: Uint128
    ) -> Result<Response, ContractError> {
        if let Some(job_owner) = JOB_OWNER.may_load(deps.storage, job_id.to_string())? {
            if job_owner != info.sender {
                return Err(ContractError::NotJobOwner {});
            }
        } else {
            JOB_OWNER.save(deps.storage, job_id.to_string(), &info.sender)?;
        }
        

        let token_info = TOKEN_INFO.load(deps.storage)?;
        let mut user_stake_amount = USER_STAKE
            .load(deps.storage, (info.sender.clone(), job_id.to_string()))
            .unwrap_or(Uint128::zero());
        user_stake_amount += amount;
        USER_STAKE.save(deps.storage, (info.sender.clone(), job_id.to_string()), &user_stake_amount)?;

        let transfer_from_msg = cw20::Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount,
        };

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_info.token_address.to_string(),
            msg: to_json_binary(&transfer_from_msg)?,
            funds: info.funds,
        });

        Ok(Response::new()
            .add_attribute("action", "user stake")
            .add_message(msg))
    }

    pub fn user_unstake(
        deps: DepsMut,
        info: MessageInfo,
        amount: Uint128,
        job_id: Uint128
    ) -> Result<Response, ContractError> {
        let job_owner = JOB_OWNER.load(deps.storage, job_id.to_string()).unwrap_or(info.sender.clone());
        if job_owner != info.sender {
            return Err(ContractError::NotJobOwner {});
        }

        let token_info = TOKEN_INFO.load(deps.storage)?;
        let mut user_stake_amount = USER_STAKE
            .load(deps.storage, (info.sender.clone(), job_id.to_string()))
            .unwrap_or(Uint128::zero());
        if user_stake_amount < amount {
            return Err(ContractError::InsufficientStake {});
        } else {
            user_stake_amount -= amount;
        };
        USER_STAKE.save(deps.storage, (info.sender.clone(), job_id.to_string()), &user_stake_amount)?;
        

        let transfer_msg = cw20::Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount,
        };

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_info.token_address.to_string(),
            msg: to_json_binary(&transfer_msg)?,
            funds: info.funds,
        });

        Ok(Response::new()
            .add_attribute("action", "user unstake")
            .add_message(msg))
    }

    pub fn agent_stake(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
        job_id: Uint128,
        cost_per_unit_time: Uint128
    ) -> Result<Response, ContractError> {
        let token_info = TOKEN_INFO.load(deps.storage)?;
        let mut agent_stake_amount = AGENT_STAKE
            .load(deps.storage, (info.sender.clone(), job_id.to_string()))
            .unwrap_or(Uint128::zero());
        agent_stake_amount += amount;
        AGENT_STAKE.save(deps.storage, (info.sender.clone(), job_id.to_string()), &agent_stake_amount)?;
        JOB_AGENT.update(deps.storage, job_id.to_string(), |agents| -> StdResult<_> {
            let mut agents = agents.unwrap_or(vec![]);
            agents.push(info.sender.clone());
            Ok(agents)
        })?;
        AGENT_COST.save(deps.storage, info.sender.clone(), &cost_per_unit_time)?;

        let transfer_from_msg = cw20::Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount,
        };

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_info.token_address.to_string(),
            msg: to_json_binary(&transfer_from_msg)?,
            funds: info.funds,
        });

        Ok(Response::new()
            .add_attribute("action", "agent stake")
            .add_message(msg))
    }

    pub fn agent_unstake(
        deps: DepsMut,
        info: MessageInfo,
        amount: Uint128,
        job_id: Uint128
    ) -> Result<Response, ContractError> {
        let token_info = TOKEN_INFO.load(deps.storage)?;
        let mut agent_stake_amount = AGENT_STAKE
            .load(deps.storage, (info.sender.clone(), job_id.to_string()))
            .unwrap_or(Uint128::zero());
        if agent_stake_amount < amount {
            return Err(ContractError::InsufficientStake {});
        } else {
            agent_stake_amount -= amount;
        };
        AGENT_STAKE.save(deps.storage, (info.sender.clone(), job_id.to_string()), &agent_stake_amount)?;
        JOB_AGENT.update(deps.storage, job_id.to_string(), |agents| -> StdResult<_> {
            let mut agents = agents.unwrap_or(vec![]);
            agents.retain(|agent| agent != &info.sender);
            Ok(agents)
        })?;

        let transfer_msg = cw20::Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount,
        };

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_info.token_address.to_string(),
            msg: to_json_binary(&transfer_msg)?,
            funds: info.funds,
        });

        Ok(Response::new()
            .add_attribute("action", "agent unstake")
            .add_message(msg))
    }

    pub fn distribute_rewards_by_agent(
        deps: DepsMut,
        job_id: Uint128
    ) -> Result<Response, ContractError> {
        let token_info = TOKEN_INFO.load(deps.storage)?;
        let job_owner_addr = JOB_OWNER.load(deps.storage, job_id.to_string()).unwrap();
        let mut rewards_owner_stake_amount = USER_STAKE
            .load(deps.storage, (job_owner_addr.clone(), job_id.to_string()))
            .unwrap_or(Uint128::zero());
        let job_agent_addrs = JOB_AGENT.load(deps.storage, job_id.to_string()).unwrap();
        let rewards_per_agent =
            rewards_owner_stake_amount / Uint128::from(job_agent_addrs.len() as u128);
        rewards_owner_stake_amount = Uint128::zero();
        USER_STAKE.save(
            deps.storage,
            (job_owner_addr.clone(), job_id.to_string()),
            &rewards_owner_stake_amount,
        )?;
        let mut messages: Vec<CosmosMsg> = vec![];
        for agent_addr in job_agent_addrs {
            // repay staked amount for agent
            let mut agent_stake_amount = AGENT_STAKE
                .load(deps.storage, ((agent_addr).clone(), job_id.to_string()))
                .unwrap_or(Uint128::zero());
            let repay_amount = agent_stake_amount + rewards_per_agent;
            agent_stake_amount = Uint128::zero();
            AGENT_STAKE.save(deps.storage, (agent_addr.clone(), job_id.to_string()), &agent_stake_amount)?;

            // send rewards to agent
            let transfer_msg = cw20::Cw20ExecuteMsg::Transfer {
                recipient: agent_addr.to_string(),
                amount: repay_amount,
            };

            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: token_info.token_address.to_string(),
                msg: to_json_binary(&transfer_msg)?,
                funds: vec![],
            });
            messages.push(msg);
        }
        Ok(Response::new()
            .add_attribute("action", "distribution rewards by agent")
            .add_messages(messages))
    }

    pub fn distribute_rewards_by_time(
        deps: DepsMut,
        job_id: Uint128
    ) -> Result<Response, ContractError> {
        let token_info = TOKEN_INFO.load(deps.storage)?;
        let job_owner_addr = JOB_OWNER.load(deps.storage, job_id.to_string()).unwrap();
        let mut rewards_owner_stake_amount = USER_STAKE
            .load(deps.storage, (job_owner_addr.clone(), job_id.to_string()))
            .unwrap_or(Uint128::zero());
        let job_agent_addrs = JOB_AGENT.load(deps.storage, job_id.to_string()).unwrap(); 
        if !query::check_if_enough_rewards(
            deps.as_ref(),
            job_id
        ) {
            return Err(ContractError::InsufficientStake {});
        }
        let mut messages: Vec<CosmosMsg> = vec![];
        let mut total_cost_per_unit_time = Uint128::zero();
        for agent_addr in job_agent_addrs {
            let agent_cost = AGENT_COST.load(deps.storage, agent_addr.clone()).unwrap();
            total_cost_per_unit_time += agent_cost;

            // send rewards to agent
            let transfer_msg = cw20::Cw20ExecuteMsg::Transfer {
                recipient: agent_addr.to_string(),
                amount: agent_cost,
            };

            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: token_info.token_address.to_string(),
                msg: to_json_binary(&transfer_msg)?,
                funds: vec![],
            });
            messages.push(msg);
        }
        rewards_owner_stake_amount -= total_cost_per_unit_time;
        USER_STAKE.save(
            deps.storage,
            (job_owner_addr.clone(), job_id.to_string()),
            &rewards_owner_stake_amount,
        )?;
        Ok(Response::new()
            .add_attribute("action", "distribution rewards by time unit")
            .add_messages(messages))
    }

    pub fn juror_vote(
        deps: DepsMut,
        info: MessageInfo,
        is_accept: bool,
    ) -> Result<Response, ContractError> {
        let accept_vote = ACCEPT_VOTE.load(deps.storage).unwrap_or(Uint128::zero());
        let reject_vote = REJECT_VOTE.load(deps.storage).unwrap_or(Uint128::zero());
        let is_juror_voted = IS_JUROR_VOTED
            .load(deps.storage, info.sender.clone())
            .unwrap_or(false);
        if is_juror_voted {
            return Err(ContractError::AlreadyVoted {});
        } else {
            IS_JUROR_VOTED.save(deps.storage, info.sender.clone(), &true)?;
        }
        if is_accept {
            ACCEPT_VOTE.save(deps.storage, &(accept_vote + Uint128::new(1)))?;
        } else {
            REJECT_VOTE.save(deps.storage, &(reject_vote + Uint128::new(1)))?;
        }
        Ok(Response::new().add_attribute("action", "juror vote"))
    }

    pub fn reset_vote(deps: DepsMut) -> Result<Response, ContractError> {
        ACCEPT_VOTE.save(deps.storage, &Uint128::zero())?;
        REJECT_VOTE.save(deps.storage, &Uint128::zero())?;
        IS_JUROR_VOTED.clear(deps.storage);
        Ok(Response::new().add_attribute("action", "reset vote"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUserStake { user_addr , job_id} => query::get_user_stake(deps, user_addr, job_id),
        QueryMsg::GetAgentStake { agent_addr , job_id} => query::get_agent_stake(deps, agent_addr, job_id),
        QueryMsg::GetTokenInfo {} => query::get_token_info(deps),
        QueryMsg::CheckIfEnoughRewards {job_id} => to_json_binary(&query::check_if_enough_rewards(
            deps,
            job_id
        )),
        QueryMsg::GetVoteResult {} => query::get_vote_result(deps),
    }
}

pub mod query {
    use super::*;

    pub fn get_user_stake(deps: Deps, user_addr: Addr, job_id: Uint128) -> StdResult<Binary> {
        let user_stake_amount = USER_STAKE
            .load(deps.storage, (user_addr, job_id.to_string()))
            .unwrap_or(Uint128::zero());
        to_json_binary(&user_stake_amount)
    }

    pub fn get_agent_stake(deps: Deps, agent_addr: Addr, job_id: Uint128) -> StdResult<Binary> {
        let agent_stake_amount = AGENT_STAKE
            .load(deps.storage, (agent_addr, job_id.to_string()))
            .unwrap_or(Uint128::zero());
        to_json_binary(&agent_stake_amount)
    }

    pub fn get_token_info(deps: Deps) -> StdResult<Binary> {
        let token_info = TOKEN_INFO.load(deps.storage)?;
        to_json_binary(&token_info)
    }

    pub fn check_if_enough_rewards(
        deps: Deps,
        job_id: Uint128
    ) -> bool {
        let job_owner_addr = JOB_OWNER.load(deps.storage, job_id.to_string()).unwrap();
        let job_owner_addrs = JOB_AGENT.load(deps.storage, job_id.to_string()).unwrap();
        let rewards_owner_stake_amount = USER_STAKE
            .load(deps.storage, (job_owner_addr, job_id.to_string()))
            .unwrap_or(Uint128::zero());
        let mut total_cost_per_unit_time = Uint128::zero();
        for agent_addr in job_owner_addrs {
            let agent_cost = AGENT_COST.load(deps.storage, agent_addr.clone()).unwrap();
            total_cost_per_unit_time += agent_cost;
        }
        rewards_owner_stake_amount >= total_cost_per_unit_time
    }

    pub fn get_vote_result(deps: Deps) -> StdResult<Binary> {
        let accept_vote = ACCEPT_VOTE.load(deps.storage).unwrap_or(Uint128::zero());
        let reject_vote = REJECT_VOTE.load(deps.storage).unwrap_or(Uint128::zero());
        let vote_result = VoteResultResponse {
            accept_vote,
            reject_vote,
        };
        to_json_binary(&vote_result)
    }
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
                None,
            )
            .unwrap();

        cw20_addr
    }

    fn setup_agent_work_contract(app: &mut App, admin: Addr, cw20_addr: Addr) -> Addr {
        let agent_work_code = ContractWrapper::new(execute, instantiate, query);
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
                None,
            )
            .unwrap();

        agent_work_addr
    }

    fn allocate_token(
        app: &mut App,
        admin: Addr,
        cw20_addr: Addr,
        user1: Addr,
        user2: Addr,
        agent1: Addr,
        agent2: Addr,
        agent3: Addr,
    ) {
        // send 500 TTK to user1, user2, agent1, agent2, agent3
        app.execute_contract(
            admin.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::Transfer {
                recipient: user1.to_string(),
                amount: Uint128::new(500),
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            admin.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::Transfer {
                recipient: user2.to_string(),
                amount: Uint128::new(500),
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            admin.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::Transfer {
                recipient: agent1.to_string(),
                amount: Uint128::new(500),
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            admin.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::Transfer {
                recipient: agent2.to_string(),
                amount: Uint128::new(500),
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            admin.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::Transfer {
                recipient: agent3.to_string(),
                amount: Uint128::new(500),
            },
            &[],
        )
        .unwrap();
    }

    #[test]
    fn test_user_stake_and_unstake() {
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
        // allocate 500 TTK to user1 and user2
        allocate_token(
            &mut app,
            admin.clone(),
            cw20_addr.clone(),
            user1.clone(),
            user2.clone(),
            agent1.clone(),
            agent2.clone(),
            agent3.clone(),
        );

        // user1 give allowance and stake 200 TTK
        app.execute_contract(
            user1.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: agent_work_addr.to_string(),
                amount: Uint128::new(200),
                expires: None,
            },
            &[],
        )
        .unwrap();
        let response = app
            .execute_contract(
                user1.clone(),
                agent_work_addr.clone(),
                &ExecuteMsg::UserStake {
                    amount: Uint128::new(200),
                    job_id: Uint128::new(1),
                },
                &[],
            )
            .unwrap();
        assert!(response.events.iter().any(|e| e.ty == "wasm"
            && e.attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "user stake")));

        // check whether current user1 balance is 300 and user1 stake is 200
        let user1_balance: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                &cw20_addr,
                &Cw20QueryMsg::Balance {
                    address: user1.to_string(),
                },
            )
            .unwrap();
        assert_eq!(user1_balance.balance, Uint128::new(300));
        let user1_stake: Uint128 = app
            .wrap()
            .query_wasm_smart(
                &agent_work_addr,
                &QueryMsg::GetUserStake {
                    user_addr: user1.clone(),
                    job_id: Uint128::new(1),
                },
            )
            .unwrap();
        assert_eq!(user1_stake, Uint128::new(200));

        // user1 unstake 100
        let response = app
            .execute_contract(
                user1.clone(),
                agent_work_addr.clone(),
                &ExecuteMsg::UserUnstake {
                    amount: Uint128::new(100),
                    job_id: Uint128::new(1),
                },
                &[],
            )
            .unwrap();
        assert!(response.events.iter().any(|e| e.ty == "wasm"
            && e.attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "user unstake")));

        // check whether current user1 balance is 400 and user1 stake is 100
        let user1_balance: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                &cw20_addr,
                &Cw20QueryMsg::Balance {
                    address: user1.to_string(),
                },
            )
            .unwrap();
        assert_eq!(user1_balance.balance, Uint128::new(400));
        let user1_stake: Uint128 = app
            .wrap()
            .query_wasm_smart(
                &agent_work_addr,
                &QueryMsg::GetUserStake {
                    user_addr: user1.clone(),
                    job_id: Uint128::new(1),
                },
            )
            .unwrap();
        assert_eq!(user1_stake, Uint128::new(100));
    }

    #[test]
    fn test_agent_stake_and_unstake() {
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
        // allocate 500 TTK to user1, user2, agent1, agent2, agent3
        allocate_token(
            &mut app,
            admin.clone(),
            cw20_addr.clone(),
            user1.clone(),
            user2.clone(),
            agent1.clone(),
            agent2.clone(),
            agent3.clone(),
        );

        // agent1 give allowance and stake 200 TTK
        app.execute_contract(
            agent1.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: agent_work_addr.to_string(),
                amount: Uint128::new(200),
                expires: None,
            },
            &[],
        )
        .unwrap();
        let response = app
            .execute_contract(
                agent1.clone(),
                agent_work_addr.clone(),
                &ExecuteMsg::AgentStake {
                    amount: Uint128::new(200),
                    job_id: Uint128::new(1),
                    cost_per_unit_time: Uint128::new(10),
                },
                &[],
            )
            .unwrap();
        assert!(response.events.iter().any(|e| e.ty == "wasm"
            && e.attributes
                .iter()
                .any(|attr| attr.key == "action" && attr.value == "agent stake")));

        // check whether current agent1 balance is 300 and agent1 stake is 200
        let agent1_balance: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                &cw20_addr,
                &Cw20QueryMsg::Balance {
                    address: agent1.to_string(),
                },
            )
            .unwrap();
        assert_eq!(agent1_balance.balance, Uint128::new(300));
        let agent1_stake: Uint128 = app
            .wrap()
            .query_wasm_smart(
                &agent_work_addr,
                &QueryMsg::GetAgentStake {
                    agent_addr: agent1.clone(),
                    job_id: Uint128::new(1),
                },
            )
            .unwrap();
        assert_eq!(agent1_stake, Uint128::new(200));

        // agent1 unstake 100
        let response = app
            .execute_contract(
                agent1.clone(),
                agent_work_addr.clone(),
                &ExecuteMsg::AgentUnstake {
                    amount: Uint128::new(100),
                    job_id: Uint128::new(1),
                },
                &[],
            )
            .unwrap();

        // check whether current agent1 balance is 400 and agent1 stake is 100
        let agent1_balance: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                &cw20_addr,
                &Cw20QueryMsg::Balance {
                    address: agent1.to_string(),
                },
            )
            .unwrap();
        assert_eq!(agent1_balance.balance, Uint128::new(400));
        let agent1_stake: Uint128 = app
            .wrap()
            .query_wasm_smart(
                &agent_work_addr,
                &QueryMsg::GetAgentStake {
                    agent_addr: agent1.clone(),
                    job_id: Uint128::new(1),
                },
            )
            .unwrap();
        assert_eq!(agent1_stake, Uint128::new(100));
    }

    #[test]
    fn test_distribute_rewards_by_agent() {
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
        // allocate 500 TTK to user1, user2, agent1, agent2, agent3
        allocate_token(
            &mut app,
            admin.clone(),
            cw20_addr.clone(),
            user1.clone(),
            user2.clone(),
            agent1.clone(),
            agent2.clone(),
            agent3.clone(),
        );

        // user1 give allowance and stake 100 TTK
        app.execute_contract(
            user1.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: agent_work_addr.to_string(),
                amount: Uint128::new(100),
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            user1.clone(),
            agent_work_addr.clone(),
            &ExecuteMsg::UserStake {
                amount: Uint128::new(100),
                job_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap();

        // agent1 give allowance and stake 10 TTK
        app.execute_contract(
            agent1.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: agent_work_addr.to_string(),
                amount: Uint128::new(10),
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            agent1.clone(),
            agent_work_addr.clone(),
            &ExecuteMsg::AgentStake {
                amount: Uint128::new(10),
                job_id: Uint128::new(1),
                cost_per_unit_time: Uint128::new(10),
            },
            &[],
        )
        .unwrap();

        // agent2 give allowance and stake 10 TTK
        app.execute_contract(
            agent2.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: agent_work_addr.to_string(),
                amount: Uint128::new(10),
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            agent2.clone(),
            agent_work_addr.clone(),
            &ExecuteMsg::AgentStake {
                amount: Uint128::new(10),
                job_id: Uint128::new(1),
                cost_per_unit_time: Uint128::new(10),
            },
            &[],
        )
        .unwrap();

        // agent3 give allowance and stake 10 TTK
        app.execute_contract(
            agent3.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: agent_work_addr.to_string(),
                amount: Uint128::new(10),
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            agent3.clone(),
            agent_work_addr.clone(),
            &ExecuteMsg::AgentStake {
                amount: Uint128::new(10),
                job_id: Uint128::new(1),
                cost_per_unit_time: Uint128::new(10),
            },
            &[],
        )
        .unwrap();

        // distribute rewards
        let response = app
            .execute_contract(
                user1.clone(),
                agent_work_addr.clone(),
                &ExecuteMsg::DistributeRewardsByAgent {
                    job_id: Uint128::new(1)
                },
                &[],
            )
            .unwrap();

        // check whether current agent3 balance is 533 and agent1 stake is 0
        let agent3_balance: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                &cw20_addr,
                &Cw20QueryMsg::Balance {
                    address: agent3.to_string(),
                },
            )
            .unwrap();
        assert_eq!(agent3_balance.balance, Uint128::new(533));
        let agent3_stake: Uint128 = app
            .wrap()
            .query_wasm_smart(
                &agent_work_addr,
                &QueryMsg::GetAgentStake {
                    agent_addr: agent3.clone(),
                    job_id: Uint128::new(1),
                },
            )
            .unwrap();
        assert_eq!(agent3_stake, Uint128::zero());
    }

    #[test]
    fn test_distribute_rewards_by_time() {
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
        // allocate 500 TTK to user1, user2, agent1, agent2, agent3
        allocate_token(
            &mut app,
            admin.clone(),
            cw20_addr.clone(),
            user1.clone(),
            user2.clone(),
            agent1.clone(),
            agent2.clone(),
            agent3.clone(),
        );

        // user1 give allowance and stake 100 TTK
        app.execute_contract(
            user1.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: agent_work_addr.to_string(),
                amount: Uint128::new(100),
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            user1.clone(),
            agent_work_addr.clone(),
            &ExecuteMsg::UserStake {
                amount: Uint128::new(100),
                job_id: Uint128::new(1),
            },
            &[],
        )
        .unwrap();

        // agent1 give allowance and stake 10 TTK
        app.execute_contract(
            agent1.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: agent_work_addr.to_string(),
                amount: Uint128::new(10),
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            agent1.clone(),
            agent_work_addr.clone(),
            &ExecuteMsg::AgentStake {
                amount: Uint128::new(10),
                job_id: Uint128::new(1),
                cost_per_unit_time: Uint128::new(5),
            },
            &[],
        )
        .unwrap();

        // agent2 give allowance and stake 10 TTK
        app.execute_contract(
            agent2.clone(),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: agent_work_addr.to_string(),
                amount: Uint128::new(10),
                expires: None,
            },
            &[],
        )
        .unwrap();
        app.execute_contract(
            agent2.clone(),
            agent_work_addr.clone(),
            &ExecuteMsg::AgentStake {
                amount: Uint128::new(10),
                job_id: Uint128::new(1),
                cost_per_unit_time: Uint128::new(10),
            },
            &[],
        )
        .unwrap();

        let response = app
            .execute_contract(
                user1.clone(),
                agent_work_addr.clone(),
                &ExecuteMsg::DistributeRewardsByTime {
                    job_id: Uint128::new(1)
                },
                &[],
            )
            .unwrap();

        // check whether current user1 balance is 400 and user1 stake is 85
        let user1_balance: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                &cw20_addr,
                &Cw20QueryMsg::Balance {
                    address: user1.to_string(),
                },
            )
            .unwrap();
        assert_eq!(user1_balance.balance, Uint128::new(400));
        let user1_stake: Uint128 = app
            .wrap()
            .query_wasm_smart(
                &agent_work_addr,
                &QueryMsg::GetUserStake {
                    user_addr: user1.clone(),
                    job_id: Uint128::new(1),
                },
            )
            .unwrap();
        assert_eq!(user1_stake, Uint128::new(85));

        // check whether current agent1 balance is 495 and agent1 stake is 10
        let agent1_balance: cw20::BalanceResponse = app
            .wrap()
            .query_wasm_smart(
                &cw20_addr,
                &Cw20QueryMsg::Balance {
                    address: agent1.to_string(),
                },
            )
            .unwrap();
        assert_eq!(agent1_balance.balance, Uint128::new(495));
        let agent1_stake: Uint128 = app
            .wrap()
            .query_wasm_smart(
                &agent_work_addr,
                &QueryMsg::GetAgentStake {
                    agent_addr: agent1.clone(),
                    job_id: Uint128::new(1),
                },
            )
            .unwrap();
        assert_eq!(agent1_stake, Uint128::new(10));
    }

    #[test]
    fn test_juror_vote() {
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
        // allocate 500 TTK to user1, user2, agent1, agent2, agent3
        allocate_token(
            &mut app,
            admin.clone(),
            cw20_addr.clone(),
            user1.clone(),
            user2.clone(),
            agent1.clone(),
            agent2.clone(),
            agent3.clone(),
        );

        // Agent 1 Vote accept
        app.execute_contract(
            agent1.clone(),
            agent_work_addr.clone(),
            &ExecuteMsg::JurorVote { is_accept: true },
            &[],
        )
        .unwrap();

        // Agent 2 vote accept
        app.execute_contract(
            agent2.clone(),
            agent_work_addr.clone(),
            &ExecuteMsg::JurorVote { is_accept: true },
            &[],
        )
        .unwrap();

        // Agent 3 vote reject
        app.execute_contract(
            agent3.clone(),
            agent_work_addr.clone(),
            &ExecuteMsg::JurorVote { is_accept: false },
            &[],
        )
        .unwrap();

        let vote_result: VoteResultResponse = app
            .wrap()
            .query_wasm_smart(&agent_work_addr, &QueryMsg::GetVoteResult {})
            .unwrap();

        assert_eq!(vote_result.accept_vote, Uint128::new(2));
        assert_eq!(vote_result.reject_vote, Uint128::new(1));
    }
}
