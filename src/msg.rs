use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub token_symbol: String,
    pub token_contract_addr: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    UserStake {
        amount: Uint128,
    },
    UserUnstake {
        amount: Uint128,
    },
    AgentStake {
        amount: Uint128,
    },
    AgentUnstake {
        amount: Uint128,
    },
    DistributeRewards {
        rewards_owner_addr: Addr,
        agent_addr_list: Vec<Addr>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    GetUserStake {
        user_addr: Addr,
    },

    #[returns(Uint128)]
    GetAgentStake {
        agent_addr: Addr,
    },

    #[returns(TokenInfoResponse)]
    GetTokenInfo {},
}

#[cw_serde]
pub struct TokenInfoResponse {
    pub token_denom: String,
    pub token_address: Addr,
}
