# Instalação e deploy, Stellar Testnet

Instale Rust com rustup, adicione `rustup target add wasm32v1-none` e instale a CLI Stellar seguindo https://developers.stellar.org/docs/tools/cli . Verifique a versão compatível da CLI e do Soroban SDK antes de compilar.

No diretório contrato:

```bash
export ADMIN_ADDRESS=G... # endereço público com XLM de teste
export ADMIN_SECRET=S... # SOMENTE chave descartável de testnet
bash deploy.sh
```

Guarde `contract_id.txt`. `allocate` cria pontos dentro do teto, `transfer` move pontos entre contas e `balance` consulta o saldo. Quantidades são inteiras, sem casas decimais. O teto é 1.000.000 pontos, a emissão é gradual. As contas precisam ter XLM de teste para taxas. O contrato é uma prova de conceito, sem auditoria e sem extensão automática de TTL de storage; não use em mainnet sem implementar renovação de TTL, eventos, testes e controles de governança.
