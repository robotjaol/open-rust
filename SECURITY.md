# Security policy

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use GitHub's private
"Report a vulnerability" form on the Security tab of
https://github.com/robotjaol/open-rust. Include the version, your OS, and steps to reproduce.

Expect an acknowledgement within 7 days. This is a one-maintainer project, so that is a
target, not a guarantee.

## Supported versions

Only the latest release on the `main` branch receives fixes while the project is below 1.0.

## Scope and design

The threat model, trust boundaries and known gaps are in [docs/security.md](docs/security.md).
In short: no network access, no telemetry, `unsafe` forbidden in project code, all file input
size-limited and path-checked, SQL built from allow-lists and bound parameters. Dependencies
are checked in CI with `cargo audit` and `cargo deny`.
