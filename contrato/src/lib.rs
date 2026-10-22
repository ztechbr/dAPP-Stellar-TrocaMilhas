//! # MotoClube Coin — contrato de recompensas em Soroban
//!
//! ## O que este contrato faz
//!
//! Um token de recompensa simples, com **teto fixo de emissão**:
//!
//! 1. No deploy, o construtor grava quem é o administrador e qual é o teto
//!    (`cap`). Nenhuma moeda existe ainda.
//! 2. O administrador chama [`MotoCoin::allocate`] para colocar moedas em
//!    circulação, sempre respeitando o teto.
//! 3. Qualquer titular move o próprio saldo com [`MotoCoin::transfer`].
//! 4. [`MotoCoin::balance`] e [`MotoCoin::remaining`] são consultas.
//!
//! ## Como ler este arquivo (tutorial)
//!
//! * **Armazenamento** (`DataKey`): o Soroban guarda dados em pares
//!   chave/valor. Dados globais (admin, teto, total distribuído) ficam no
//!   storage *instance*, que vive junto do contrato. Saldos ficam no storage
//!   *persistent*, um registro por endereço.
//! * **TTL**: todo registro expira se ninguém o tocar. Por isso cada acesso
//!   renova o prazo (`extend_ttl`). As constantes abaixo dizem "quando faltar
//!   menos de N ledgers, renove para M ledgers".
//! * **Autorização**: `address.require_auth()` faz a transação falhar se o
//!   dono daquele endereço não tiver assinado. É a única barreira de
//!   segurança: sem ela qualquer um moveria saldo alheio.
//! * **Eventos**: cada alteração de saldo publica um evento para indexadores
//!   e carteiras acompanharem o histórico on-chain.
//!
//! ## Por que um construtor e não `initialize`
//!
//! A versão anterior expunha `initialize` como função pública separada. Entre
//! o deploy e a chamada de `initialize`, qualquer pessoa poderia chamá-la
//! primeiro e virar administradora (*front-running*). O `__constructor`
//! roda **na mesma transação do deploy**, eliminando essa janela.
#![no_std]

use soroban_sdk::{contract, contractevent, contractimpl, contracttype, Address, Env};

// -----------------------------------------------------------------------------
// Prazos de vida (TTL) em ledgers. Um ledger fecha a cada ~5 segundos:
// 518_400 ledgers ≈ 30 dias; 259_200 ledgers ≈ 15 dias.
// -----------------------------------------------------------------------------

/// Prazo para o qual o registro é renovado (~30 dias).
const TTL_EXTEND_TO: u32 = 518_400;
/// Só renova quando restar menos que isto (~15 dias), economizando taxa.
const TTL_THRESHOLD: u32 = 259_200;

/// Chaves de armazenamento do contrato.
#[contracttype]
#[derive(Clone)]
enum DataKey {
    /// Endereço autorizado a distribuir moedas (storage instance).
    Admin,
    /// Teto máximo de moedas que podem existir (storage instance).
    Cap,
    /// Quantas moedas já foram distribuídas (storage instance).
    Distributed,
    /// Saldo de um titular (storage persistent, um registro por endereço).
    Balance(Address),
}

/// Evento emitido quando o administrador distribui moedas.
/// Tópicos: `["allocate", to]`; dados: `{ amount }`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Allocate {
    #[topic]
    pub to: Address,
    pub amount: i128,
}

/// Evento emitido em toda transferência entre titulares.
/// Tópicos: `["transfer", from, to]`; dados: `{ amount }`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transfer {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
}

// -----------------------------------------------------------------------------
// Funções auxiliares privadas: cada uma faz uma única coisa e tem um nome que
// explica o quê. As funções públicas do contrato ficam curtas e legíveis.
// -----------------------------------------------------------------------------

/// Mantém vivo o storage *instance* (admin, teto, distribuído e o código).
fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

/// Lê um valor obrigatório do storage *instance*.
///
/// Só falha se o contrato estiver corrompido, pois o construtor grava todas
/// as chaves globais no deploy.
fn read_instance<V: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>>(env: &Env, key: &DataKey) -> V {
    env.storage()
        .instance()
        .get(key)
        .expect("contract not initialized")
}

/// Retorna o saldo de `owner` (zero se nunca recebeu moedas) e renova o TTL
/// do registro existente.
fn read_balance(env: &Env, owner: &Address) -> i128 {
    let key = DataKey::Balance(owner.clone());
    let storage = env.storage().persistent();
    match storage.get::<_, i128>(&key) {
        Some(balance) => {
            storage.extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
            balance
        }
        None => 0,
    }
}

/// Grava o novo saldo de `owner` e renova o TTL do registro.
fn write_balance(env: &Env, owner: &Address, amount: i128) {
    let key = DataKey::Balance(owner.clone());
    let storage = env.storage().persistent();
    storage.set(&key, &amount);
    storage.extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
}

/// Rejeita quantidades nulas ou negativas, que inverteriam o sentido da
/// operação (ex.: "transferir -10" roubaria do destinatário).
fn require_positive(amount: i128) {
    assert!(amount > 0, "invalid amount");
}

// -----------------------------------------------------------------------------
// Interface pública do contrato
// -----------------------------------------------------------------------------

#[contract]
pub struct MotoCoin;

#[contractimpl]
impl MotoCoin {
    /// **Construtor** — executado uma única vez, dentro da transação de deploy.
    ///
    /// * `admin`: endereço que poderá distribuir moedas. Precisa assinar o
    ///   deploy, o que garante que é um endereço controlado por alguém.
    /// * `cap`: teto de moedas que poderão existir. Deve ser positivo.
    ///
    /// Exemplo via CLI (o `compile.bat deploy` faz isso por você):
    /// `stellar contract deploy --wasm motoclube_coin.wasm ... -- --admin G... --cap 1000000`
    pub fn __constructor(env: Env, admin: Address, cap: i128) {
        assert!(cap > 0, "invalid cap");
        admin.require_auth();

        let storage = env.storage().instance();
        storage.set(&DataKey::Admin, &admin);
        storage.set(&DataKey::Cap, &cap);
        storage.set(&DataKey::Distributed, &0_i128);
        extend_instance_ttl(&env);
    }

    /// **Distribui** `amount` moedas novas para `to`.
    ///
    /// Passo a passo:
    /// 1. Exige a assinatura do administrador.
    /// 2. Valida a quantidade e confere se o teto não será ultrapassado.
    /// 3. Credita o destinatário, atualiza o total distribuído e emite o
    ///    evento `allocate`.
    ///
    /// Falha com `"cap exceeded"` se não houver moedas disponíveis.
    pub fn allocate(env: Env, to: Address, amount: i128) {
        let admin: Address = read_instance(&env, &DataKey::Admin);
        admin.require_auth();
        require_positive(amount);
        extend_instance_ttl(&env);

        let cap: i128 = read_instance(&env, &DataKey::Cap);
        let distributed: i128 = read_instance(&env, &DataKey::Distributed);
        let new_distributed = distributed.checked_add(amount).expect("overflow");
        assert!(new_distributed <= cap, "cap exceeded");

        let new_balance = read_balance(&env, &to)
            .checked_add(amount)
            .expect("overflow");
        write_balance(&env, &to, new_balance);
        env.storage()
            .instance()
            .set(&DataKey::Distributed, &new_distributed);

        Allocate { to, amount }.publish(&env);
    }

    /// **Transfere** `amount` moedas de `from` para `to`.
    ///
    /// Somente o dono de `from` pode autorizar. Falha com
    /// `"insufficient balance"` se o saldo não cobrir a quantidade e com
    /// `"same address"` se origem e destino forem iguais.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        require_positive(amount);
        assert!(from != to, "same address");
        extend_instance_ttl(&env);

        let from_balance = read_balance(&env, &from);
        assert!(from_balance >= amount, "insufficient balance");
        let to_balance = read_balance(&env, &to)
            .checked_add(amount)
            .expect("overflow");

        write_balance(&env, &from, from_balance - amount);
        write_balance(&env, &to, to_balance);

        Transfer { from, to, amount }.publish(&env);
    }

    /// **Consulta** o saldo de `user`. Endereços desconhecidos têm saldo 0.
    pub fn balance(env: Env, user: Address) -> i128 {
        extend_instance_ttl(&env);
        read_balance(&env, &user)
    }

    /// **Consulta** quantas moedas ainda podem ser distribuídas
    /// (`teto - já distribuído`).
    pub fn remaining(env: Env) -> i128 {
        extend_instance_ttl(&env);
        let cap: i128 = read_instance(&env, &DataKey::Cap);
        let distributed: i128 = read_instance(&env, &DataKey::Distributed);
        cap - distributed
    }
}

// -----------------------------------------------------------------------------
// Testes: rodam num ambiente Soroban simulado com `cargo test`.
// `mock_all_auths` aprova qualquer `require_auth`, permitindo testar a lógica
// de negócio sem gerar assinaturas reais.
// -----------------------------------------------------------------------------
#[cfg(test)]
mod test {
    extern crate std;

    use super::*;
    use soroban_sdk::testutils::Address as _;

    /// Monta um contrato com teto `cap` e devolve o cliente e o admin.
    fn setup(env: &Env, cap: i128) -> (MotoCoinClient<'_>, Address) {
        env.mock_all_auths();
        let admin = Address::generate(env);
        let contract_id = env.register(MotoCoin, (&admin, cap));
        (MotoCoinClient::new(env, &contract_id), admin)
    }

    #[test]
    fn allocate_and_transfer_update_balances() {
        let env = Env::default();
        let (client, _admin) = setup(&env, 100);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        client.allocate(&alice, &70);
        client.transfer(&alice, &bob, &20);

        assert_eq!(client.balance(&alice), 50);
        assert_eq!(client.balance(&bob), 20);
        assert_eq!(client.remaining(), 30);
    }

    #[test]
    fn allocate_cannot_exceed_cap() {
        let env = Env::default();
        let (client, _admin) = setup(&env, 100);
        let bob = Address::generate(&env);

        client.allocate(&bob, &100);

        assert!(client.try_allocate(&bob, &1).is_err());
        assert_eq!(client.remaining(), 0);
    }

    #[test]
    fn transfer_rejects_invalid_requests() {
        let env = Env::default();
        let (client, _admin) = setup(&env, 100);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        client.allocate(&alice, &10);

        assert!(client.try_transfer(&alice, &bob, &11).is_err(), "saldo insuficiente");
        assert!(client.try_transfer(&alice, &bob, &0).is_err(), "quantidade zero");
        assert!(client.try_transfer(&alice, &bob, &-5).is_err(), "quantidade negativa");
        assert!(client.try_transfer(&alice, &alice, &1).is_err(), "mesmo endereço");
        assert_eq!(client.balance(&alice), 10);
    }

    #[test]
    fn allocate_requires_admin_signature() {
        let env = Env::default();
        let (client, admin) = setup(&env, 100);
        let alice = Address::generate(&env);

        client.allocate(&alice, &5);

        let (signer, _) = env.auths().pop().expect("allocate must require auth");
        assert_eq!(signer, admin);
    }

    #[test]
    #[should_panic(expected = "invalid cap")]
    fn constructor_rejects_non_positive_cap() {
        let env = Env::default();
        setup(&env, 0);
    }
}
