# Token Interface and Mint/Burn Flows

## SEP-41 Method Mapping (Rebased Units → Shares)
- **balance(owner)**:
  - Input units: none.
  - Internal: `rusd_from_shares(shares[owner])` using live `underlying`.
  - State: read-only.
- **total_supply()**:
  - Input units: none.
  - Internal: `rusd_from_shares(total_shares)`.
  - State: read-only.
- **transfer(from, to, amount)**:
  - Input units: rebased rUSD.
  - Internal: `shares = shares_from_rusd(amount)`, then move `shares`.
  - State: `shares[from]`, `shares[to]`.
- **approve(owner, spender, amount)**:
  - Input units: rebased rUSD.
  - Internal: store allowance in rebased units via OpenZeppelin helpers.
  - State: allowance storage only.
- **allowance(owner, spender)**:
  - Input units: none.
  - Internal: read allowance in rebased units.
  - State: read-only.
- **transfer_from(spender, from, to, amount)**:
  - Input units: rebased rUSD.
  - Internal: check/update allowance in rebased units; `shares = shares_from_rusd(amount)`; move shares.
  - State: allowance storage, `shares[from]`, `shares[to]`.

## Mint Flow (Ordered Steps)
1) Validate `amount > 0`.
2) Pull USDC in via `USDC.transfer_from(user, contract, amount)` after user approval.
3) Compute `shares_to_mint = shares_from_rusd(amount)` using live `underlying`.
4) Increase `shares[user]` and `total_shares` by `shares_to_mint`.
5) Emit standard SEP-41 transfer event (if supported by library) from `None` to `user` in rebased units.

## Burn Flow (Ordered Steps)
1) Validate `amount > 0`.
2) Compute `shares_to_burn = shares_from_rusd(amount)` using live `underlying`.
3) Ensure `shares[user] >= shares_to_burn`.
4) Decrease `shares[user]` and `total_shares` by `shares_to_burn`.
5) Compute `usdc_out = rusd_from_shares(shares_to_burn)` and transfer via `USDC.transfer(contract, user, usdc_out)`.
6) Emit standard SEP-41 transfer event from `user` to `None` in rebased units.

## Failure Conditions (Non-Exhaustive)
- **Zero amount**: reject `amount == 0` for transfer/mint/burn.
- **Insufficient shares**: `shares[from] < shares_needed` for transfer/burn.
- **Insufficient allowance**: `allowance(from, spender) < amount` for `transfer_from`.
- **Zero supply edge**: converting shares to rUSD when `total_shares == 0` should return `0` (read-only).
- **USDC transfer failure**: underlying token transfer/transfer_from fails or is unauthorized.
- **Arithmetic errors**: overflow/underflow on multiply/divide or share updates.

## Post-Operation Invariants
- **Share conservation**:
  - `transfer` and `transfer_from` move shares without changing `total_shares`.
- **Supply reflects shares**:
  - `total_supply()` equals `rusd_from_shares(total_shares)` under the current exchange rate.
- **Mint/Burn supply updates**:
  - `mint` increases both `shares[user]` and `total_shares` by the same share amount.
  - `burn` decreases both `shares[user]` and `total_shares` by the same share amount.
- **Allowance semantics**:
  - Allowances are stored and enforced in rebased rUSD units (no share conversion).
