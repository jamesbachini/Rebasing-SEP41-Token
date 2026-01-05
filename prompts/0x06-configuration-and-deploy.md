# 0x06 Configuration and Deployment

## Intent
Define how contract and frontend configuration is provided and how deployment will be staged for testnet.

## Read First
- `prompts/AGENTS.md` for config requirements and no hard-coded constants.

## Scope
- Required configuration values (network, USDC contract ID, rUSD contract ID).
- Init-time parameters and persistent config storage.
- Deployment stage: testnet only.

## Out of Scope
- CI/CD pipelines.
- Full automation scripts for deployment.
- Local or production deployments.

## Required Output
Produce a concise configuration and deployment plan that includes:
- A list of required config values and where they live (contract init args, env vars, or config file).
- The contract initialization parameters and what gets persisted on-chain.
- A minimal checklist for a testnet deployment and verification steps.

## Acceptance Criteria
- A simple, consistent configuration scheme for both contract and frontend.
- A minimal deployment checklist for initial testnet release.
- No mention of local or production deploy steps.

## Future Extensions
- Environment-specific config tooling.
- Key management and signing UX improvements.
