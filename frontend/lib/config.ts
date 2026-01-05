export type NetworkKey = "testnet" | "futurenet" | "mainnet" | "local";

export const config = {
  network: (process.env.NEXT_PUBLIC_NETWORK || "testnet") as NetworkKey,
  rpcUrl: process.env.NEXT_PUBLIC_RPC_URL || "",
  usdcContractId: process.env.NEXT_PUBLIC_USDC_CONTRACT_ID || "",
  rusdContractId: process.env.NEXT_PUBLIC_RUSD_CONTRACT_ID || ""
};

export function assertConfig() {
  const missing = Object.entries(config)
    .filter(([, value]) => !value)
    .map(([key]) => key);
  if (missing.length > 0) {
    return `Missing env values: ${missing.join(", ")}`;
  }
  return "";
}
