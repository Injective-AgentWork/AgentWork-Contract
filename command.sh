# build the contract
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.16.0

# upload the Wasm contract
# inside the "injective-core-staging" container, or from the contract directory if running injectived locally
yes 12345678 | injectived tx wasm store artifacts/injective_agent_work.wasm \
--from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=3000000 \
--node=https://testnet.sentry.tm.injective.network:443

# Instantiate the contract
# CODE_ID = 24742
# INJ_ADDRESS = inj1z0ax5ypjskzhcsxhdz6sh5twvjdc6e4ta4f3rq
INIT='{"token_symbol": "MYT", "token_contract_addr": "inj1mgqj43w6f7pfqqfaa9t29gph6gje368ydzwvnc"}'
yes 12345678 | injectived tx wasm instantiate $CODE_ID "$INIT" \
--label="Instantiate Injective Agent Work" \
--from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj \
--gas=2000000 \
--no-admin \
--node=https://testnet.sentry.tm.injective.network:443

# User stake to contract
USER_STAKE='{"user_stake":{"amount":"100"}}'
yes 12345678 | injectived tx wasm execute inj1wn20cgalqtv84e58xa2k2a902p6mm6fly5lsg8 "$USER_STAKE" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# Get user staked amount
GET_USER_STAKE='{"get_user_stake": {"user_addr": "inj1z0ax5ypjskzhcsxhdz6sh5twvjdc6e4ta4f3rq"}}'
injectived query wasm contract-state smart inj1wn20cgalqtv84e58xa2k2a902p6mm6fly5lsg8 "$GET_USER_STAKE" \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json 

# User unstake from contract
USER_UNSTAKE='{"user_unstake":{"amount":"50"}}'
yes 12345678 | injectived tx wasm execute inj1wn20cgalqtv84e58xa2k2a902p6mm6fly5lsg8 "$USER_UNSTAKE" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# AGENT_INJ_ADDRESS = inj19r9a7jxj5d2mh7487a4rtn32d6l5mq5zf6tm2g
# Increase allowance for Agent1
INCREASE_ALLOWANCE='{"increase_allowance":{"spender": "inj1wn20cgalqtv84e58xa2k2a902p6mm6fly5lsg8", "amount":"20", "expires": null}}'
yes 12345678 | injectived tx wasm execute inj1mgqj43w6f7pfqqfaa9t29gph6gje368ydzwvnc "$INCREASE_ALLOWANCE" --from=$(echo $AGENT_INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# Get allowance for Agent1
ALLOWANCE_QUERY='{"allowance": {"owner": "inj19r9a7jxj5d2mh7487a4rtn32d6l5mq5zf6tm2g", "spender": "inj1wn20cgalqtv84e58xa2k2a902p6mm6fly5lsg8"}}'
injectived query wasm contract-state smart inj1mgqj43w6f7pfqqfaa9t29gph6gje368ydzwvnc "$ALLOWANCE_QUERY" \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json 

# Agent stake to contract
AGENT_STAKE='{"agent_stake":{"amount":"20"}}'
yes 12345678 | injectived tx wasm execute inj1wn20cgalqtv84e58xa2k2a902p6mm6fly5lsg8 "$AGENT_STAKE" --from=$(echo $AGENT_INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# Get agent staked amount
GET_AGENT_STAKE='{"get_agent_stake": {"agent_addr": "inj19r9a7jxj5d2mh7487a4rtn32d6l5mq5zf6tm2g"}}'
injectived query wasm contract-state smart inj1wn20cgalqtv84e58xa2k2a902p6mm6fly5lsg8 "$GET_AGENT_STAKE" \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json 

# Agent unstake to contract
AGENT_UNSTAKE='{"agent_unstake":{"amount":"10"}}'
yes 12345678 | injectived tx wasm execute inj1wn20cgalqtv84e58xa2k2a902p6mm6fly5lsg8 "$AGENT_UNSTAKE" --from=$(echo $AGENT_INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# distribute rewards from User to Agent by number of agents 
# In case the task has a finite time -> the rewards is divided equally among the participating agents
DISTRIBUTE_REWARDS_BY_AGENT='{"distribute_rewards_by_agent":{"rewards_owner_addr": "inj1z0ax5ypjskzhcsxhdz6sh5twvjdc6e4ta4f3rq", "agent_addr_list": ["inj19r9a7jxj5d2mh7487a4rtn32d6l5mq5zf6tm2g"]}}'
yes 12345678 | injectived tx wasm execute inj1wn20cgalqtv84e58xa2k2a902p6mm6fly5lsg8 "$DISTRIBUTE_REWARDS_BY_AGENT" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# distribute rewards by unit time
# In case the task has a infinite time and the rewards is divided based on the cost of each agent and is called every time unit until the money in the pool exhausted
DISTRIBUTE_REWARDS_BY_TIME='{"distribute_rewards_by_time":{"rewards_owner_addr": "inj1z0ax5ypjskzhcsxhdz6sh5twvjdc6e4ta4f3rq", "agent_list": [{"addr": "inj19r9a7jxj5d2mh7487a4rtn32d6l5mq5zf6tm2g", "cost_per_unit_time": "15"}]}}'
yes 12345678 | injectived tx wasm execute inj1wn20cgalqtv84e58xa2k2a902p6mm6fly5lsg8 "$DISTRIBUTE_REWARDS_BY_TIME" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# Agent Juror role vote Accpect or Reject
JUROR_VOTE='{"juror_vote":{"is_accpect": "true"}}'
yes 12345678 | injectived tx wasm execute inj1wn20cgalqtv84e58xa2k2a902p6mm6fly5lsg8 "$JUROR_VOTE" --from=$(echo $AGENT_INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# Get Vote result 
GetVoteResult='{}'
injectived query wasm contract-state smart inj1wn20cgalqtv84e58xa2k2a902p6mm6fly5lsg8 "$GetVoteResult" \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json 