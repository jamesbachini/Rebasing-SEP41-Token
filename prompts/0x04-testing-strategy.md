# 0x04 Testing Strategy

## Intent
Plan a focused test suite that validates rebasing behavior and SEP-41 compliance.

## Read First
- `prompts/AGENTS.md` for default rounding and flow choices.
- `prompts/0x02-storage-and-rebasing-math.md` for conversion rules.
- `prompts/0x03-token-interface-and-flows.md` for method behaviors.

## Scope
- Unit tests for share math and exchange-rate computation.
- Mint/burn tests with USDC mock or test token.
- Rebase scenario tests when extra USDC arrives.
 - Basic SEP-41 behavior checks (transfer, approve, allowance).

## Out of Scope
- Full frontend or wallet integration tests.
- Performance/load tests.
 - Fuzzing or formal verification.

## Required Output
Produce a concise test plan that includes:
- A list of unit test cases grouped by concern (math, mint/burn, transfers, rebasing).
- For each test case, the setup, action, and expected result.
- Any fixtures or mocks required (e.g., USDC mock token contract).
- A minimal set of invariants checked across tests.

## Test Case Guidance (Non-Exhaustive)
**Math and Conversion**
- `shares_from_rusd` and `rusd_from_shares` round-trip with defined rounding.
- First mint when `total_shares == 0` sets an initial exchange rate.
- Zero-supply balance queries return 0 without division errors.

**Mint/Burn**
- Mint with allowance: USDC decreases from user, increases in contract, shares increase.
- Burn with sufficient shares: shares decrease, USDC transferred out.
- Mint/burn with zero amount fails.
- Burn more than balance fails.

**Transfers and Allowances**
- `transfer` moves shares equivalent to rebased amount.
- `approve`/`allowance` reflect rebased units and are enforced in `transfer_from`.
- `transfer_from` decrements allowance appropriately.

**Rebase Scenarios**
- External USDC transfer increases exchange rate; balances rise pro-rata.
- Rebase does not change `total_shares`.
- Small amounts handle rounding without creating extra value.

## Acceptance Criteria
- A minimal but complete list of test cases with pass/fail expectations.
- Coverage of edge cases: zero supply, rounding, small amounts.
- Tests explicitly validate that balances are computed from shares + underlying.

## Future Extensions
- Integration tests with live or local network.
- Snapshot tests for frontend display.
