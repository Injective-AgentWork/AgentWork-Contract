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

        Ok(Response::new().add_attribute("method", "user stake").add_message(msg))
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

        Ok(Response::new().add_attribute("method", "user unstake").add_message(msg))
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

        Ok(Response::new().add_attribute("method", "agent stake").add_message(msg))
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

        Ok(Response::new().add_attribute("method", "agent unstake").add_message(msg))
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
        Ok(Response::new().add_messages(messages))
    }

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
