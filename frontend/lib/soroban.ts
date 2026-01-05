import {
  BASE_FEE,
  Contract,
  Networks,
  TransactionBuilder,
  scValToNative,
  nativeToScVal,
  rpc
} from "@stellar/stellar-sdk";

import { config, NetworkKey } from "./config";

type I128Like = bigint | number;

export function getNetworkPassphrase(network: NetworkKey) {
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

export function getServer() {
  const allowHttp = config.rpcUrl.startsWith("http://");
  return new rpc.Server(config.rpcUrl, { allowHttp });
}

export function toScValI128(value: I128Like) {
  const val = typeof value === "bigint" ? value : BigInt(value);
  return nativeToScVal(val, { type: "i128" });
}

export function toScValU32(value: number) {
  return nativeToScVal(value, { type: "u32" });
}

export function toScValAddress(address: string) {
  return nativeToScVal(address, { type: "address" });
}

export async function readContractValue(
  contractId: string,
  method: string,
  args: ReturnType<typeof nativeToScVal>[],
  source: string,
  network: NetworkKey
) {
  const server = getServer();
  const account = await server.getAccount(source);
  const contract = new Contract(contractId);
  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: getNetworkPassphrase(network)
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(30)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(sim)) {
    throw new Error(sim.error);
  }
  const retval = sim.result?.retval;
  if (!retval) {
    return null;
  }
  return scValToNative(retval);
}

export async function submitContractCall(params: {
  contractId: string;
  method: string;
  args: ReturnType<typeof nativeToScVal>[];
  source: string;
  network: NetworkKey;
  sign: (xdr: string, networkPassphrase: string) => Promise<string>;
}) {
  const server = getServer();
  const account = await server.getAccount(params.source);
  const contract = new Contract(params.contractId);
  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: getNetworkPassphrase(params.network)
  })
    .addOperation(contract.call(params.method, ...params.args))
    .setTimeout(120)
    .build();

  const prepared = await server.prepareTransaction(tx);
  const signedXdr = await params.sign(
    prepared.toXDR(),
    getNetworkPassphrase(params.network)
  );
  const signed = TransactionBuilder.fromXDR(signedXdr, getNetworkPassphrase(params.network));
  const send = await server.sendTransaction(signed);
  if (send.status === "ERROR") {
    throw new Error(send.errorResultXdr || "Transaction failed");
  }
  if (!send.hash) {
    throw new Error("Missing transaction hash");
  }

  for (let i = 0; i < 20; i += 1) {
    await new Promise((resolve) => setTimeout(resolve, 1000));
    const result = await server.getTransaction(send.hash);
    if (result.status === "SUCCESS") {
      return result;
    }
    if (result.status === "FAILED") {
      throw new Error(result.resultXdr || "Transaction failed");
    }
  }

  throw new Error("Transaction timed out");
}

export async function getLatestLedger(network: NetworkKey) {
  const server = getServer();
  const latest = await server.getLatestLedger();
  return latest.sequence ?? 0;
}
