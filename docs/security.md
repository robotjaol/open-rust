# Security and threat model

Assets: user data in SQLite and settings, local files the user opens, crash reports and logs.
Adversaries considered: a malicious file chosen by the user (import, preview, restore), a corrupted settings file, a bug that leaks internal detail, a vulnerable dependency.
Out of scope: a hostile local administrator, physical access, network attacks (the app has no network code).

| Threat | Mitigation | Evidence |
|---|---|---|
| Path traversal or symlink escape | `PathPolicy` | unit tests |
| Oversized input exhausts memory | `read_limited`, import row/size caps | unit tests |
| CSV formula injection on export | `csv_safe` | unit test |
| SQL injection | bound parameters, enum sort column | unit and property tests |
| Corrupt config destroys settings | `.corrupt` copy, defaults | integration test |
| Bad backup replaces good data | validate before swap | integration test |
| Secrets in logs | `Redacted<T>` | unit test |
| Internal paths shown to users | `user_message()` sanitising | unit tests |
| Vulnerable dependencies | `cargo audit`, `cargo deny` | CI configured, **not yet run remotely** |

Known gaps: no OS keyring (ADR-0010); no update mechanism or signature check; no sandboxing;
unsigned installers; crash reports contain a backtrace and file paths (local only, never sent).
Report vulnerabilities as described in [SECURITY.md](../SECURITY.md).
