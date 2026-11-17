# Distribution research

| Target | Tool | Source | Status here |
|---|---|---|---|
| Linux `.tar.gz`, `.deb` | `scripts/package-linux.sh` (dpkg-deb) | local | Built and inspected on Linux |
| Windows `.exe` (NSIS) | cargo-packager | S11, S12 | Configured only, **not built** |
| macOS `.dmg` | cargo-packager | S11, S12 | Configured only, **not built** |
| AppImage, pacman | cargo-packager supports them (S11) | S11 | Not configured |

cargo-packager reads `Packager.toml` and runs with `cargo packager --release` (S12).

## Signing and notarization (documented, not performed)

Windows needs an Authenticode certificate; macOS needs an Apple Developer ID and notarization.
Both require the maintainer's credentials and are therefore **not implemented**. Unsigned
installers trigger OS warnings. Never commit certificates; use CI secrets.
