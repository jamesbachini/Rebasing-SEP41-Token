# Testing Strategy

## Fixtures / Mocks
- **USDC mock token**: SEP-41 compatible token with `mint`, `transfer`, `transfer_from`, and `balance` for controlled balances.
- **rUSD contract instance**: Initialized with USDC mock address and metadata.
- **Test accounts**: `alice`, `bob`, `spender`, `contract`.

## Test Cases

### Math and Conversion
- **Round-trip conversion**
  - Setup: `total_shares = 1000`, `underlying = 1000`.
  - Action: `shares_from_rusd(101)` then `rusd_from_shares(result)`.
  - Expect: result `>= 101` on shares input, and `<= 101` on rUSD output (ceil/floor behavior).
- **First mint bootstrap**
  - Setup: `total_shares = 0`, `underlying = 0`.
  - Action: mint `100` rUSD (with USDC transferred in).
  - Expect: `shares_to_mint == 100`, `total_shares == 100`, `balance(alice) == 100`.
- **Zero supply balance query**
  - Setup: `total_shares = 0`.
  - Action: `balance(alice)`.
  - Expect: `0`, no divide-by-zero.

### Mint / Burn
- **Mint with allowance**
  - Setup: Alice has 1000 USDC, approves rUSD for 200.
  - Action: `mint(200)`.
  - Expect: Alice USDC `-200`, contract USDC `+200`, `shares[alice]` increases, `total_shares` increases.
- **Burn with sufficient shares**
  - Setup: Alice has `shares` representing 200 rUSD.
  - Action: `burn(200)`.
  - Expect: Alice shares decrease, `total_shares` decreases, USDC transferred to Alice.
- **Zero amount mint/burn**
  - Setup: none.
  - Action: `mint(0)` and `burn(0)`.
  - Expect: both fail.
- **Burn exceeds balance**
  - Setup: Alice has shares worth 50 rUSD.
  - Action: `burn(100)`.
  - Expect: fail with insufficient shares.

### Transfers and Allowances
- **Transfer moves shares**
  - Setup: Alice holds shares for 100 rUSD.
  - Action: `transfer(alice, bob, 40)`.
  - Expect: shares move so `balance(alice) == 60`, `balance(bob) == 40` (subject to rounding).
- **Approve / allowance**
  - Setup: Alice approves `spender` for 75 rUSD.
  - Action: `allowance(alice, spender)`.
  - Expect: returns 75.
- **Transfer_from enforces allowance**
  - Setup: Alice approves `spender` for 50 rUSD.
  - Action: `transfer_from(spender, alice, bob, 60)`.
  - Expect: fails for insufficient allowance.
- **Transfer_from decrements allowance**
  - Setup: Alice approves `spender` for 50 rUSD.
  - Action: `transfer_from(spender, alice, bob, 20)`.
  - Expect: allowance becomes 30.

### Rebasing Scenarios
- **External USDC inflow**
  - Setup: Alice 100 rUSD shares, Bob 50 rUSD shares, contract USDC 150.
  - Action: transfer 15 USDC directly to contract.
  - Expect: `total_shares` unchanged; `balance(alice) == 110`, `balance(bob) == 55`.
- **Rebase preserves shares**
  - Setup: capture `total_shares`.
  - Action: external USDC inflow.
  - Expect: `total_shares` unchanged.
- **Rounding on small amounts**
  - Setup: `total_shares = 3`, `underlying = 10`.
  - Action: `transfer(alice, bob, 1)`.
  - Expect: share conversion uses ceil; no new shares are created; balances remain consistent with floor on outputs.

## Cross-Test Invariants
- **Share conservation**: transfers do not change `total_shares`.
- **Supply derives from shares**: `total_supply()` equals `rusd_from_shares(total_shares)`.
- **Allowance in rebased units**: allowance checks compare against rebased amount inputs.
- **No implicit rebase**: balances change only via exchange-rate shift from `underlying` changes.
