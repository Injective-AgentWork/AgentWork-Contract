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
# CODE_ID = 24542
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

