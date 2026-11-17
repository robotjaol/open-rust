# ADR-0007: Keep panic = unwind in release

Status: Accepted (2026-11-17)

## Context
Aborting on panic would let one failed background job close the app.

## Decision
Release profile keeps unwinding. TaskManager wraps jobs in catch_unwind. A panic hook writes a crash report; the next start shows the Recovery page.

## Consequences
Positive: job failures are isolated. Negative: slightly larger binary than panic = abort (not measured).
