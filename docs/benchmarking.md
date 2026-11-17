# Benchmarking

Criterion benchmarks: `crates/app-storage/benches/storage.rs` (paged query, search, stats, CSV export on 10,000 rows) and `crates/app-core/benches/tasks.rs` (100 trivial tasks on 2 workers).

```bash
cargo bench -p app-storage -p app-core -- --sample-size 20 --warm-up-time 1 --measurement-time 3
```

## Rules

1. Record hardware, OS, toolchain, profile, sample count, date next to every number.
2. Numbers from a shared VM are indicative only.
3. Never copy numbers between machines.

Measured results for this repository are in the QA report ([qa-report.md](qa-report.md)); this
page holds the method only.
