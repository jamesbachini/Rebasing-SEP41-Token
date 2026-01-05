# rUSD Rebasing Token (Soroban) — Prompt Workflow Root

This file defines the evolving prompt-driven workflow for building rUSD, a rebasing SEP-41 token on Stellar using Soroban and OpenZeppelin token libraries. It is a living guide; future prompts may refine or override parts of this plan.

## System Architecture (Overview)
- **Core contract**: A Soroban smart contract that implements SEP-41 behavior while using **share-based accounting** for rebasing balances.
- **Backing asset**: The contract holds USDC directly; users mint/burn rUSD 1:1 relative to the current exchange rate.
- **Frontend**: A minimal Next.js app using creittech v2 wallet kit + Freighter to approve USDC, mint/burn rUSD, and display balances.

## Rebasing Share Model (Concept)
- Track `total_shares` and `shares[addr]` in storage.
- Fetch `underlying` as the current USDC balance of the contract.
- User-visible rUSD balance is computed as:
  - `balance_rusd(addr) = shares[addr] * underlying / total_shares`
- Minting and burning operate in **shares**, while user inputs are in **rebased rUSD units**.
- Rebases are **automatic**: if extra USDC is sent to the contract, the exchange rate rises without updating stored balances.

## Repo Structure and Conventions
- `contracts/`: Soroban contract, storage logic, and tests.
- `frontend/`: Next.js app for wallet connect, approve, mint/burn, and balance display.
- `prompts/`: This workflow and all prompt files.
- Prompts are the source of truth for scoped tasks; avoid mixing responsibilities across prompts.

## Configuration Strategy (No Hard-Coded Constants)
- USDC contract ID and network selection must be provided at deploy/init time or via environment configuration.
- The contract should accept configuration via an initializer and/or persistent config storage.
- The frontend should read network and contract IDs from env vars or a runtime config file.

## Decision Checklist (Multiple-Choice)
If the user does not answer, use the defaults marked **(default)**.

1) **Decimal handling for displayed rUSD values**
- A) Mirror USDC decimals **(default)**
- B) Fixed 7 decimals
- C) Fixed 18 decimals

2) **Rounding behavior for share-to-rUSD conversion**
- A) Floor on outputs, ceil on inputs **(default)**
- B) Always floor
- C) Always ceil

3) **Mint flow order**
- A) Transfer USDC in, then mint shares **(default)**
- B) Mint shares, then transfer USDC in

4) **Burn flow order**
- A) Burn shares, then transfer USDC out **(default)**
- B) Transfer USDC out, then burn shares

5) **Allowance mechanism for USDC transfers**
- A) Standard SEP-41 approve + transfer_from **(default)**
- B) Permit-style (if available)

6) **Testing scope for initial pass**
- A) Unit tests only **(default)**
- B) Unit + minimal integration tests

## How to Add Future Prompts
- Create a new markdown file in `prompts/` with a short, descriptive name.
- Include: intent, scope, non-scope, acceptance criteria, and future extensions.
- If the prompt depends on previous prompts, say so explicitly and list the assumptions.
- Prefer smaller prompts that unlock incremental changes.

## Workflow is Evolving
This workflow is expected to change as the system design is refined. New prompts should be added to handle emerging complexity.
