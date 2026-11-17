# Async and concurrency

The template uses threads, not an async runtime (ADR-0003).

`TaskManager::spawn(name, |ctx| -> Result<String>)` queues a job on a fixed pool (default 2
workers). Inside the job:

```rust
ctx.progress(0.4, "Copying database");   // 0.0..=1.0 plus a message
ctx.check_cancelled()?;                  // returns AppError::Cancelled after Cancel is pressed
```

Statuses: Queued, Running, Succeeded, Failed, Cancelled. Queued jobs cancel immediately; running
jobs stop at the next `check_cancelled`. A panicking job becomes Failed and the worker survives.
`shutdown(timeout)` stops accepting work and waits for the pool. Tested in
`crates/app-core/tests/lifecycle.rs`.

Rules: jobs never touch egui; share data by cloning `Database` (an `Arc` handle) or by sending
results through events. Do not hold the SQLite mutex across a long computation.
