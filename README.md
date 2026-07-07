# Stellium Contracts

Soroban smart contracts for the Stellium payment gateway. These contracts handle escrow protection and direct payments on the Stellar network.

## Overview

This repo contains two Soroban contracts:

### Escrow Contract (`contracts/escrow/`)

Locks funds from a buyer until they confirm delivery, or automatically refunds after a timeout.

**Functions:**
- `create(buyer, seller, amount, asset, timeout)` — Lock funds in escrow. Returns escrow ID.
- `release(escrow_id)` — Buyer confirms delivery, funds go to seller.
- `refund(escrow_id)` — After timeout, buyer gets their funds back.
- `get_escrow(escrow_id)` — Read escrow details.

**Events emitted:**
- `escrow.created` — `(id, buyer, seller, amount)`
- `escrow.released` — `(id, seller, amount)`
- `escrow.refunded` — `(id, buyer, amount)`

### Payment Contract (`contracts/payment/`)

Direct one-shot payments with metadata support.

**Functions:**
- `pay(sender, recipient, amount, asset, metadata)` — Execute a payment. Returns payment ID.
- `get_payment(payment_id)` — Read payment details.
- `verify(payment_id)` — Check if a payment exists and completed.

**Events emitted:**
- `payment.created` — `(id, sender, recipient, amount)`

## Project Structure

```
contracts/
├── escrow/
│   ├── Cargo.toml
│   └── src/lib.rs          ← Escrow contract logic
├── payment/
│   ├── Cargo.toml
│   └── src/lib.rs          ← Payment contract logic
├── tests/
│   ├── Cargo.toml
│   └── src/lib.rs          ← Integration tests (needs expansion)
└── Cargo.toml              ← Workspace root
```

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup#install-soroban-cli)

```bash
cargo install --locked soroban-cli
```

## Build

```bash
# Build all contracts to .wasm
soroban contract build

# Or just check compilation
cargo check
```

## Test

```bash
cargo test
```

## What Needs Work

These contracts have a working foundation but need contributions in several areas. Check the issues for specific tasks.

**Contract features that are missing:**
- Escrow partial release (release funds in installments)
- Dispute resolution mechanism (third-party arbitrator)
- Contract upgradeability (WASM hash upgrades)
- Multi-sig approval for large payments
- Payment batching (multiple recipients in one tx)

**Testing gaps:**
- Full integration tests with Soroban sandbox (the current tests only verify compilation)
- Edge case tests (zero amount, duplicate release, expired timeout)
- Event emission verification in tests

**Code quality:**
- Add inline documentation for complex logic
- Add `#[cfg(test)]` unit tests within each contract module
- Input validation (minimum amounts, address format checks)

## Contract Addresses (Testnet)

| Contract | Address |
|----------|---------|
| Escrow   | *Not yet deployed* |
| Payment  | *Not yet deployed* |

## License

MIT
