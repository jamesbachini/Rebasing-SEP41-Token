# 0x03 Token Interface and Mint/Burn Flows

## Intent
Define how the SEP-41 interface behaves with rebasing shares and specify mint/burn flows.

## Read First
- `prompts/AGENTS.md` for decision defaults.
- `prompts/0x01-contract-architecture.md` and `prompts/0x02-storage-and-rebasing-math.md` for interface/internal separation and math conventions.

## Scope
- Mapping of SEP-41 `transfer`, `balance`, `approve`, `allowance` to shares.
- Mint and burn flows using USDC transfer_from and transfer.
- Error conditions and invariant checks for supply and balances.

## Out of Scope
- Exact storage field definitions.
- End-to-end tests and integration tests.
- Frontend UX details.
- Detailed math derivations; reference the formulas from `prompts/0x02-storage-and-rebasing-math.md`.

## Required Output
Produce a concise interface behavior spec that includes:
- A SEP-41 method-by-method mapping that clarifies:
  - input units (rebased rUSD)
  - internal conversion (shares)
  - which state is mutated
- A mint flow and a burn flow, each described as ordered steps.
- A list of failure conditions with brief reasons.
- The invariants that must hold after each operation (supply, shares, and allowance).

## Interface Behavior (Guidance)
- `balance(address)`: returns rebased rUSD computed from shares and the current exchange rate.
- `transfer(from, to, amount)`: interpret `amount` as rebased rUSD; convert to shares, then move shares.
- `approve(spender, amount)` / `allowance(owner, spender)`: amounts in rebased rUSD units; allowances are stored/checked in rebased units unless explicitly decided otherwise.
- `transfer_from(spender, from, to, amount)`: same as `transfer`, plus allowance checks and updates.

## Mint/Burn Flow (Guidance)
- **Mint**: user authorizes USDC, contract pulls USDC via `transfer_from`, computes shares to mint, updates shares and total_shares.
- **Burn**: user burns rebased rUSD amount, contract computes shares to burn and USDC out, updates shares and total_shares, transfers USDC out.
- Ordering should follow defaults in `prompts/AGENTS.md` unless explicitly overridden.

## Error Conditions (Non-Exhaustive)
- Zero amount inputs.
- Insufficient shares for transfer or burn.
- Insufficient allowance for `transfer_from`.
- Zero total_shares when converting from shares to rUSD (read-only cases should handle gracefully).
- Underflow/overflow or precision loss beyond acceptable thresholds.

## Acceptance Criteria
- Clear treatment of amounts as rebased rUSD units at the interface boundary.
- A defined conversion between external amount inputs and internal shares.
- A list of failure cases (insufficient allowance, zero amount, etc.).
- The mint/burn sequences are explicit, including when allowances and balances are checked.

## Future Extensions
- A prompt focused on invariant tests and property checks.
- A prompt for gas and compute optimizations, if needed.
