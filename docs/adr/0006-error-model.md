# ADR-0006: Typed errors with sanitised user messages

Status: Accepted (2026-11-17)

## Context
Internal errors contain paths, SQL and OS text that users should not see.

## Decision
app-errors defines AppError with stable codes (E-*). user_message() is safe to show; Display and Debug are for logs. Io errors show only the file name to users.

## Consequences
Positive: consistent UI, safe by default. Negative: every new failure needs a variant or mapping.
