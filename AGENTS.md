# rUSD Rebasing Token on Stellar (Soroban) — Project Overview (AGENTS)

## Goal
Build a Soroban smart contract implementing a **rebasing** SEP-41 token **rUSD** (ERC-20 equivalent on Stellar), backed 1:1 by **USDC** deposits.

- Users deposit USDC → receive rUSD minted 1:1
- Users burn rUSD → receive USDC returned 1:1 at the current rebase rate
- Contract holds underlying USDC directly (for now)
- “Yield simulation” is supported: if extra USDC is sent to the contract address, **rUSD balances rebase upward automatically** (pro-rata)

Example:
- User1: 100 rUSD, User2: 50 rUSD (total shares represent 150 rUSD, underlying 150 USDC)
- Someone transfers +15 USDC to the contract address → underlying becomes 165
- Rebase implied exchange rate increases 1.10x
- User1 now sees 110 rUSD, User2 sees 55 rUSD

## Architecture Summary
### Core concept: share-based rebasing (recommended)
Maintain:
- `total_shares` (fixed unless mint/burn)
- `shares[addr]`
- `underlying_usdc_balance` = USDC.balance(contract_address)
- “Displayed rUSD balance” computed as:
  - `balance_rusd(addr) = shares[addr] * underlying / total_shares`
- Mint:
  - user transfers USDC into contract (via allowance / transfer_from)
  - compute shares_to_mint based on current exchange rate
- Burn:
  - compute underlying_out based on shares to burn
  - transfer USDC out to user
  - reduce shares

This makes rebasing “automatic” because balances are computed from changing underlying.

### SEP-41 interface
Expose the standard SEP-41 token methods (transfer, approve, allowance, etc.) and metadata (name, symbol, decimals).

Implementation approach:
- Use OpenZeppelin Soroban token libraries as the base (SEP-41) and adapt storage logic so “balances” are shares while user-facing balances are rebased amounts.
- If full ERC-20-style `transfer(amount)` is required, the contract should interpret `amount` as **rebased rUSD units** and internally convert to shares.

## Repo Structure
- contracts/
  - src/main.rs   (Soroban contract)
  - src/test.rs   (unit tests)
  - Cargo.toml
- frontend/
  - Next.js app using creittech v2 wallet kit
  - Connect Freighter
  - Approve USDC spend
  - Mint rUSD
  - Display balances + rebase effect

## Configuration / Constants
Must be configurable without code edits:
- USDC contract id (per network)
- Network selection (futurenet/testnet/local)

Initial deployment to testnet. USDC contract ID on testnet is: CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA