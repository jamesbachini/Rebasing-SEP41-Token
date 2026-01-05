export function formatAmount(value: bigint, decimals: number) {
  const negative = value < 0n;
  const abs = negative ? -value : value;
  const base = 10n ** BigInt(decimals);
  const whole = abs / base;
  const fraction = abs % base;
  const fractionStr = fraction.toString().padStart(decimals, "0").replace(/0+$/, "");
  const formatted = fractionStr ? `${whole.toString()}.${fractionStr}` : whole.toString();
  return negative ? `-${formatted}` : formatted;
}

export function parseAmount(input: string, decimals: number) {
  const trimmed = input.trim();
  if (!trimmed) {
    return 0n;
  }
  const negative = trimmed.startsWith("-");
  const sanitized = negative ? trimmed.slice(1) : trimmed;
  const [wholePart, fracPart = ""] = sanitized.split(".");
  const whole = wholePart ? BigInt(wholePart) : 0n;
  const padded = fracPart.padEnd(decimals, "0").slice(0, decimals);
  const fraction = padded ? BigInt(padded) : 0n;
  const base = 10n ** BigInt(decimals);
  const value = whole * base + fraction;
  return negative ? -value : value;
}

export function shorten(address: string, size = 4) {
  if (!address) {
    return "";
  }
  return `${address.slice(0, size)}...${address.slice(-size)}`;
}
