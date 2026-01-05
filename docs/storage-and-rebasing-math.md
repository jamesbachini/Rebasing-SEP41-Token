# Storage and Rebasing Math

## Minimal Storage Schema
- **Config**
  - `Config::USDC_CONTRACT_ID -> Address`
  - `Config::DECIMALS -> u32` (mirrors USDC decimals)
  - `Config::NAME -> String`
  - `Config::SYMBOL -> String`
- **Share accounting**
  - `Data::TOTAL_SHARES -> i128`
  - `Data::SHARES(addr) -> i128`
- **Allowances**
  - Managed via OpenZeppelin SEP-41 helpers (rebased rUSD units).

## Core Inputs
- `underlying` = `USDC.balance(contract_address)` (queried at call time)
- `total_shares` = storage value
- `decimals` = from config (mirrors USDC)

## Conversion Formulas
- **exchange_rate**
  - `exchange_rate = underlying / total_shares` (conceptual; avoid storing)
- **shares_from_rusd(amount)**
  - If `total_shares == 0`: `shares = amount`
  - Else: `shares = ceil(amount * total_shares / underlying)`
- **rusd_from_shares(shares)**
  - If `total_shares == 0`: `rusd = 0`
  - Else: `rusd = floor(shares * underlying / total_shares)`

## Rounding Rules (Defaults)
- **Inputs (mint/transfer/burn amount → shares)**: round **up** (`ceil`) to avoid under-collecting shares.
- **Outputs (shares → rUSD view / USDC out)**: round **down** (`floor`) to avoid overpaying.
- These match the defaults in `prompts/AGENTS.md` (floor on outputs, ceil on inputs).

## Edge-Case Handling
- **First mint (`total_shares == 0`)**
  - Set `shares = amount` so 1 rUSD == 1 share at bootstrap.
  - `total_shares` becomes `amount`.
  - Requires `underlying` to reflect the transferred-in USDC.
- **Zero-balance accounts**
  - `shares[addr]` can be absent or `0`; `balance(addr)` returns `0`.
- **Dust handling**
  - `rusd_from_shares` uses floor, so tiny share positions may display as `0`.
  - `shares_from_rusd` uses ceil, preventing mint/transfer of zero-share amounts.
- **Underlying balance changes between calls**
  - No cached rate; each call recomputes based on live USDC balance.
  - Rebases are implicit: any external USDC inflow changes exchange rate.

## Overflow and Precision Constraints
- Use `i128` for amounts and shares (Soroban standard).
- Multiply before divide; check for overflow on `amount * total_shares` and `shares * underlying`.
- No fixed-point scaling beyond token decimals; `decimals` mirrors USDC to preserve precision.
