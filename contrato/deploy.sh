#!/usr/bin/env bash
set -euo pipefail
: "${ADMIN_ADDRESS:?Required: public G address}"
: "${CAP:=1000000}"
: "${NETWORK:=testnet}"
: "${SOURCE_ALIAS:=motoclube-admin}"
cd "$(dirname "$0")"
stellar contract build
stellar contract deploy --wasm target/wasm32v1-none/release/motoclube_coin.wasm --source "$SOURCE_ALIAS" --network "$NETWORK" > contract_id.txt
CONTRACT_ID="$(cat contract_id.txt)"
stellar contract invoke --id "$CONTRACT_ID" --source "$SOURCE_ALIAS" --network "$NETWORK" -- initialize --admin "$ADMIN_ADDRESS" --cap "$CAP"
printf 'CONTRACT_ID=%s\n' "$CONTRACT_ID"
