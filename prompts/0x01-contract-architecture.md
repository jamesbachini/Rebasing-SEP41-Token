# 0x01 Contract Architecture

## Intent
Define the contract’s high-level structure and clarify how a rebasing, share-based token can still present a SEP-41 interface.

## Read First
- `prompts/AGENTS.md` for the rebasing share model and decision defaults.

## Scope
- Identify contract components (entrypoints, internal helpers, storage access, math utilities).
- Explain where rebasing logic lives and how it is used at read/write boundaries.
- Describe how USDC balance is queried to derive the exchange rate.
- Clarify which responsibilities are delegated to OpenZeppelin SEP-41 helpers and which are customized.

## Out of Scope
- Exact storage field definitions or layouts.
- Detailed math formulas or rounding edge cases.
- Concrete method signatures for mint/burn flows.
- Any frontend integration or UX details.

## Required Output
Produce a brief architecture writeup that includes:
- A component map naming each module or logical section and its responsibilities.
- The public entrypoint categories (e.g., SEP-41 surface vs. custom mint/burn surface).
- A description of the control flow for reads (balance queries) and writes (transfer/mint/burn).
- A note on how the USDC token contract is accessed (client usage and data needed).

## Design Notes
- The contract must expose SEP-41 semantics for balances and transfers, but internally operate on shares.
- All user-facing “amounts” are rebased rUSD units; shares are internal only.
- Rebase awareness is **implicit**: exchange rate is derived from the current USDC balance.
- Use OpenZeppelin token libraries as a base; avoid rewriting SEP-41 logic unless required.
- Configuration (USDC contract ID, network) is passed at init or stored as config; no hard-coded constants.

## Acceptance Criteria
- The architecture clearly separates interface logic from share accounting internals.
- The writeup explains how a SEP-41 transfer converts rebased units to shares and updates internal storage.
- It is explicit that no admin-driven rebase exists; balances shift through exchange-rate changes.
- The USDC dependency is described (which token calls are needed and why).

## Future Extensions
- `prompts/0x02-storage-and-rebasing-math.md` for storage layout and formulas.
- `prompts/0x03-token-interface-and-flows.md` for detailed method behavior and mint/burn ordering.
