import type { ReactNode } from "react";

import "./globals.css";

export const metadata = {
  title: "rUSD",
  description: "Rebasing SEP-41 token backed by USDC on Soroban."
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
