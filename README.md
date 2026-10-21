# MotoClube Coins, protótipo Stellar Soroban

- `contrato/`: contrato Rust Soroban, teto de 1.000.000 pontos, distribuição pelo administrador, transferências autorizadas e saldo.
- `admin/`: painel Flask + Tailwind para alocação via CLI Stellar.
- `android/`: projeto Android Studio Jetpack Compose com interface inicial. **A integração RPC e a assinatura de transferências no Android ainda não estão implementadas.** Botões indisponíveis intencionalmente para evitar operações simuladas apresentadas como reais.

## Instalação

1. Siga `contrato/INSTALACAO.md` para compilar e publicar em Testnet. Configure `ADMIN_ADDRESS` e `ADMIN_SECRET` antes do deploy.
2. Configure uma chave local `motoclube-admin` na CLI da máquina do Flask, preferencialmente via método seguro suportado pela CLI.
3. `cd admin && python -m venv .venv && source .venv/bin/activate && pip install -r requirements.txt`
4. `export CONTRACT_ID=C... ADMIN_SECRET=configured FLASK_SECRET=change-this-to-a-long-random-value` e `python app.py`.
5. Abra `android/` no Android Studio e sincronize o Gradle.

## Limitações de segurança e operação

Este é um MVP técnico incompleto, apenas para Testnet. A tela Android não está ligada à blockchain. O painel administrativo deve permanecer local, sem acesso público, pois ainda não possui autenticação e proteção CSRF. A moeda não é um Stellar Asset Contract padrão SEP-41. Não há mecanismo de resgate, eventos, expiração renovada de storage ou auditoria. O SDK e CLI podem exigir ajustes de compatibilidade conforme suas versões. Não use fundos reais nem chaves de mainnet.
