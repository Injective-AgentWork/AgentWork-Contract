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
# CODE_ID = 26518
# INJ_ADDRESS = inj1z0ax5ypjskzhcsxhdz6sh5twvjdc6e4ta4f3rq
INIT='{"token_symbol": "AWT", "token_contract_addr": "inj1wp6x43895dewtfugkv08tvu7ajmvthzvel5mwn"}'
yes 12345678 | injectived tx wasm instantiate $CODE_ID "$INIT" \
--label="Instantiate Injective Agent Work" \
--from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj \
--gas=2000000 \
--no-admin \
--node=https://testnet.sentry.tm.injective.network:443

# Query balance of specific address
BALANCE_QUERY='{"balance": {"address": "inj1z0ax5ypjskzhcsxhdz6sh5twvjdc6e4ta4f3rq"}}'
injectived query wasm contract-state smart inj1wp6x43895dewtfugkv08tvu7ajmvthzvel5mwn "$BALANCE_QUERY" \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json 

# Query allowance of user for contract
ALLOWANCE_QUERY='{"allowance": {"owner": "inj1z0ax5ypjskzhcsxhdz6sh5twvjdc6e4ta4f3rq", "spender": "inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc"}}'
injectived query wasm contract-state smart inj1wp6x43895dewtfugkv08tvu7ajmvthzvel5mwn "$ALLOWANCE_QUERY" \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json 

# Increase allowance of user for contract
INCREASE_ALLOWANCE='{"increase_allowance":{"spender": "inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc", "amount":"100", "expires": null}}'
yes 12345678 | injectived tx wasm execute inj1wp6x43895dewtfugkv08tvu7ajmvthzvel5mwn "$INCREASE_ALLOWANCE" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# User stake to contract
USER_STAKE='{"user_stake":{"amount":"100", "job_id": "1"}}'
yes 12345678 | injectived tx wasm execute inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc "$USER_STAKE" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# Get user staked amount
GET_USER_STAKE='{"get_user_stake": {"user_addr": "inj1z0ax5ypjskzhcsxhdz6sh5twvjdc6e4ta4f3rq", "job_id": "1"}}'
injectived query wasm contract-state smart inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc "$GET_USER_STAKE" \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json 

# User unstake from contract
USER_UNSTAKE='{"user_unstake":{"amount":"50", "job_id": "1"}}'
yes 12345678 | injectived tx wasm execute inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc "$USER_UNSTAKE" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# AGENT_INJ_ADDRESS = inj1met6ppqdxvu4y6r2lf68uphty85hnz46qcv04u
# Increase allowance for Agent1
INCREASE_ALLOWANCE='{"increase_allowance":{"spender": "inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc", "amount":"20", "expires": null}}'
yes 12345678 | injectived tx wasm execute inj1wp6x43895dewtfugkv08tvu7ajmvthzvel5mwn "$INCREASE_ALLOWANCE" --from=$(echo $AGENT_INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# Get allowance for Agent1
ALLOWANCE_QUERY='{"allowance": {"owner": "inj1met6ppqdxvu4y6r2lf68uphty85hnz46qcv04u", "spender": "inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc"}}'
injectived query wasm contract-state smart inj1wp6x43895dewtfugkv08tvu7ajmvthzvel5mwn "$ALLOWANCE_QUERY" \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json 

# Agent stake to contract
AGENT_STAKE='{"agent_stake":{"amount":"20", "job_id": "1", "cost_per_unit_time": "10"}}'
yes 12345678 | injectived tx wasm execute inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc "$AGENT_STAKE" --from=$(echo $AGENT_INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# Get agent staked amount
GET_AGENT_STAKE='{"get_agent_stake": {"agent_addr": "inj1met6ppqdxvu4y6r2lf68uphty85hnz46qcv04u", "job_id": "1"}}'
injectived query wasm contract-state smart inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc "$GET_AGENT_STAKE" \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json 

# Agent unstake to contract
AGENT_UNSTAKE='{"agent_unstake":{"amount":"20", "job_id": "1"}}'
yes 12345678 | injectived tx wasm execute inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc "$AGENT_UNSTAKE" --from=$(echo $AGENT_INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# Get number of agents
GET_NUM_OF_AGENT='{"get_num_of_agent": {"job_id": "1"}}'
injectived query wasm contract-state smart inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc "$GET_NUM_OF_AGENT" \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# distribute rewards from User to Agent by number of agents 
# In case the task has a finite time -> the rewards is divided equally among the participating agents
DISTRIBUTE_REWARDS_BY_AGENT='{"distribute_rewards_by_agent":{"job_id": "1"}}'
yes 12345678 | injectived tx wasm execute inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc "$DISTRIBUTE_REWARDS_BY_AGENT" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# distribute rewards by unit time
# In case the task has a infinite time and the rewards is divided based on the cost of each agent and is called every time unit until the money in the pool exhausted
DISTRIBUTE_REWARDS_BY_TIME='{"distribute_rewards_by_time":{"job_id": "1"}}'
yes 12345678 | injectived tx wasm execute inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc "$DISTRIBUTE_REWARDS_BY_TIME" --from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# Agent Juror role vote accept or Reject
JUROR_VOTE='{"juror_vote":{"is_accept": "true"}}'
yes 12345678 | injectived tx wasm execute inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc "$JUROR_VOTE" --from=$(echo $AGENT_INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json

# Get Vote result 
GetVoteResult='{}'
injectived query wasm contract-state smart inj12xjw4pkv2trn5kah8lmu7a3ygprpu5q002egwc "$GetVoteResult" \
--node=https://testnet.sentry.tm.injective.network:443 \
--output json 