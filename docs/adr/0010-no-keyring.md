# ADR-0010: No OS keyring dependency by default

Status: Accepted (2026-11-17)

## Context
Keyrings differ per OS and add native dependencies; the template stores no credentials.

## Decision
app-security exposes a SecretStore trait with an in-memory implementation and Redacted<T>. Projects that need secrets implement the trait over a keyring crate.

## Consequences
Positive: smaller footprint. Negative: the template does not demonstrate persistent secret storage; documented as a gap.
