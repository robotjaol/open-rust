# Workspace structure

| Path | Responsibility | Depends on |
|---|---|---|
| `crates/app-errors` | `AppError`, codes, `ErrorReport`, time formatting | std, thiserror |
| `crates/app-config` | `Config`, `Environment`, migrations, atomic save | app-errors, serde, toml |
| `crates/app-security` | `PathPolicy`, `read_limited`, `write_atomic`, `csv_safe`, `Redacted`, `SecretStore` | app-errors |
| `crates/app-logging` | `init`, `LogBuffer`, rotating file | tracing-subscriber |
| `crates/app-storage` | `Database`, migrations, items and notes, transfer | rusqlite, app-security |
| `crates/app-platform` | `AppPaths`, `FileService`, `dialogs`, `PlatformInfo` | directories, rfd, app-security |
| `crates/app-core` | `AppContext`, `TaskManager`, `AppState`, `EventBus`, notifications | all services |
| `crates/ui-components` | `Palette`, icons, widgets, charts, overlays | egui |
| `apps/desktop` | binary `open-rust`: shell, `FeatureModule`, pages, screenshot driver | everything |

Other top-level folders: `docs/`, `research/`, `assets/icons`, `packaging/`, `scripts/`,
`.github/`. Shared versions and lints live in the root `Cargo.toml` (`[workspace.dependencies]`,
`[workspace.lints]`).
