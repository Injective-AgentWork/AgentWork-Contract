use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};  

#[cw_serde]
pub struct TokenInfo {
    pub token_denom: String,
    pub token_address: Addr,
}
pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");

pub const USER_STAKE: Map<Addr, Uint128> = Map::new("user_stake");

pub const AGENT_STAKE: Map<Addr, Uint128> = Map::new("agent_stake");

pub const ACCPECT_VOTE: Item<Uint128> = Item::new("accpect_vote");
pub const REJECT_VOTE: Item<Uint128> = Item::new("reject_vote");
pub const IS_JUROR_VOTED: Map<Addr, bool> = Map::new("is_juror_voted");