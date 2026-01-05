# rUSD Contract Architecture

## Component Map
- **Entrypoints (public surface)**: Exposes SEP-41 methods plus custom mint/burn and init.
- **Config storage**: Persists USDC contract ID, decimals, token metadata, and any network-specific settings.
- **Share accounting storage**: Tracks `total_shares` and `shares[addr]` (internal-only).
- **Allowance/approval storage**: Uses OpenZeppelin SEP-41 helpers to track approvals in rebased rUSD units.
- **Rebasing math utilities**: Converts between rebased rUSD units and internal shares using the current exchange rate.
- **USDC client adapter**: A thin wrapper around the USDC token client for balance and transfer calls.

## Public Entrypoint Categories
- **Initialization/config**
  - `init`: Stores USDC contract ID and token metadata; no hard-coded constants.
- **SEP-41 surface (rebased units)**
  - `balance`, `transfer`, `approve`, `allowance`, `transfer_from`, `name`, `symbol`, `decimals`, `total_supply`.
- **Custom mint/burn surface**
  - `mint`: Pulls USDC in and mints shares.
  - `burn`: Burns shares and returns USDC.

## Control Flow (Reads and Writes)
- **Reads**
  - `balance(address)`:
    1) Load `shares[address]`.
    2) Query `underlying_usdc` via `USDC.balance(contract_address)`.
    3) Convert shares → rebased rUSD using the current exchange rate.
  - `total_supply()`:
    1) Load `total_shares`.
    2) Convert shares → rebased rUSD using current underlying balance.
  - All reads are rebase-aware by construction; no stored “rebased balance” exists.

- **Writes**
  - `transfer(amount)` / `transfer_from(amount)`:
    1) Interpret `amount` as rebased rUSD units.
    2) Convert rUSD → shares at current exchange rate.
    3) Update `shares[sender]` and `shares[recipient]` accordingly.
    4) For `transfer_from`, update allowance in rebased units via OpenZeppelin helpers.
  - `mint(amount)`:
    1) Pull USDC in via `USDC.transfer_from(user, contract, amount)` after approval.
    2) Convert the rebased `amount` to shares at the current exchange rate.
    3) Increase `shares[user]` and `total_shares`.
  - `burn(amount)`:
    1) Convert rebased `amount` to shares.
    2) Decrease `shares[user]` and `total_shares`.
    3) Transfer underlying USDC out via `USDC.transfer(contract, user, underlying_out)`.

## Rebasing Semantics
- No explicit “rebase” entrypoint exists.
- The exchange rate is derived from the contract’s live USDC balance.
- When extra USDC is sent to the contract, all rebased balances increase implicitly.

## USDC Contract Access
- The USDC contract ID is stored in config at `init`.
- A token client (e.g., OpenZeppelin or Soroban token client) is constructed with that ID.
- Required calls:
  - `balance(contract_address)` for exchange-rate derivation.
  - `transfer_from(user, contract, amount)` to pull USDC on mint.
  - `transfer(contract, user, amount)` to return USDC on burn.
