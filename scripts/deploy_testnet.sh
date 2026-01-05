#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACTS_DIR="${ROOT_DIR}/contracts"
ENV_FILE="${ROOT_DIR}/.env"

NETWORK="testnet"
RPC_URL="https://rpc-testnet.stellar.org"
PASSPHRASE="Test SDF Network ; September 2015"
USDC_CONTRACT_ID="CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"

IDENTITY="${SOROBAN_IDENTITY:-rusd-deployer}"
SECRET_KEY="${SOROBAN_SECRET_KEY:-}"

if ! command -v soroban >/dev/null 2>&1; then
  echo "soroban CLI not found. Install it first."
  exit 1
fi

if ! soroban network ls | rg -q "^${NETWORK}\\b"; then
  soroban network add \
    --network "${NETWORK}" \
    --rpc-url "${RPC_URL}" \
    --network-passphrase "${PASSPHRASE}"
fi

if [[ -n "${SECRET_KEY}" ]]; then
  soroban keys add --secret-key "${SECRET_KEY}" "${IDENTITY}" --overwrite
else
  if ! soroban keys ls | rg -q "^${IDENTITY}\\b"; then
    echo "Missing identity '${IDENTITY}'. Set SOROBAN_SECRET_KEY or create the identity."
    exit 1
  fi
fi

echo "Building contract..."
soroban contract build --manifest-path "${CONTRACTS_DIR}/Cargo.toml"

WASM_PATH="${CONTRACTS_DIR}/target/wasm32-unknown-unknown/release/rusd_rebasing_token.wasm"
if [[ ! -f "${WASM_PATH}" ]]; then
  echo "WASM not found at ${WASM_PATH}"
  exit 1
fi

echo "Deploying contract..."
RUSD_CONTRACT_ID="$(
  soroban contract deploy \
    --wasm "${WASM_PATH}" \
    --source "${IDENTITY}" \
    --network "${NETWORK}"
)"

echo "Initializing contract..."
soroban contract invoke \
  --id "${RUSD_CONTRACT_ID}" \
  --source "${IDENTITY}" \
  --network "${NETWORK}" \
  -- \
  init \
  --usdc_contract_id "${USDC_CONTRACT_ID}" \
  --name "rUSD" \
  --symbol "rUSD" \
  --decimals 7

update_env() {
  local key="$1"
  local value="$2"
  local file="$3"
  if [[ ! -f "${file}" ]]; then
    touch "${file}"
  fi
  if rg -q "^${key}=" "${file}"; then
    sed -i "s|^${key}=.*|${key}=${value}|" "${file}"
  else
    echo "${key}=${value}" >> "${file}"
  fi
}

update_env "NEXT_PUBLIC_NETWORK" "${NETWORK}" "${ENV_FILE}"
update_env "NEXT_PUBLIC_RPC_URL" "${RPC_URL}" "${ENV_FILE}"
update_env "NEXT_PUBLIC_USDC_CONTRACT_ID" "${USDC_CONTRACT_ID}" "${ENV_FILE}"
update_env "NEXT_PUBLIC_RUSD_CONTRACT_ID" "${RUSD_CONTRACT_ID}" "${ENV_FILE}"

echo "Deployment complete."
echo "rUSD contract ID: ${RUSD_CONTRACT_ID}"
echo "Updated ${ENV_FILE} for frontend use."
