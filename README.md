![MotoClube Recompensas Stellar, visão do contrato Soroban, painel Flask e aplicativo Android](assets/MOTOCLUBE_%20Recompensas%20Stellar.png)

# MotoClube Coins, Stellar Soroban (Testnet)

Projeto de referência para moedas de recompensa entre coordenadores e motociclistas. **Ainda não é um dApp de transferência completo no Android**: consulta real de saldos implementada por API Flask, mas assinatura de transferências exige integração futura com uma carteira Soroban. Nunca use em Mainnet sem auditoria.

## Estrutura
- `contrato/`: Rust Soroban, emissão limitada, saldos, transferências com `require_auth`, eventos, extensão de TTL e teste unitário.
- `admin/`: Flask/Tailwind, login com senha hash, CSRF, rate limiting, SQLite para histórico local e API pública de leitura.
- `android/`: Kotlin/Jetpack Compose, imagens de moedas douradas, consulta de saldo via API, validação básica de endereços.

## 1. Contrato
Instale Rust, target `wasm32v1-none` e Stellar CLI compatível. Veja `contrato/INSTALACAO.md`. Crie um alias local `motoclube-admin` com chave Testnet. O deploy não solicita segredos como argumentos de processo.

```bash
cd contrato
stellar keys generate motoclube-admin --network testnet --fund
stellar keys address motoclube-admin
export ADMIN_ADDRESS=$(stellar keys address motoclube-admin)
./deploy.sh
```

Copie o ID C... exibido para `admin/.env`. O teto inicial é 1.000.000, mas as moedas só entram em circulação quando `allocate` é invocado. Não há mint além do teto.

## 2. Painel Flask
```bash
cd admin
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
python -c 'from werkzeug.security import generate_password_hash; print(generate_password_hash(input("Senha do admin: ")))'
cp .env.example .env
# Edite .env: FLASK_SECRET (32+ chars), ADMIN_PASSWORD_HASH, CONTRACT_ID
python app.py
```

Abra `http://127.0.0.1:5000/login`. A chave de assinatura permanece no alias local da Stellar CLI. A aplicação não expõe rotas para transferência ou emissão anônima. O endpoint `/api/balance/G...` é somente leitura e consulta o contrato. Para acesso Android em dispositivo físico, configure VPN e HTTPS; nunca publique o servidor de desenvolvimento Flask.

## 3. Android Studio
Abra `android/`, sincronize Gradle e rode em emulador Android. A URL padrão é `http://10.0.2.2:5000` (loopback do host no emulador). Para alterar: `./gradlew assembleDebug -PAPI_BASE_URL=https://api.seu-dominio.com`. Para telefone físico use endereço HTTPS acessível pelo dispositivo. O `usesCleartextTraffic=true` serve apenas para o emulador de desenvolvimento e deve ser removido antes de distribuição.

## 4. Verificações
```bash
cd contrato && cargo test
cd ../admin && python -m compileall -q .
```

## Limitações e segurança
- O contrato é personalizado, não implementa interface SEP-41; integração com carteiras pode exigir adapter.
- TTL é estendido ao tocar nos registros, mas contas inativas por muito tempo podem exigir restauração de storage.
- O painel grava apenas distribuições concluídas no SQLite, não substitui indexador de eventos blockchain.
- Não há assinatura de transferência no Android, gestão de chaves do usuário, QR ou resgate de recompensas. Os controles de transferência permanecem intencionalmente desativados.
- O contrato e o app não foram validados com compilação/deploy real neste ambiente. Faça testes na Testnet e revisão de segurança antes de uso operacional.
