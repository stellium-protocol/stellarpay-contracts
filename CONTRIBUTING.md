# Contributing to Stellium Contracts

Thanks for your interest in contributing! This guide will help you get set up.

## Getting Started

### Prerequisites

1. Install Rust: https://rustup.rs/
2. Install Soroban CLI:
   ```bash
   cargo install --locked soroban-cli
   ```
3. Verify installation:
   ```bash
   rustc --version
   soroban --version
   ```

### Setup

```bash
git clone https://github.com/stellium/stellium-contracts.git
cd stellium-contracts
cargo check     # Verify everything compiles
cargo test      # Run tests
```

### Project Structure

```
contracts/
├── escrow/src/lib.rs    ← Escrow contract (lock, release, refund)
├── payment/src/lib.rs   ← Payment contract (direct pay, verify)
└── tests/src/lib.rs     ← Integration tests
```

## How to Contribute

1. Find an issue you want to work on (check `good first issue` label)
2. Comment on the issue to let others know you're working on it
3. Fork the repo and create a branch: `git checkout -b fix/issue-number-description`
4. Make your changes
5. Run `cargo test` and `cargo check` — both must pass
6. Submit a PR linking the issue

## Contract Development Tips

- **Auth**: Always call `require_auth()` on addresses that should authorize the action
- **Events**: Emit events for all state changes — the SDK and dashboard depend on them
- **Storage**: Use `env.storage().instance()` for contract state
- **Testing**: Use `Env::default()` and `env.register(Contract, ())` in tests
- **Errors**: Use `assert!()` with clear messages — they become error codes on-chain

## Soroban Resources

- [Soroban Docs](https://soroban.stellar.org/docs)
- [Soroban Examples](https://github.com/stellar/soroban-examples)
- [Stellar SDK Reference](https://docs.rs/soroban-sdk/latest/soroban_sdk/)
