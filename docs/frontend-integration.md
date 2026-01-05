# Frontend Integration Plan

## User Journey
- **Connect**
  - User connects Freighter via creittech v2 wallet kit.
  - App reads active account and network from wallet kit.
- **Approve + Mint**
  - User enters mint amount in rebased rUSD units.
  - App checks USDC allowance for the rUSD contract.
  - If insufficient, prompt approval and submit USDC `approve`.
  - After approval, submit `mint(amount)` on rUSD.
- **Burn**
  - User enters burn amount in rebased rUSD units.
  - App submits `burn(amount)` on rUSD.
  - User receives USDC back at current exchange rate.

## Contract Calls and Timing
- **On connect**
  - `rusd.balance(user)` to display rebased rUSD balance.
  - `usdc.balance(user)` to show wallet USDC.
  - Optional: `usdc.balance(rusd_contract)` to show underlying and exchange rate.
- **Approve flow**
  - `usdc.allowance(user, rusd_contract)` before mint.
  - `usdc.approve(rusd_contract, amount)` if allowance is insufficient.
- **Mint flow**
  - `rusd.mint(amount)` after approval.
- **Burn flow**
  - `rusd.burn(amount)`.

## State Model (Minimal)
- **Wallet state**
  - `connected`, `account`, `network`, `isConnecting`.
- **Balances**
  - `rusdBalance` (rebased), `usdcBalance`, `underlyingBalance` (optional).
  - `exchangeRate` derived client-side as `underlyingBalance / totalShares` if exposed, or shown as contextual info only.
- **Allowances**
  - `usdcAllowance` for rUSD contract.
- **Transaction state**
  - `isApproving`, `isMinting`, `isBurning`, `txHash`, `txError`.

## Balance Freshness Plan
- **Polling**
  - Poll balances on a short interval (e.g., every 10–15s) to capture rebases.
- **Event-driven refresh**
  - After any successful tx (approve/mint/burn), immediately refetch balances.
- **Manual refresh**
  - Provide a refresh action if desired (optional UX).

## Minimal Configuration Surface
- `NEXT_PUBLIC_NETWORK` (e.g., futurenet/testnet/local)
- `NEXT_PUBLIC_USDC_CONTRACT_ID`
- `NEXT_PUBLIC_RUSD_CONTRACT_ID`
- `NEXT_PUBLIC_RPC_URL`
