import type { NetworkKey } from "./config";
import { Networks } from "@stellar/stellar-sdk";

let kitReady: Promise<void> | null = null;

function toWalletNetwork(network: NetworkKey) {
  switch (network) {
    case "mainnet":
      return Networks.PUBLIC;
    case "futurenet":
      return Networks.FUTURENET;
    case "local":
      return Networks.STANDALONE;
    case "testnet":
    default:
      return Networks.TESTNET;
  }
}

async function initWalletKit(network: NetworkKey, networkPassphrase?: string) {
  if (!kitReady) {
    kitReady = (async () => {
      const [{ StellarWalletsKit }, { defaultModules }] = await Promise.all([
        import("@creit-tech/stellar-wallets-kit/sdk"),
        import("@creit-tech/stellar-wallets-kit/modules/utils")
      ]);
      StellarWalletsKit.init({
        modules: defaultModules(),
        network: networkPassphrase ?? toWalletNetwork(network)
      });
    })();
  }
  return kitReady;
}

export async function connectWallet(network: NetworkKey) {
  await initWalletKit(network);
  const { StellarWalletsKit } = await import("@creit-tech/stellar-wallets-kit/sdk");
  const { address } = await StellarWalletsKit.authModal();
  if (!address) {
    throw new Error("Wallet connection canceled.");
  }
  return { publicKey: address };
}

export async function signTransaction(
  xdr: string,
  networkPassphrase: string,
  address: string,
  network: NetworkKey
) {
  await initWalletKit(network, networkPassphrase);
  const { StellarWalletsKit } = await import("@creit-tech/stellar-wallets-kit/sdk");
  const { signedTxXdr } = await StellarWalletsKit.signTransaction(xdr, {
    networkPassphrase,
    address
  });
  return signedTxXdr;
}
