# rUSD — Rebasing USD Token on Stellar (Soroban)

rUSD is a **rebasing SEP-41 token** (ERC-20 equivalent on Stellar) built with **Soroban** and **OpenZeppelin token libraries**.  
It is backed 1:1 by **USDC** and designed to automatically distribute yield by rebasing balances based on the amount of underlying USDC held by the contract.

This repo contains:
- A Soroban smart contract implementing a share-based rebasing token
- Unit tests covering minting, burning, transfers, and rebasing
- A simple Next.js frontend using **creittech v2 wallet kit** to interact with the contract via **Freighter**

---

## Core Idea

Instead of tracking balances directly, rUSD tracks **shares**.

- Users hold shares
- Total shares only change on mint/burn
- The **displayed rUSD balance** is derived from:

```

user_rUSD = user_shares × (underlying_USDC / total_shares)

```

Because balances are *derived*, if more USDC enters the contract, **all balances increase automatically** — no state updates required.

### Example

| User | Shares | rUSD (initial) |
|-----|-------|----------------|
| A | 100 | 100 |
| B | 50 | 50 |
| **Total** | 150 | 150 |

If **+15 USDC** is sent directly to the contract:

| User | Shares | rUSD (after rebase) |
|-----|-------|---------------------|
| A | 100 | 110 |
| B | 50 | 55 |

---

## Features

- ✅ SEP-41 compatible token interface
- ✅ Mint rUSD by depositing USDC
- ✅ Burn rUSD to redeem USDC
- ✅ Automatic rebasing via share accounting
- ✅ Permissionless (no admin required)
- ✅ Yield simulation by transferring USDC directly to the contract
- ✅ React / Next.js frontend with Freighter support

---

## Repository Structure

```

.
├── contracts/
│   ├── src/
│   │   ├── main.rs      # rUSD Soroban contract
│   │   └── test.rs      # Unit tests
│   └── Cargo.toml
│
├── frontend/
│   ├── app/ or pages/   # Next.js frontend
│   ├── components/
│   ├── lib/
│   └── README.md
│
├── prompts/
│   ├── AGENTS.md        # High-level architecture + decisions
│   ├── 0x01-contract.md
│   ├── 0x02-unittests.md
│   └── 0x03-frontend.md
│
└── README.md

````

---

## Smart Contract Overview

### Key Concepts

- **Shares**: Internal accounting unit
- **Underlying**: Actual USDC balance of the contract
- **Rebased balance**: Derived from shares × exchange rate
- **Exchange rate**: `underlying_USDC / total_shares`

### Main Methods

| Method | Description |
|-----|------------|
| `init` | Initialize contract with USDC address + metadata |
| `balance(address)` | Returns rebased rUSD balance |
| `mint(amount)` | Pulls USDC and mints rUSD |
| `burn(amount)` | Burns rUSD and returns USDC |
| `transfer` | Transfers rebased rUSD |
| `approve` | Approves allowance (rebased units) |
| `transfer_from` | Transfers using allowance |

---

## Unit Tests

Tests validate:

- 1:1 minting bootstrap
- Multi-user minting
- Pro-rata rebasing after extra USDC is sent
- Transfer correctness under rebasing
- Burn redemption at current exchange rate
- Allowance + `transfer_from`

### Run Tests

```bash
cd contracts
cargo test
````

---

## Frontend

The frontend is a **minimal Next.js app** that allows you to:

* Connect a **Freighter** wallet
* View USDC and rUSD balances
* Approve USDC spending
* Mint rUSD
* Burn rUSD
* Observe rebasing after yield simulation

### Environment Variables

Create `frontend/.env.local`:

```env
NEXT_PUBLIC_NETWORK=futurenet
NEXT_PUBLIC_USDC_CONTRACT_ID=...
NEXT_PUBLIC_RUSD_CONTRACT_ID=...
NEXT_PUBLIC_RPC_URL=https://rpc-futurenet.stellar.org
```

### Run Frontend

```bash
cd frontend
npm install
npm run dev
```

---

## Yield Simulation (Important)

There is **no yield strategy yet**.

To simulate yield:

1. Copy the **rUSD contract address**
2. Send USDC **directly** to that address
3. Refresh balances in the frontend
4. All rUSD holders will see balances increase automatically

This mirrors how future vault or yield logic will work.

---

## Design Goals

* Minimal, readable, auditable
* No hidden rebasing logic
* No cron jobs or admin calls
* Purely derived balances
* Safe rounding (flooring in favor of the contract)

---

## Future Work

* 🔜 Plug underlying USDC into a yield vault
* 🔜 Automated yield harvesting
* 🔜 ERC-4626-style vault interface
* 🔜 Indexer-friendly events
* 🔜 Better UX around approvals + max mint
* 🔜 Permit / signature-based approvals

---

## Disclaimer

This project is **experimental** and intended for learning, prototyping, and demonstration purposes.
It has **not been audited**.

Use at your own risk.

---

## License

MIT

