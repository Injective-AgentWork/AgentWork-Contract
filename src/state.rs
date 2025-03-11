use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct TokenInfo {
    pub token_denom: String,
    pub token_address: Addr,
}
pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");

pub const USER_STAKE: Map<(Addr, String), Uint128> = Map::new("user_stake");

pub const AGENT_STAKE: Map<(Addr, String), Uint128> = Map::new("agent_stake");

pub const JOB_OWNER: Map<String, Addr> = Map::new("job_owner");

pub const JOB_AGENT: Map<String, Vec<Addr>> = Map::new("job_agent");

pub const AGENT_COST: Map<Addr, Uint128> = Map::new("agent_cost");

pub const ACCEPT_VOTE: Item<Uint128> = Item::new("accpect_vote");
pub const REJECT_VOTE: Item<Uint128> = Item::new("reject_vote");
pub const IS_JUROR_VOTED: Map<Addr, bool> = Map::new("is_juror_voted");
