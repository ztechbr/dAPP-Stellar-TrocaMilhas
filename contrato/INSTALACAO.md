# Deploy Soroban em Stellar Testnet

Requisitos: Rust, Stellar CLI (26.x) e o alvo `wasm32v1-none`. Consulte https://developers.stellar.org/docs/tools/cli e https://developers.stellar.org/docs/build/smart-contracts/getting-started.

## Windows (compile.bat)

Na raiz do repositório:

```bat
compile contrato   :: cargo test + stellar contract build
compile deploy     :: compila e publica na testnet
```

O `deploy` faz tudo sozinho:
1. Se o alias `motoclube-admin` não existir, gera a chave e a financia pelo Friendbot (só na testnet).
2. Usa o endereço desse alias como administrador, a menos que você defina `ADMIN_ADDRESS`.
3. Publica o contrato passando `--admin` e `--cap` ao **construtor**, na mesma transação do deploy.
4. Salva o ID em `contrato\contract_id.txt` e atualiza `CONTRACT_ID` em `admin\.env`, se esse arquivo existir.

Para mudar os padrões, defina as variáveis antes de rodar:

```bat
set CAP=500000
set NETWORK=testnet
set SOURCE_ALIAS=motoclube-admin
compile deploy
```

## Linux/macOS

```bash
rustup target add wasm32v1-none
stellar keys generate motoclube-admin --network testnet --fund
cd contrato
./deploy.sh
```

## Após o deploy

O deploy não distribui moedas. Elas entram em circulação com `allocate`:

```bash
stellar contract invoke --id motoclube_coin --source motoclube-admin --network testnet -- allocate --to GDESTINO... --amount 100
stellar contract invoke --id motoclube_coin --source motoclube-admin --network testnet -- balance --user GDESTINO...
stellar contract invoke --id motoclube_coin --source motoclube-admin --network testnet -- remaining
```

(`motoclube_coin` é o alias local que o deploy registra para o ID `C...`.)

Nunca coloque a chave secreta no código, no APK ou no repositório.
