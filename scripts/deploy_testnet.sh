#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACTS_DIR="${ROOT_DIR}/contracts"
ENV_FILE="${ROOT_DIR}/.env"

NETWORK="testnet"
RPC_URL="https://rpc-testnet.stellar.org"
PASSPHRASE="Test SDF Network ; September 2015"
USDC_CONTRACT_ID="CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"

IDENTITY="${STELLAR_IDENTITY:-${SOROBAN_IDENTITY:-james}}"
SECRET_KEY="${STELLAR_SECRET_KEY:-${SOROBAN_SECRET_KEY:-}}"

if command -v stellar >/dev/null 2>&1; then
  CLI_BIN="stellar"
  SOURCE_FLAG="--source-account"
  WASM_TARGET="wasm32v1-none"
elif command -v soroban >/dev/null 2>&1; then
  CLI_BIN="soroban"
  SOURCE_FLAG="--source"
  WASM_TARGET="wasm32-unknown-unknown"
else
  echo "Stellar CLI not found. Install it first."
  exit 1
fi

if ! ${CLI_BIN} network ls | grep -Eq "^${NETWORK}\\b"; then
  if [[ "${CLI_BIN}" == "stellar" ]]; then
    ${CLI_BIN} network add \
      --rpc-url "${RPC_URL}" \
      --network-passphrase "${PASSPHRASE}" \
      "${NETWORK}"
  else
    ${CLI_BIN} network add \
      --network "${NETWORK}" \
      --rpc-url "${RPC_URL}" \
      --network-passphrase "${PASSPHRASE}"
  fi
fi

if [[ -n "${SECRET_KEY}" ]]; then
  ${CLI_BIN} keys add --secret-key "${SECRET_KEY}" "${IDENTITY}" --overwrite
else
  if ! ${CLI_BIN} keys ls | grep -Eq "^${IDENTITY}\\b"; then
    echo "Missing identity '${IDENTITY}'. Set STELLAR_SECRET_KEY (or SOROBAN_SECRET_KEY) or create the identity."
    exit 1
  fi
fi

echo "Building contract..."
${CLI_BIN} contract build --manifest-path "${CONTRACTS_DIR}/Cargo.toml"

WASM_PATH="${CONTRACTS_DIR}/target/${WASM_TARGET}/release/rusd_rebasing_token.wasm"
if [[ ! -f "${WASM_PATH}" ]]; then
  echo "WASM not found at ${WASM_PATH}"
  exit 1
fi

echo "Deploying contract..."
RUSD_CONTRACT_ID="$(
  ${CLI_BIN} contract deploy \
    --wasm "${WASM_PATH}" \
    ${SOURCE_FLAG} "${IDENTITY}" \
    --network "${NETWORK}"
)"

echo "Initializing contract..."
${CLI_BIN} contract invoke \
  --id "${RUSD_CONTRACT_ID}" \
  ${SOURCE_FLAG} "${IDENTITY}" \
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
  if grep -q "^${key}=" "${file}"; then
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
