# ADR-0004: SQLite through rusqlite with bundled library

Status: Accepted (2026-11-17)

## Context
Desktop apps need local, transactional storage that users can back up as one file.

## Decision
rusqlite with features bundled and backup. Versioned migrations in code; each migration runs in a transaction. Queries use bound parameters; sort columns come from an enum allow-list.

## Consequences
Positive: no system SQLite needed, reproducible. Negative: compiles C code (needs a C compiler); no async access; schema is hand-written SQL, not an ORM.
