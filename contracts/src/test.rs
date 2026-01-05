#![cfg(test)]

extern crate std;

use soroban_sdk::{
    contract, contractimpl, contracttype, IntoVal, Symbol,
    testutils::{Address as AddressTest, Ledger as LedgerTest},
    Address, Env, String, Val, Vec as SorobanVec,
};

use crate::{Error as ContractError, RUsdToken, RUsdTokenClient};

#[derive(Clone)]
#[contracttype]
struct AllowanceValue {
    amount: i128,
    expiration_ledger: u32,
}

#[derive(Clone)]
#[contracttype]
enum UsdcKey {
    Balance(Address),
    Allowance(Address, Address),
}

#[contract]
pub struct UsdcMock;

#[contractimpl]
impl UsdcMock {
    pub fn mint(env: Env, to: Address, amount: i128) {
        let balance = read_balance(&env, &to);
        write_balance(&env, &to, balance + amount);
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        read_balance(&env, &id)
    }

    pub fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();
        write_allowance(&env, &from, &spender, amount, expiration_ledger);
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        read_allowance_amount(&env, &from, &spender)
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        if amount <= 0 {
            panic!("invalid amount");
        }
        from.require_auth();
        spend_balance(&env, &from, amount);
        add_balance(&env, &to, amount);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        if amount <= 0 {
            panic!("invalid amount");
        }
        spender.require_auth();
        let allowance = read_allowance_amount(&env, &from, &spender);
        if allowance < amount {
            panic!("insufficient allowance");
        }
        spend_balance(&env, &from, amount);
        add_balance(&env, &to, amount);
        write_allowance(&env, &from, &spender, allowance - amount, read_allowance_expiration(&env, &from, &spender));
    }
}

fn read_balance(env: &Env, owner: &Address) -> i128 {
    env.storage()
        .instance()
        .get(&UsdcKey::Balance(owner.clone()))
        .unwrap_or(0)
}

fn write_balance(env: &Env, owner: &Address, amount: i128) {
    env.storage()
        .instance()
        .set(&UsdcKey::Balance(owner.clone()), &amount);
}

fn add_balance(env: &Env, owner: &Address, amount: i128) {
    let balance = read_balance(env, owner);
    write_balance(env, owner, balance + amount);
}

fn spend_balance(env: &Env, owner: &Address, amount: i128) {
    let balance = read_balance(env, owner);
    if balance < amount {
        panic!("insufficient balance");
    }
    write_balance(env, owner, balance - amount);
}

fn read_allowance(env: &Env, owner: &Address, spender: &Address) -> AllowanceValue {
    env.storage()
        .instance()
        .get(&UsdcKey::Allowance(owner.clone(), spender.clone()))
        .unwrap_or(AllowanceValue {
            amount: 0,
            expiration_ledger: 0,
        })
}

fn read_allowance_amount(env: &Env, owner: &Address, spender: &Address) -> i128 {
    let allowance = read_allowance(env, owner, spender);
    let current = env.ledger().sequence();
    if allowance.expiration_ledger != 0 && allowance.expiration_ledger < current {
        0
    } else {
        allowance.amount
    }
}

fn read_allowance_expiration(env: &Env, owner: &Address, spender: &Address) -> u32 {
    read_allowance(env, owner, spender).expiration_ledger
}

fn write_allowance(
    env: &Env,
    owner: &Address,
    spender: &Address,
    amount: i128,
    expiration_ledger: u32,
) {
    let key = UsdcKey::Allowance(owner.clone(), spender.clone());
    if amount == 0 {
        env.storage().instance().remove(&key);
    } else {
        env.storage().instance().set(
            &key,
            &AllowanceValue {
                amount,
                expiration_ledger,
            },
        );
    }
}

fn setup() -> (Env, Address, Address, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| {
        li.sequence_number = 100;
    });

    let usdc_id = env.register_contract(None, UsdcMock);
    let rusd_id = env.register_contract(None, RUsdToken);
    let rusd = RUsdTokenClient::new(&env, &rusd_id);

    let name = String::from_str(&env, "rUSD");
    let symbol = String::from_str(&env, "rUSD");
    rusd.init(&usdc_id, &name, &symbol, &7u32);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let donor = Address::generate(&env);
    let spender = Address::generate(&env);

    (env, usdc_id, rusd_id, alice, bob, donor, spender)
}

#[test]
fn init_and_metadata() {
    let (env, _usdc_id, rusd_id, ..) = setup();
    let rusd = RUsdTokenClient::new(&env, &rusd_id);
    assert_eq!(rusd.name(), String::from_str(&env, "rUSD"));
    assert_eq!(rusd.symbol(), String::from_str(&env, "rUSD"));
    assert_eq!(rusd.decimals(), 7);
}

#[test]
fn mint_sets_balance_and_supply() {
    let (env, usdc_id, rusd_id, alice, ..) = setup();
    let usdc = UsdcMockClient::new(&env, &usdc_id);
    let rusd = RUsdTokenClient::new(&env, &rusd_id);
    usdc.mint(&alice, &1_000);
    usdc.approve(&alice, &rusd_id, &200, &200);

    rusd.mint(&alice, &200);

    assert_eq!(usdc.balance(&alice), 800);
    assert_eq!(usdc.balance(&rusd_id), 200);
    assert_eq!(rusd.balance(&alice), 200);
    assert_eq!(rusd.total_supply(), 200);
}

#[test]
fn transfer_moves_rebased_amount() {
    let (env, usdc_id, rusd_id, alice, bob, ..) = setup();
    let usdc = UsdcMockClient::new(&env, &usdc_id);
    let rusd = RUsdTokenClient::new(&env, &rusd_id);
    usdc.mint(&alice, &500);
    usdc.approve(&alice, &rusd_id, &100, &200);
    rusd.mint(&alice, &100);

    rusd.transfer(&alice, &bob, &40);

    assert_eq!(rusd.balance(&alice), 60);
    assert_eq!(rusd.balance(&bob), 40);
}

#[test]
fn allowance_and_transfer_from() {
    let (env, usdc_id, rusd_id, alice, bob, _donor, spender) = setup();
    let usdc = UsdcMockClient::new(&env, &usdc_id);
    let rusd = RUsdTokenClient::new(&env, &rusd_id);
    usdc.mint(&alice, &500);
    usdc.approve(&alice, &rusd_id, &100, &200);
    rusd.mint(&alice, &100);

    rusd.approve(&alice, &spender, &50);
    assert_eq!(rusd.allowance(&alice, &spender), 50);

    rusd.transfer_from(&spender, &alice, &bob, &20);

    assert_eq!(rusd.allowance(&alice, &spender), 30);
    assert_eq!(rusd.balance(&alice), 80);
    assert_eq!(rusd.balance(&bob), 20);
}

#[test]
fn rebase_inflow_increases_balances() {
    let (env, usdc_id, rusd_id, alice, bob, donor, ..) = setup();
    let usdc = UsdcMockClient::new(&env, &usdc_id);
    let rusd = RUsdTokenClient::new(&env, &rusd_id);
    usdc.mint(&alice, &1_000);
    usdc.mint(&bob, &1_000);
    usdc.mint(&donor, &1_000);

    usdc.approve(&alice, &rusd_id, &100, &200);
    rusd.mint(&alice, &100);
    usdc.approve(&bob, &rusd_id, &50, &200);
    rusd.mint(&bob, &50);

    usdc.transfer(&donor, &rusd_id, &15);

    assert_eq!(rusd.balance(&alice), 110);
    assert_eq!(rusd.balance(&bob), 55);
    assert_eq!(rusd.total_supply(), 165);
}

#[test]
fn burn_redeems_usdc() {
    let (env, usdc_id, rusd_id, alice, ..) = setup();
    let usdc = UsdcMockClient::new(&env, &usdc_id);
    let rusd = RUsdTokenClient::new(&env, &rusd_id);
    usdc.mint(&alice, &1_000);
    usdc.approve(&alice, &rusd_id, &100, &200);
    rusd.mint(&alice, &100);

    rusd.burn(&alice, &40);

    assert_eq!(rusd.balance(&alice), 60);
    assert_eq!(rusd.total_supply(), 60);
    assert_eq!(usdc.balance(&alice), 940);
    assert_eq!(usdc.balance(&rusd_id), 60);
}

#[test]
#[ignore]
fn zero_amount_mint_fails() {
    let (env, _usdc_id, rusd_id, alice, ..) = setup();
    let args = args_with_amount(&env, &alice, 0);
    assert_contract_error(&env, &rusd_id, "mint", args, ContractError::ZeroAmount);
}

#[test]
#[ignore]
fn burn_more_than_balance_fails() {
    let (env, usdc_id, rusd_id, alice, ..) = setup();
    let usdc = UsdcMockClient::new(&env, &usdc_id);
    let rusd = RUsdTokenClient::new(&env, &rusd_id);
    usdc.mint(&alice, &100);
    usdc.approve(&alice, &rusd_id, &50, &200);
    rusd.mint(&alice, &50);

    let args = args_with_amount(&env, &alice, 100);
    assert_contract_error(
        &env,
        &rusd_id,
        "burn",
        args,
        ContractError::InsufficientShares,
    );
}

fn args_with_amount(env: &Env, owner: &Address, amount: i128) -> SorobanVec<Val> {
    let mut args = SorobanVec::new(env);
    args.push_back(owner.clone().into_val(env));
    args.push_back(amount.into_val(env));
    args
}

fn assert_contract_error(
    env: &Env,
    contract_id: &Address,
    fn_name: &str,
    args: SorobanVec<Val>,
    expected: ContractError,
) {
    let result = env.try_invoke_contract::<(), ContractError>(
        contract_id,
        &Symbol::new(env, fn_name),
        args,
    );
    match result {
        Err(Ok(err)) => assert_eq!(err, expected),
        Err(Err(_)) => panic!("unexpected invoke error"),
        Ok(_) => panic!("expected contract error"),
    }
}
