# Configuration and Deployment Plan (Testnet)

## Required Configuration Values
- **Contract init args**
  - `usdc_contract_id` (testnet USDC contract address)
  - `name`, `symbol`, `decimals` (decimals mirror USDC)
- **Frontend env vars**
  - `NEXT_PUBLIC_NETWORK=testnet`
  - `NEXT_PUBLIC_USDC_CONTRACT_ID`
  - `NEXT_PUBLIC_RUSD_CONTRACT_ID`
  - `NEXT_PUBLIC_RPC_URL`

## On-Chain Config (Persisted at Init)
- `USDC_CONTRACT_ID`
- `NAME`
- `SYMBOL`
- `DECIMALS`
- Any network selector or chain identifier (if used by the contract logic)

## Testnet Deployment Checklist
1) **Build** the Soroban contract WASM.
2) **Deploy** to testnet and capture the rUSD contract ID.
3) **Initialize** with:
   - `usdc_contract_id` = testnet USDC (CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA)
   - metadata (name/symbol/decimals)
4) **Verify** on-chain config by calling `name`, `symbol`, `decimals`.
5) **Verify** USDC dependency by reading `USDC.balance(rusd_contract)` (should start at 0).
6) **Frontend setup**:
   - Populate `.env.local` with testnet values and the deployed rUSD contract ID.
7) **Smoke test**:
   - Approve USDC, mint a small amount of rUSD, and confirm balances update.
   - Burn a small amount and confirm USDC returns.
