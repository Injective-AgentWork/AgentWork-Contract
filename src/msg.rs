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
        job_id: Uint128,
    },
    UserUnstake {
        amount: Uint128,
        job_id: Uint128,
    },
    AgentStake {
        amount: Uint128,
        job_id: Uint128,
        cost_per_unit_time: Uint128,
    },
    AgentUnstake {
        amount: Uint128,
        job_id: Uint128,
    },
    DistributeRewardsByAgent {
        job_id: Uint128
    },
    DistributeRewardsByTime {
        job_id: Uint128
    },
    JurorVote {
        is_accept: bool,
    },
    ResetVote {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    GetUserStake { 
        user_addr: Addr,
        job_id: Uint128
    },

    #[returns(Uint128)]
    GetAgentStake { 
        agent_addr: Addr,
        job_id: Uint128
    },

    #[returns(TokenInfoResponse)]
    GetTokenInfo {},

    #[returns(bool)]
    CheckIfEnoughRewards {
        job_id: Uint128
    },

    #[returns(VoteResultResponse)]
    GetVoteResult {},
}

#[cw_serde]
pub struct VoteResultResponse {
    pub accept_vote: Uint128,
    pub reject_vote: Uint128,
}

#[cw_serde]
pub struct TokenInfoResponse {
    pub token_denom: String,
    pub token_address: Addr,
}
