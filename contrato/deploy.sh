#!/usr/bin/env bash
# =============================================================================
# deploy.sh - equivalente Linux/macOS do "compile.bat deploy".
#
# Compila o contrato e faz o deploy passando --admin e --cap ao construtor,
# na mesma transação, para que ninguém tome o controle do contrato entre o
# deploy e a inicialização.
#
#   ADMIN_ADDRESS  administrador (G...) ... padrão: endereço do SOURCE_ALIAS
#   CAP            teto de moedas ......... padrão: 1000000
#   NETWORK        rede Stellar ........... padrão: testnet
#   SOURCE_ALIAS   alias da chave na CLI .. padrão: motoclube-admin
# =============================================================================
set -euo pipefail
: "${CAP:=1000000}"
: "${NETWORK:=testnet}"
: "${SOURCE_ALIAS:=motoclube-admin}"
: "${ADMIN_ADDRESS:=$(stellar keys address "$SOURCE_ALIAS")}"
cd "$(dirname "$0")"

cargo test
stellar contract build
stellar contract deploy \
  --wasm target/wasm32v1-none/release/motoclube_coin.wasm \
  --source "$SOURCE_ALIAS" --network "$NETWORK" --alias motoclube_coin \
  -- --admin "$ADMIN_ADDRESS" --cap "$CAP" > contract_id.txt

printf 'CONTRACT_ID=%s\n' "$(cat contract_id.txt)"
