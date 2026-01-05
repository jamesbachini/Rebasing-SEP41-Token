# 0x05 Frontend Integration

## Intent
Define the frontend's minimal flows for connecting a wallet, approving USDC, minting/burning rUSD, and displaying rebased balances.

## Read First
- `prompts/AGENTS.md` for config defaults and decision checklist.
- `prompts/0x03-token-interface-and-flows.md` for interface behavior and units.

## Scope
- Wallet connection flow with Freighter via creittech v2 wallet kit.
- UI states for approve, mint, burn, and balance display.
- Data needed from the contract for balance and exchange rate display.
 - Minimal configuration surface (contract IDs, network, token decimals).

## Out of Scope
- Visual design details and styling.
- Advanced error recovery and analytics.
 - Full multi-network switching UX (unless required by config).

## Required Output
Produce a concise frontend integration plan that includes:
- The user journey for mint and burn, including required approvals.
- The list of contract calls and when they are invoked.
- The state model for balances and loading/tx states.
- A plan for keeping displayed rebased balances fresh (polling or event-driven).

## Flow Guidance (Non-Exhaustive)
**Connect**
- Connect Freighter with creittech v2 wallet kit.
- Read active account and network info.

**Approve + Mint**
- If USDC allowance < desired mint amount, prompt approval.
- After approval, call mint with the rebased amount.

**Burn**
- Call burn with rebased amount; receive USDC out.

**Balance Display**
- Read rUSD balance (rebased) from the rUSD contract.
- Read underlying USDC balance for context and exchange-rate display if desired.
- Refresh balances after tx and on a periodic interval.

## Acceptance Criteria
- A clear user flow for approve → mint and burn → receive USDC.
- Defined calls the frontend must make to the contract.
- A plan for showing rebased balances that change with underlying.
 - States and transitions are explicit enough to implement without UI design.

## Future Extensions
- UX refinements, error handling, and transaction history.
- Multi-network support details if not already decided.
