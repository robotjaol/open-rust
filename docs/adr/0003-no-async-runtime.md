# ADR-0003: Plain threads instead of an async runtime

Status: Accepted (2026-11-17)

## Context
The template has no network I/O. An async runtime adds compile time and a second concurrency model.

## Decision
TaskManager is a fixed worker pool over std threads with progress, cancellation tokens, panic isolation (catch_unwind) and shutdown with timeout. UI receives events through a channel and a repaint waker.

## Consequences
Positive: small, easy to read, deterministic tests. Negative: projects that add networking will likely add tokio; the TaskManager API is small enough to wrap.
