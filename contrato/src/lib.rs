#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
enum Key { Admin, Cap, Distributed, Balance(Address) }

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
    }
    pub fn allocate(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&Key::Admin).unwrap();
        admin.require_auth();
        assert!(amount > 0, "invalid amount");
        let used: i128 = env.storage().instance().get(&Key::Distributed).unwrap();
        let cap: i128 = env.storage().instance().get(&Key::Cap).unwrap();
        let next = used.checked_add(amount).expect("overflow");
        assert!(next <= cap, "cap exceeded");
        let key = Key::Balance(to);
        let balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &balance.checked_add(amount).expect("overflow"));
        env.storage().instance().set(&Key::Distributed, &next);
    }
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        assert!(amount > 0, "invalid amount");
        let fk = Key::Balance(from);
        let tk = Key::Balance(to);
        let balance: i128 = env.storage().persistent().get(&fk).unwrap_or(0);
        assert!(balance >= amount, "insufficient balance");
        let recipient: i128 = env.storage().persistent().get(&tk).unwrap_or(0);
        env.storage().persistent().set(&fk, &(balance - amount));
        env.storage().persistent().set(&tk, &recipient.checked_add(amount).expect("overflow"));
    }
    pub fn balance(env: Env, user: Address) -> i128 {
        env.storage().persistent().get(&Key::Balance(user)).unwrap_or(0)
    }
    pub fn remaining(env: Env) -> i128 {
        let cap: i128 = env.storage().instance().get(&Key::Cap).unwrap();
        let used: i128 = env.storage().instance().get(&Key::Distributed).unwrap();
        cap - used
    }
}
