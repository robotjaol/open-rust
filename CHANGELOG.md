# Changelog

All notable changes are recorded here. Format follows Keep a Changelog; versions follow SemVer.

## [Unreleased]

## [0.1.0] - 2026-11-17

### Added
- Cargo workspace with nine crates: `app-errors`, `app-config`, `app-security`, `app-logging`,
  `app-storage`, `app-platform`, `app-core`, `ui-components`, and the `open-rust` desktop binary.
- egui/eframe 0.37 desktop shell with sidebar, command palette (Ctrl+K), notifications,
  toasts, back/forward navigation and window-state persistence.
- Eleven example pages: Welcome, Dashboard, Workspace, Files, Data, Tasks, Logs, Settings,
  Appearance, About, Recovery.
- Design system with light/dark/system themes, five accents, WCAG contrast tests.
- Layered configuration with schema migration and atomic save.
- SQLite storage with versioned migrations, backup, restore, JSON/CSV transfer.
- Background task pool with progress, cancellation and panic isolation.
- Crash reports via panic hook and a Recovery page.
- Unit, integration and property tests; Criterion benchmarks.
- Linux packaging script (tar.gz, .deb); cargo-packager configuration for Windows and macOS.
- GitHub Actions CI and release workflows.
