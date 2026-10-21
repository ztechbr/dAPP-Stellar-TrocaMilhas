#!/usr/bin/env bash
set -euo pipefail
: "${ADMIN_ADDRESS:?Set ADMIN_ADDRESS to your funded Stellar account}"
: "${ADMIN_SECRET:?Set ADMIN_SECRET (testnet only)}"
stellar keys add motoclube-admin --secret-key "$ADMIN_SECRET" --network testnet
stellar contract build
stellar contract deploy --wasm target/wasm32v1-none/release/motoclube_coin.wasm --source motoclube-admin --network testnet > contract_id.txt
CONTRACT_ID=$(cat contract_id.txt)
stellar contract invoke --id "$CONTRACT_ID" --source motoclube-admin --network testnet -- initialize --admin "$ADMIN_ADDRESS" --cap 1000000
printf 'Contract ID: %s\n' "$CONTRACT_ID"
