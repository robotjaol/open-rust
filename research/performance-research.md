# Performance research

Policy: no performance number appears in this repository unless it was measured and the
method is recorded in [docs/benchmarking.md](../docs/benchmarking.md).

## Design choices that affect performance (Recommendations)

- Heavy work is off the UI thread (`TaskManager`).
- SQLite queries are paged (`LIMIT/OFFSET`) so the table never loads all rows.
- The window repaints on events; continuous repaint only while a task runs.
- Release profile: thin LTO, `strip = "symbols"`, `codegen-units = 1` (check `Cargo.toml`).

## Known risks

- AccessKit overhead was reported once (S2). Not measured here.
- Software rendering (llvmpipe under Xvfb) was used for screenshots, so frame times shown on
  the Dashboard of those images are **not** representative of real GPUs.
- Memory (RSS) is read from `/proc` and exists only on Linux.

## Unresolved

Startup time and memory on Windows and macOS; behaviour with 1M rows; GPU renderer selection.
