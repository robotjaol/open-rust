# ADR-0008: Dual licence MIT OR Apache-2.0

Status: Accepted (2026-11-17)

## Context
Starter code should be adoptable in open and closed projects.

## Decision
Dual licence, same as most of the Rust ecosystem; copyright holder robotjaol. Third-party licences are checked by cargo deny.

## Consequences
Contributors license their work under both. No GPL dependencies in the default feature set (verified by deny.toml allow list in CI).
