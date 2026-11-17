# ADR-0009: cargo-packager plus a Linux shell script

Status: Accepted (2026-11-17)

## Context
Need installers for three OSes, but only Linux can be validated by the author.

## Decision
scripts/package-linux.sh builds tar.gz and deb with dpkg-deb and is tested locally. Packager.toml configures NSIS and dmg for CI. Signing and notarization are documented, not automated.

## Consequences
Windows and macOS installers are unvalidated until a maintainer builds and runs them.
