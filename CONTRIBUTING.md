# Contributing

Thanks for helping. This guide keeps contributions quick to review.

## Set up

```bash
git clone https://github.com/robotjaol/open-rust && cd open-rust
cargo run -p open-rust            # debug build with a demo database
```

Linux needs the system libraries listed in [docs/getting-started.md](docs/getting-started.md).

## Before you open a pull request

Run exactly what CI runs:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Rules of the codebase:

- `unsafe` is forbidden (`unsafe_code = "forbid"`). Open an issue first if you believe you need it.
- No `unwrap()` or `expect()` in library code. Return `AppError` and let the UI show `user_message()`.
- UI code never blocks. Long work goes through `TaskManager` and reports progress.
- New file access goes through `PathPolicy` and the helpers in `app-security`.
- New storage changes are a new migration; never edit an applied migration.
- Add a test with every bug fix. Do not add benchmark numbers you did not measure.

## Commit and PR style

Short imperative subject (under 72 characters), a body that says why. One topic per PR.
Fill in the pull request template, including how you tested it and on which OS.

## Adding a feature

See [docs/extending-the-application.md](docs/extending-the-application.md).

## Licence

By contributing you agree that your work is licensed under MIT OR Apache-2.0, like the project.
