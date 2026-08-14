# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Input validation for escrow and payment contracts (zero amounts, self-transfers)

### Changed
- Improved error messages in `assert!()` calls to include context (escrow/payment IDs, remaining time)

## [0.1.0] - 2026-07-07

### Added
- Escrow contract with `create`, `release`, `refund`, and `get_escrow` functions
- Payment contract with `pay`, `get_payment`, and `verify` functions
- Escrow timeout mechanism for automatic refund eligibility
- Event emission for all state changes (`escrow.created`, `escrow.released`, `escrow.refunded`, `payment.created`)
- Auto-incrementing ID counters for both escrow and payment records
- SEP-41 token transfer integration for fund custody and direct payments
- 10 sandbox-based integration tests covering escrow and payment workflows
- Inline documentation explaining storage patterns, auth, timeout mechanism, and counter pattern
