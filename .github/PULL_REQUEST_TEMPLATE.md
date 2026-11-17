## What and why

## How I tested it

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- OS and version tested on:

## Checklist

- [ ] No new `unsafe`, `unwrap()` or `expect()` in library code
- [ ] User-visible errors use `AppError::user_message()`
- [ ] Docs and CHANGELOG updated if behaviour changed
- [ ] No unmeasured performance claims
