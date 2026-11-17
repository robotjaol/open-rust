# Security research

Sources: the author's threat modelling; no external advisory data was retrieved this session.
Treat as **Recommendation**. Full model: [docs/security.md](../docs/security.md).

| Risk | Control in code | Test |
|---|---|---|
| Path traversal, symlink escape | `PathPolicy` canonicalises and checks the root | unit tests in `app-security` |
| Unsafe file names | `validate_file_name` | unit and property tests |
| Huge or hostile input | `read_limited`, JSON import size and row caps | unit tests |
| CSV formula injection | `csv_safe` prefixes risky cells | unit tests |
| SQL injection | Bound parameters; sort column from an allow-list | property test on query builder |
| Secret leakage in logs | `Redacted<T>` prints `[redacted]` | unit test |
| Crash data leakage | Crash report stored locally, never sent | by design |
| Supply chain | `cargo audit`, `cargo deny` in CI | CI (not yet run remotely) |

**Not implemented:** OS keyring integration, update signature verification, sandboxing.
