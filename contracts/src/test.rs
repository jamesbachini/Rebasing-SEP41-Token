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

    rusd.approve(&alice, &spender, &50, &200);
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
fn rebase_fair_share_after_activity_three_users() {
    let (env, usdc_id, rusd_id, alice, bob, donor, ..) = setup();
    let carol = Address::generate(&env);
    let usdc = UsdcMockClient::new(&env, &usdc_id);
    let rusd = RUsdTokenClient::new(&env, &rusd_id);

    usdc.mint(&alice, &1_000);
    usdc.mint(&bob, &1_000);
    usdc.mint(&carol, &1_000);
    usdc.mint(&donor, &1_000);

    let mut underlying: i128 = 0;
    let mut total_shares: i128 = 0;
    let mut alice_shares: i128 = 0;
    let mut bob_shares: i128 = 0;
    let mut carol_shares: i128 = 0;

    usdc.approve(&alice, &rusd_id, &100, &200);
    rusd.mint(&alice, &100);
    let shares_to_mint = model_shares_from_rusd(100, total_shares, underlying);
    alice_shares += shares_to_mint;
    total_shares += shares_to_mint;
    underlying += 100;
    assert_eq!(usdc.balance(&rusd_id), underlying);

    usdc.approve(&bob, &rusd_id, &60, &200);
    rusd.mint(&bob, &60);
    let shares_to_mint = model_shares_from_rusd(60, total_shares, underlying);
    bob_shares += shares_to_mint;
    total_shares += shares_to_mint;
    underlying += 60;
    assert_eq!(usdc.balance(&rusd_id), underlying);

    usdc.approve(&carol, &rusd_id, &40, &200);
    rusd.mint(&carol, &40);
    let shares_to_mint = model_shares_from_rusd(40, total_shares, underlying);
    carol_shares += shares_to_mint;
    total_shares += shares_to_mint;
    underlying += 40;
    assert_eq!(usdc.balance(&rusd_id), underlying);

    rusd.transfer(&bob, &carol, &15);
    let shares_to_move = model_shares_from_rusd(15, total_shares, underlying);
    bob_shares -= shares_to_move;
    carol_shares += shares_to_move;

    rusd.burn(&alice, &20);
    let shares_to_burn = model_shares_from_rusd(20, total_shares, underlying);
    let usdc_out = model_rusd_from_shares(shares_to_burn, total_shares, underlying);
    alice_shares -= shares_to_burn;
    total_shares -= shares_to_burn;
    underlying -= usdc_out;
    assert_eq!(usdc.balance(&rusd_id), underlying);

    usdc.approve(&carol, &rusd_id, &25, &200);
    rusd.mint(&carol, &25);
    let shares_to_mint = model_shares_from_rusd(25, total_shares, underlying);
    carol_shares += shares_to_mint;
    total_shares += shares_to_mint;
    underlying += 25;
    assert_eq!(usdc.balance(&rusd_id), underlying);

    let alice_before = rusd.balance(&alice);
    let bob_before = rusd.balance(&bob);
    let carol_before = rusd.balance(&carol);

    usdc.transfer(&donor, &rusd_id, &17);
    underlying += 17;
    assert_eq!(usdc.balance(&rusd_id), underlying);

    let expected_alice = model_rusd_from_shares(alice_shares, total_shares, underlying);
    let expected_bob = model_rusd_from_shares(bob_shares, total_shares, underlying);
    let expected_carol = model_rusd_from_shares(carol_shares, total_shares, underlying);

    assert_eq!(rusd.balance(&alice), expected_alice);
    assert_eq!(rusd.balance(&bob), expected_bob);
    assert_eq!(rusd.balance(&carol), expected_carol);
    assert_eq!(rusd.total_supply(), underlying);

    let expected_alice_gain = model_mul_div_floor(alice_shares, 17, total_shares);
    let expected_bob_gain = model_mul_div_floor(bob_shares, 17, total_shares);
    let expected_carol_gain = model_mul_div_floor(carol_shares, 17, total_shares);

    assert_eq!(expected_alice - alice_before, expected_alice_gain);
    assert_eq!(expected_bob - bob_before, expected_bob_gain);
    assert_eq!(expected_carol - carol_before, expected_carol_gain);
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

fn model_mul_div_floor(a: i128, b: i128, denom: i128) -> i128 {
    let prod = a.checked_mul(b).expect("overflow");
    prod / denom
}

fn model_mul_div_ceil(a: i128, b: i128, denom: i128) -> i128 {
    let prod = a.checked_mul(b).expect("overflow");
    let div = prod / denom;
    if prod % denom == 0 {
        div
    } else {
        div + 1
    }
}

fn model_shares_from_rusd(amount: i128, total_shares: i128, underlying: i128) -> i128 {
    if total_shares == 0 {
        return amount;
    }
    model_mul_div_ceil(amount, total_shares, underlying)
}

fn model_rusd_from_shares(shares: i128, total_shares: i128, underlying: i128) -> i128 {
    if total_shares == 0 || shares == 0 {
        return 0;
    }
    model_mul_div_floor(shares, underlying, total_shares)
}
