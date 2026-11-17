# Rust ecosystem analysis

Method: choose crates for each concern, read their licence and version from the lockfile
(L1), and keep the set small. No popularity ranking was used.

| Concern | Choice | Version (L1) | Reason | Alternative considered |
|---|---|---|---|---|
| GUI | eframe, egui, egui_extras | 0.37.0 | See ADR-0001 | iced, Slint, Tauri |
| Errors | thiserror | 2.0.21 | Typed errors, no runtime cost | anyhow in libraries (rejected: loses type) |
| Config | serde + toml | toml 1.2.1 | Human-editable file | ron, json |
| Logging | tracing + tracing-subscriber | tracing 0.1.44 | Structured, layered | log + env_logger |
| Storage | rusqlite (bundled) | 0.43.0 | Embedded, no server | sqlx (needs async), sled |
| Paths | directories | 6.0.0 | Per-OS app directories | dirs |
| Dialogs | rfd (xdg-portal) | 0.18.0 | Native dialogs | native-dialog |
| Tests | proptest, tempfile, criterion | 1.12.0, 3.27.0, 0.8.2 | Property tests, benches | quickcheck, divan |

Dependency footprint: **434 packages** in `Cargo.lock` including dev and platform-specific
crates (L1). Licence mix is permissive; the only copyleft-flavoured entries are dual-licensed
(`MIT OR Apache-2.0 OR LGPL-2.1-or-later`) and one MPL-2.0 crate.

No async runtime is used (ADR-0003). **Unresolved:** if the app needs HTTP or many concurrent
I/O tasks, adding tokio becomes reasonable; that is a feature decision, not a template one.
