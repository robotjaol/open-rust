# Testing

```bash
cargo test --workspace          # everything
cargo test -p app-core --test lifecycle   # integration tests
```

| Layer | Where | Count and kind |
|---|---|---|
| Unit | `#[cfg(test)]` in each crate | errors, config migration, security helpers, storage, state reducer, palette contrast, task pool |
| Property | `proptest` in app-config, app-security, app-storage | file-name validation, config round trip, query builder |
| Integration | `crates/app-core/tests/lifecycle.rs` | boot, reboot without reseed, task failure and panic isolation, cancellation, backup/restore, corrupt backup, corrupt config, event bus |
| UI | Screenshot driver (`--screenshot-dir`) | Visual check by a human; not an automated pixel comparison |

Run counts and results for the current commit are in [../docs/qa-report.md](qa-report.md). There is
no coverage measurement in this repository; do not quote a coverage figure.
Not tested: Windows, macOS, Wayland, screen readers, GPU rendering.
