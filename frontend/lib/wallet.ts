import type { NetworkKey } from "./config";

let kitPromise: Promise<any> | null = null;

function toWalletNetwork(network: NetworkKey, kitModule: any) {
  const { WalletNetwork } = kitModule;
  switch (network) {
    case "mainnet":
      return WalletNetwork.PUBLIC;
    case "futurenet":
      return WalletNetwork.FUTURENET;
    case "local":
      return WalletNetwork.STANDALONE;
    case "testnet":
    default:
      return WalletNetwork.TESTNET;
  }
}

export async function getWalletKit(network: NetworkKey) {
  if (!kitPromise) {
    kitPromise = import("@creit.tech/stellar-wallets-kit").then((mod) => {
      const { StellarWalletsKit, allowAllModules, WalletType } = mod as any;
      const walletNetwork = toWalletNetwork(network, mod);
      const kit = new StellarWalletsKit({
        network: walletNetwork,
        modules: allowAllModules(),
        selectedWalletId: WalletType.FREIGHTER
      });
      return kit;
    });
  }
  return kitPromise;
}

export async function connectWallet(network: NetworkKey) {
  const kit = await getWalletKit(network);
  if (typeof kit.openModal === "function") {
    await kit.openModal();
  }
  if (typeof kit.setWallet === "function") {
    const { WalletType } = await import("@creit.tech/stellar-wallets-kit");
    await kit.setWallet(WalletType.FREIGHTER);
  }
  const publicKey = await kit.getPublicKey();
  return { kit, publicKey };
}

export async function signTransaction(
  kit: any,
  xdr: string,
  networkPassphrase: string
) {
  if (typeof kit.signTransaction === "function") {
    return kit.signTransaction(xdr, { networkPassphrase });
  }
  if (typeof kit.signTx === "function") {
    return kit.signTx(xdr, networkPassphrase);
  }
  throw new Error("Wallet does not support signing");
}
