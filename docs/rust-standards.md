# Rust standards

- Edition 2024, `rust-version = 1.95`, resolver 3.
- `unsafe_code = "forbid"` for all workspace crates.
- Clippy: `all` warn, `unwrap_used`, `expect_used`, `dbg_macro`, `todo` warn; CI uses `-D warnings`.
  Tests may use `unwrap`/`expect`.
- `missing_debug_implementations = "warn"`: derive or implement `Debug` for public types.
- Errors: library code returns `app_errors::Result<T>`; never `String` errors across crate boundaries.
- No `println!` in libraries; use `tracing` macros with structured fields (`error = %e`).
- Public items have doc comments that say what, not how.
- Formatting: `rustfmt.toml` in the repository; run `cargo fmt --all`.
- Naming: modules `snake_case`, types `UpperCamelCase`, UI page types named after the page.
- Keep functions short enough to read on one screen; extract when a closure nests three deep.
