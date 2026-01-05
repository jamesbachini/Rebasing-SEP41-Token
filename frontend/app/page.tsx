"use client";

import { useCallback, useEffect, useMemo, useState } from "react";

import { config, assertConfig } from "../lib/config";
import { formatAmount, parseAmount, shorten } from "../lib/format";
import { connectWallet, signTransaction } from "../lib/wallet";
import {
  getLatestLedger,
  readContractValue,
  submitContractCall,
  toScValAddress,
  toScValI128,
  toScValU32
} from "../lib/soroban";

type Bigish = bigint | null;

export default function HomePage() {
  const [account, setAccount] = useState<string>("");
  const [status, setStatus] = useState<string>("");
  const [error, setError] = useState<string>("");
  const [busy, setBusy] = useState<boolean>(false);
  const [decimals, setDecimals] = useState<number>(7);

  const [rusdBalance, setRusdBalance] = useState<Bigish>(null);
  const [usdcBalance, setUsdcBalance] = useState<Bigish>(null);
  const [usdcAllowance, setUsdcAllowance] = useState<Bigish>(null);
  const [underlyingBalance, setUnderlyingBalance] = useState<Bigish>(null);
  const [totalSupply, setTotalSupply] = useState<Bigish>(null);

  const [mintAmount, setMintAmount] = useState<string>("");
  const [burnAmount, setBurnAmount] = useState<string>("");

  const envIssue = useMemo(() => assertConfig(), []);

  const needsApproval = useMemo(() => {
    if (usdcAllowance === null) {
      return true;
    }
    const amount = parseAmount(mintAmount, decimals);
    return amount > 0n && usdcAllowance < amount;
  }, [usdcAllowance, mintAmount, decimals]);

  const exchangeRate = useMemo(() => {
    if (!underlyingBalance || !totalSupply || totalSupply === 0n) {
      return "1.000000";
    }
    const scaled = (underlyingBalance * 1_000_000n) / totalSupply;
    return formatAmount(scaled, 6);
  }, [underlyingBalance, totalSupply]);

  const refreshBalances = useCallback(async () => {
    if (!account) {
      return;
    }
    try {
      const [rusdBal, usdcBal, allowance, underlying, supply, dec] = await Promise.all([
        readContractValue(
          config.rusdContractId,
          "balance",
          [toScValAddress(account)],
          account,
          config.network
        ),
        readContractValue(
          config.usdcContractId,
          "balance",
          [toScValAddress(account)],
          account,
          config.network
        ),
        readContractValue(
          config.usdcContractId,
          "allowance",
          [toScValAddress(account), toScValAddress(config.rusdContractId)],
          account,
          config.network
        ),
        readContractValue(
          config.usdcContractId,
          "balance",
          [toScValAddress(config.rusdContractId)],
          account,
          config.network
        ),
        readContractValue(config.rusdContractId, "total_supply", [], account, config.network),
        readContractValue(config.rusdContractId, "decimals", [], account, config.network)
      ]);

      setRusdBalance(toBigInt(rusdBal));
      setUsdcBalance(toBigInt(usdcBal));
      setUsdcAllowance(toBigInt(allowance));
      setUnderlyingBalance(toBigInt(underlying));
      setTotalSupply(toBigInt(supply));
      const decValue = toNumber(dec);
      if (decValue !== null) {
        setDecimals(decValue);
      }
    } catch (err: any) {
      setError(err?.message || "Failed to refresh balances");
    }
  }, [account]);

  useEffect(() => {
    if (!account) {
      return;
    }
    refreshBalances();
    const handle = setInterval(refreshBalances, 12_000);
    return () => clearInterval(handle);
  }, [account, refreshBalances]);

  const onConnect = async () => {
    setError("");
    setStatus("Connecting wallet...");
    try {
      const { publicKey } = await connectWallet(config.network);
      setAccount(publicKey);
      setStatus("Wallet connected.");
      await refreshBalances();
    } catch (err: any) {
      setError(err?.message || "Failed to connect wallet");
      setStatus("");
    }
  };

  const onDisconnect = () => {
    setAccount("");
    setStatus("");
    setRusdBalance(null);
    setUsdcBalance(null);
    setUsdcAllowance(null);
    setUnderlyingBalance(null);
    setTotalSupply(null);
  };

  const onApprove = async () => {
    if (!account) {
      return;
    }
    const amount = parseAmount(mintAmount, decimals);
    if (amount <= 0n) {
      setError("Enter an amount to approve.");
      return;
    }
    setBusy(true);
    setError("");
    setStatus("Submitting approval...");
    try {
      const latestLedger = await getLatestLedger(config.network);
      const expirationLedger = latestLedger + 100_000;
      await submitContractCall({
        contractId: config.usdcContractId,
        method: "approve",
        args: [
          toScValAddress(account),
          toScValAddress(config.rusdContractId),
          toScValI128(amount),
          toScValU32(expirationLedger)
        ],
        source: account,
        network: config.network,
        sign: (xdr, passphrase) =>
          signTransaction(xdr, passphrase, account, config.network)
      });
      setStatus("Approval confirmed.");
      await refreshBalances();
    } catch (err: any) {
      setError(err?.message || "Approval failed.");
    } finally {
      setBusy(false);
    }
  };

  const onMint = async () => {
    if (!account) {
      return;
    }
    const amount = parseAmount(mintAmount, decimals);
    if (amount <= 0n) {
      setError("Enter a mint amount.");
      return;
    }
    setBusy(true);
    setError("");
    setStatus("Minting rUSD...");
    try {
      await submitContractCall({
        contractId: config.rusdContractId,
        method: "mint",
        args: [toScValAddress(account), toScValI128(amount)],
        source: account,
        network: config.network,
        sign: (xdr, passphrase) =>
          signTransaction(xdr, passphrase, account, config.network)
      });
      setStatus("Mint confirmed.");
      setMintAmount("");
      await refreshBalances();
    } catch (err: any) {
      setError(err?.message || "Mint failed.");
    } finally {
      setBusy(false);
    }
  };

  const onBurn = async () => {
    if (!account) {
      return;
    }
    const amount = parseAmount(burnAmount, decimals);
    if (amount <= 0n) {
      setError("Enter a burn amount.");
      return;
    }
    setBusy(true);
    setError("");
    setStatus("Burning rUSD...");
    try {
      await submitContractCall({
        contractId: config.rusdContractId,
        method: "burn",
        args: [toScValAddress(account), toScValI128(amount)],
        source: account,
        network: config.network,
        sign: (xdr, passphrase) =>
          signTransaction(xdr, passphrase, account, config.network)
      });
      setStatus("Burn confirmed.");
      setBurnAmount("");
      await refreshBalances();
    } catch (err: any) {
      setError(err?.message || "Burn failed.");
    } finally {
      setBusy(false);
    }
  };

  return (
    <main>
      <div className="page">
        <section className="hero">
          <div>
            <div className="pill">rUSD Rebasing Token</div>
            <h1>Yield appears as balance growth.</h1>
            <p>
              Deposit USDC, mint rUSD, and watch balances expand automatically when
              underlying USDC lands in the contract.
            </p>
          </div>
          <div className="panel">
            <h2>Wallet</h2>
            <div className="stat">
              <span className="label">Network</span>
              <span className="value">{config.network}</span>
            </div>
            <div className="stat">
              <span className="label">Account</span>
              <span className="value">{account ? shorten(account) : "Not connected"}</span>
            </div>
            <div className="split">
              {account ? (
                <button className="ghost" onClick={onDisconnect}>
                  Disconnect
                </button>
              ) : (
                <button className="secondary" onClick={onConnect}>
                  Connect Wallet
                </button>
              )}
              <button className="ghost" onClick={refreshBalances} disabled={!account}>
                Refresh
              </button>
            </div>
          </div>
        </section>

        {envIssue ? <div className="status">{envIssue}</div> : null}
        {status ? <div className="status">{status}</div> : null}
        {error ? <div className="status">{error}</div> : null}

        <section className="grid">
          <div className="panel">
            <h2>Balances</h2>
            <div className="stat">
              <span className="label">rUSD balance</span>
              <span className="value">
                {rusdBalance === null ? "--" : formatAmount(rusdBalance, decimals)}
              </span>
            </div>
            <div className="stat">
              <span className="label">USDC balance</span>
              <span className="value">
                {usdcBalance === null ? "--" : formatAmount(usdcBalance, decimals)}
              </span>
            </div>
            <div className="stat">
              <span className="label">Underlying USDC</span>
              <span className="value">
                {underlyingBalance === null ? "--" : formatAmount(underlyingBalance, decimals)}
              </span>
            </div>
            <div className="stat">
              <span className="label">Exchange rate</span>
              <span className="value">{exchangeRate}</span>
            </div>
          </div>

          <div className="panel">
            <h2>Approve + Mint</h2>
            <div className="action">
              <input
                value={mintAmount}
                onChange={(event) => setMintAmount(event.target.value)}
                placeholder="Amount in rUSD"
                inputMode="decimal"
              />
              <button
                className="primary"
                onClick={needsApproval ? onApprove : onMint}
                disabled={!account || busy}
              >
                {needsApproval ? "Approve USDC" : "Mint rUSD"}
              </button>
              <div className="stat">
                <span className="label">USDC allowance</span>
                <span className="value">
                  {usdcAllowance === null ? "--" : formatAmount(usdcAllowance, decimals)}
                </span>
              </div>
            </div>
          </div>

          <div className="panel">
            <h2>Burn rUSD</h2>
            <div className="action">
              <input
                value={burnAmount}
                onChange={(event) => setBurnAmount(event.target.value)}
                placeholder="Amount in rUSD"
                inputMode="decimal"
              />
              <button className="primary" onClick={onBurn} disabled={!account || busy}>
                Burn rUSD
              </button>
              <div className="stat">
                <span className="label">Total supply</span>
                <span className="value">
                  {totalSupply === null ? "--" : formatAmount(totalSupply, decimals)}
                </span>
              </div>
            </div>
          </div>
        </section>

        <section className="footer">
          <div className="split">
            <span className="tag">USDC: {shorten(config.usdcContractId)}</span>
            <span className="tag">rUSD: {shorten(config.rusdContractId)}</span>
            <span className="tag">RPC: {config.rpcUrl || "unset"}</span>
          </div>
        </section>
      </div>
    </main>
  );
}

function toBigInt(value: unknown): bigint | null {
  if (value === null || value === undefined) {
    return null;
  }
  if (typeof value === "bigint") {
    return value;
  }
  if (typeof value === "number") {
    return BigInt(value);
  }
  if (typeof value === "string") {
    return BigInt(value);
  }
  return null;
}

function toNumber(value: unknown): number | null {
  if (value === null || value === undefined) {
    return null;
  }
  if (typeof value === "number") {
    return value;
  }
  if (typeof value === "bigint") {
    return Number(value);
  }
  if (typeof value === "string") {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : null;
  }
  return null;
}
