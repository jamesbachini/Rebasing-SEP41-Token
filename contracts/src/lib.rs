#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error,
    token::Client as TokenClient, Address, Env, String,
};
use stellar_macros::default_impl;
use stellar_tokens::fungible::{emit_mint, emit_transfer, Base, FungibleToken};

#[contracterror]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    ZeroAmount = 3,
    InsufficientShares = 4,
    DivisionByZero = 6,
    Overflow = 7,
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub usdc_contract_id: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    TotalShares,
    Shares(Address),
}

#[contract]
pub struct RUsdToken;

pub struct RebasingOverrides;

impl stellar_tokens::fungible::ContractOverrides for RebasingOverrides {
    fn total_supply(e: &Env) -> i128 {
        let total_shares = read_total_shares(e);
        if total_shares == 0 {
            return 0;
        }
        let underlying = read_underlying(e);
        rusd_from_shares(e, total_shares, total_shares, underlying)
    }

    fn balance(e: &Env, account: &Address) -> i128 {
        let total_shares = read_total_shares(e);
        if total_shares == 0 {
            return 0;
        }
        let shares = read_shares(e, account);
        if shares == 0 {
            return 0;
        }
        let underlying = read_underlying(e);
        rusd_from_shares(e, shares, total_shares, underlying)
    }

    fn transfer(e: &Env, from: &Address, to: &Address, amount: i128) {
        require_positive_amount(e, amount);
        from.require_auth();

        let total_shares = read_total_shares(e);
        let underlying = read_underlying(e);
        let shares_to_move = shares_from_rusd(e, amount, total_shares, underlying);
        let from_shares = read_shares(e, from);
        if from_shares < shares_to_move {
            panic_with_error!(e, Error::InsufficientShares);
        }
        let new_from_shares = from_shares
            .checked_sub(shares_to_move)
            .unwrap_or_else(|| panic_with_error!(e, Error::Overflow));
        write_shares(e, from, new_from_shares);
        let to_shares = read_shares(e, to);
        let new_to_shares = to_shares
            .checked_add(shares_to_move)
            .unwrap_or_else(|| panic_with_error!(e, Error::Overflow));
        write_shares(e, to, new_to_shares);
        emit_transfer(e, from, to, amount);
    }

    fn transfer_from(e: &Env, spender: &Address, from: &Address, to: &Address, amount: i128) {
        require_positive_amount(e, amount);
        spender.require_auth();
        Base::spend_allowance(e, from, spender, amount);
        let total_shares = read_total_shares(e);
        let underlying = read_underlying(e);
        let shares_to_move = shares_from_rusd(e, amount, total_shares, underlying);
        let from_shares = read_shares(e, from);
        if from_shares < shares_to_move {
            panic_with_error!(e, Error::InsufficientShares);
        }
        let new_from_shares = from_shares
            .checked_sub(shares_to_move)
            .unwrap_or_else(|| panic_with_error!(e, Error::Overflow));
        write_shares(e, from, new_from_shares);
        let to_shares = read_shares(e, to);
        let new_to_shares = to_shares
            .checked_add(shares_to_move)
            .unwrap_or_else(|| panic_with_error!(e, Error::Overflow));
        write_shares(e, to, new_to_shares);
        emit_transfer(e, from, to, amount);
    }
}

#[cfg(test)]
mod test;

#[contractimpl]
impl RUsdToken {
    pub fn init(env: Env, usdc_contract_id: Address, name: String, symbol: String, decimals: u32) {
        if env.storage().instance().has(&DataKey::Config) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        let config = Config { usdc_contract_id };
        env.storage().instance().set(&DataKey::Config, &config);
        Base::set_metadata(&env, decimals, name, symbol);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        require_positive_amount(&env, amount);
        to.require_auth();
        let contract = env.current_contract_address();
        let usdc = usdc_client(&env);
        usdc.transfer_from(&contract, &to, &contract, &amount);
        let total_shares = read_total_shares(&env);
        let underlying_after = read_underlying(&env);
        let shares_to_mint = if total_shares == 0 {
            amount
        } else {
            let underlying_before = underlying_after
                .checked_sub(amount)
                .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
            shares_from_rusd(&env, amount, total_shares, underlying_before)
        };
        let user_shares = read_shares(&env, &to);
        let new_user_shares = user_shares
            .checked_add(shares_to_mint)
            .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
        write_shares(&env, &to, new_user_shares);
        let new_total_shares = total_shares
            .checked_add(shares_to_mint)
            .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
        write_total_shares(&env, new_total_shares);

        emit_mint(&env, &to, amount);
    }

    pub fn burn(env: Env, from: Address, amount: i128) {
        require_positive_amount(&env, amount);
        from.require_auth();
        let total_shares = read_total_shares(&env);
        let underlying = read_underlying(&env);
        let shares_to_burn = shares_from_rusd(&env, amount, total_shares, underlying);
        let user_shares = read_shares(&env, &from);
        if user_shares < shares_to_burn {
            panic_with_error!(&env, Error::InsufficientShares);
        }
        let usdc_out = rusd_from_shares(&env, shares_to_burn, total_shares, underlying);
        let new_user_shares = user_shares
            .checked_sub(shares_to_burn)
            .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
        write_shares(&env, &from, new_user_shares);
        let new_total_shares = total_shares
            .checked_sub(shares_to_burn)
            .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
        write_total_shares(&env, new_total_shares);
        let contract = env.current_contract_address();
        let usdc = usdc_client(&env);
        usdc.transfer(&contract, &from, &usdc_out);
    }
}

#[default_impl]
#[contractimpl]
impl FungibleToken for RUsdToken {
    type ContractType = RebasingOverrides;
}

fn read_config(env: &Env) -> Config {
    env.storage().instance().get(&DataKey::Config)
        .unwrap_or_else(|| panic_with_error!(env, Error::NotInitialized))
}

fn read_total_shares(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::TotalShares).unwrap_or(0)
}

fn write_total_shares(env: &Env, total_shares: i128) {
    env.storage().instance().set(&DataKey::TotalShares, &total_shares);
}

fn read_shares(env: &Env, owner: &Address) -> i128 {
    env.storage().instance().get(&DataKey::Shares(owner.clone())).unwrap_or(0)
}

fn write_shares(env: &Env, owner: &Address, shares: i128) {
    let key = DataKey::Shares(owner.clone());
    if shares == 0 {
        env.storage().instance().remove(&key);
    } else {
        env.storage().instance().set(&key, &shares);
    }
}

fn usdc_client(env: &Env) -> TokenClient<'_> {
    let config = read_config(env);
    TokenClient::new(env, &config.usdc_contract_id)
}

fn read_underlying(env: &Env) -> i128 {
    let contract = env.current_contract_address();
    let usdc = usdc_client(env);
    usdc.balance(&contract)
}

fn require_positive_amount(env: &Env, amount: i128) {
    if amount <= 0 {
        panic_with_error!(env, Error::ZeroAmount);
    }
}

fn shares_from_rusd(env: &Env, amount: i128, total_shares: i128, underlying: i128) -> i128 {
    if total_shares == 0 {
        return amount;
    }
    if underlying == 0 {
        panic_with_error!(env, Error::DivisionByZero);
    }
    mul_div_ceil(env, amount, total_shares, underlying)
}

fn rusd_from_shares(env: &Env, shares: i128, total_shares: i128, underlying: i128) -> i128 {
    if total_shares == 0 || shares == 0 {
        return 0;
    }
    mul_div_floor(env, shares, underlying, total_shares)
}

fn mul_div_floor(env: &Env, a: i128, b: i128, denom: i128) -> i128 {
    if denom == 0 {
        panic_with_error!(env, Error::DivisionByZero);
    }
    let prod = a
        .checked_mul(b)
        .unwrap_or_else(|| panic_with_error!(env, Error::Overflow));
    prod / denom
}

fn mul_div_ceil(env: &Env, a: i128, b: i128, denom: i128) -> i128 {
    if denom == 0 {
        panic_with_error!(env, Error::DivisionByZero);
    }
    let prod = a
        .checked_mul(b)
        .unwrap_or_else(|| panic_with_error!(env, Error::Overflow));
    let div = prod / denom;
    if prod % denom == 0 {
        div
    } else {
        div + 1
    }
}
