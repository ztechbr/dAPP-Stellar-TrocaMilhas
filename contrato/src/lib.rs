#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

#[contracttype]
#[derive(Clone)]
enum Key { Admin, Cap, Distributed, Balance(Address) }

const INSTANCE_TTL: u32 = 518_400;
const INSTANCE_THRESHOLD: u32 = 259_200;
const BALANCE_TTL: u32 = 518_400;
const BALANCE_THRESHOLD: u32 = 259_200;

fn keep_instance(env: &Env) {
    env.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_TTL);
}
fn read_balance(env: &Env, owner: &Address) -> i128 {
    let key = Key::Balance(owner.clone());
    if env.storage().persistent().has(&key) {
        env.storage().persistent().extend_ttl(&key, BALANCE_THRESHOLD, BALANCE_TTL);
    }
    env.storage().persistent().get(&key).unwrap_or(0)
}
fn save_balance(env: &Env, owner: &Address, amount: i128) {
    let key = Key::Balance(owner.clone());
    env.storage().persistent().set(&key, &amount);
    env.storage().persistent().extend_ttl(&key, BALANCE_THRESHOLD, BALANCE_TTL);
}

#[contract]
pub struct MotoCoin;

#[contractimpl]
impl MotoCoin {
    pub fn initialize(env: Env, admin: Address, cap: i128) {
        assert!(!env.storage().instance().has(&Key::Admin), "initialized");
        assert!(cap > 0, "invalid cap");
        admin.require_auth();
        env.storage().instance().set(&Key::Admin, &admin);
        env.storage().instance().set(&Key::Cap, &cap);
        env.storage().instance().set(&Key::Distributed, &0_i128);
        keep_instance(&env);
    }
    pub fn allocate(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&Key::Admin).unwrap();
        admin.require_auth();
        keep_instance(&env);
        assert!(amount > 0, "invalid amount");
        let used: i128 = env.storage().instance().get(&Key::Distributed).unwrap();
        let cap: i128 = env.storage().instance().get(&Key::Cap).unwrap();
        let next = used.checked_add(amount).expect("overflow");
        assert!(next <= cap, "cap exceeded");
        let balance = read_balance(&env, &to);
        save_balance(&env, &to, balance.checked_add(amount).expect("overflow"));
        env.events().publish((symbol_short!("allocate"), to), amount);
        env.storage().instance().set(&Key::Distributed, &next);
    }
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        keep_instance(&env);
        assert!(amount > 0, "invalid amount");
        assert!(from != to, "same address");
        let balance = read_balance(&env, &from);
        assert!(balance >= amount, "insufficient balance");
        let recipient = read_balance(&env, &to);
        save_balance(&env, &from, balance - amount);
        save_balance(&env, &to, recipient.checked_add(amount).expect("overflow"));
        env.events().publish((symbol_short!("transfer"), from, to), amount);
    }
    pub fn balance(env: Env, user: Address) -> i128 {
        keep_instance(&env);
        read_balance(&env, &user)
    }
    pub fn remaining(env: Env) -> i128 {
        keep_instance(&env);
        let cap: i128 = env.storage().instance().get(&Key::Cap).unwrap();
        let used: i128 = env.storage().instance().get(&Key::Distributed).unwrap();
        cap - used
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::testutils::Address as _;
    #[test]
    fn distribution_transfer_and_cap() {
        let env = Env::default();
        env.mock_all_auths();
        let contract = env.register(MotoCoin, ());
        let client = MotoCoinClient::new(&env, &contract);
        let admin = Address::generate(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        client.initialize(&admin, &100);
        client.allocate(&alice, &70);
        client.transfer(&alice, &bob, &20);
        assert_eq!(client.balance(&alice), 50);
        assert_eq!(client.balance(&bob), 20);
        assert_eq!(client.remaining(), 30);
        assert!(client.try_allocate(&bob, &31).is_err());
        assert!(client.try_transfer(&bob, &alice, &21).is_err());
    }
}
