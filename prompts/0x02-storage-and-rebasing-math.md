# 0x02 Storage and Rebasing Math

## Intent
Define the storage model and the share-to-rUSD math that powers rebasing balances.

## Read First
- `prompts/AGENTS.md` for default rounding and decimal choices.
- `prompts/0x01-contract-architecture.md` for the separation of interface vs. internals.

## Scope
- Storage primitives for `total_shares`, `shares[addr]`, and configuration.
- Conversion formulas between shares and rebased rUSD units.
- Rounding behavior and edge-case handling (zero supply, first mint, dust).
- Explicit constraints around overflow and precision.

## Out of Scope
- Full SEP-41 method definitions or error types.
- Mint/burn flow ordering and allowance checks.
- Frontend display formatting or UX.

## Required Output
Produce a brief design writeup that includes:
- A minimal storage schema with key names or enum-like identifiers and value types.
- Config fields required for math (USDC contract ID, decimals, and any stored rate data if needed).
- Conversion formulas for:
  - `shares_from_rusd(amount)`
  - `rusd_from_shares(shares)`
  - `exchange_rate = underlying / total_shares`
- Rounding rules for each conversion and how they align with the defaults in `prompts/AGENTS.md`.
- Defined behavior for:
  - First mint (when `total_shares == 0`)
  - Zero-balance accounts and dust handling
  - Underlying balance changes between calls

## Design Notes
- Store shares as integers; never store rebased balances.
- `underlying` is read from the USDC contract balance at call time, not cached.
- Use consistent rounding to avoid value creation or destruction across mint/burn and transfer flows.
- Precision must be sufficient for USDC decimals; avoid fractional shares unless explicitly justified.
- Any fixed-point scaling factor must be configurable or derived from token decimals.

## Acceptance Criteria
- The storage schema is minimal yet complete for share accounting and configuration.
- The math is explicit, consistent, and aligns with default rounding decisions.
- The writeup explains how the system behaves at zero supply and during first mint.
- No hard-coded constants are introduced; configuration is explicit.

## Future Extensions
- `prompts/0x03-token-interface-and-flows.md` for interface behavior and mint/burn ordering.
- `prompts/0x04-testing-strategy.md` for math and rounding test cases.
