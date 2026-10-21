# Deploy Soroban em Stellar Testnet

Requisitos: Rust, Stellar CLI, `rustup target add wasm32v1-none`. Consulte https://developers.stellar.org/docs/tools/cli e https://developers.stellar.org/docs/build/smart-contracts/getting-started.

```bash
rustup target add wasm32v1-none
stellar keys generate motoclube-admin --network testnet --fund
export ADMIN_ADDRESS=$(stellar keys address motoclube-admin)
cd contrato
cargo test
./deploy.sh
```

Para outra rede, configure `NETWORK` e use alias com saldo suficiente. A distribuição inicial é feita com `allocate`, não no deploy.

```bash
export CONTRACT_ID=$(cat contract_id.txt)
stellar contract invoke --id "$CONTRACT_ID" --source motoclube-admin --network testnet -- allocate --to GDESTINO... --amount 100
stellar contract invoke --id "$CONTRACT_ID" --source motoclube-admin --network testnet -- balance --user GDESTINO...
```

Nunca coloque a chave secreta no código, APK ou repositório.
