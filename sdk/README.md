# @fluxapay/sdk

Official TypeScript SDK for interacting with FluxaPay's Soroban smart contracts on the Stellar network.

## Installation

```bash
npm install @fluxapay/sdk
```

## Quick Start

```typescript
import { FluxapayClient } from "@fluxapay/sdk";

const client = new FluxapayClient({
  network: "testnet",
  rpcUrl: "https://soroban-testnet.stellar.org",
  contractId: "C...", // Your contract ID
});

async function main() {
  // Create a payment
  const payment = await client.createPayment({
    paymentId: "pay_123",
    merchantId: "G...",
    amount: 1000000n, // 1 USDC
    currency: "USDC",
    depositAddress: "G...",
    expiresAt: BigInt(Math.floor(Date.now() / 1000) + 3600),
  });

  console.log("Payment created:", payment);

  // Get payment status
  const status = await client.getPayment("pay_123");
  console.log("Payment status:", status);
}
```

## Features

- **High-level Wrapper**: `FluxapayClient` simplifies complex contract interactions.
- **Typed Interfaces**: Full TypeScript support for all contract models (`Merchant`, `Payment`, `Refund`, etc.).
- **Automatic Simulation**: Built-in support for Soroban transaction simulation.
- **Network Presets**: Easy switching between `testnet` and `mainnet`.

## License

MIT
