# Release process

1. Update `CHANGELOG.md` and the version in the root `Cargo.toml` (`[workspace.package]`) and `Packager.toml`.
2. Run the checks from [testing.md](testing.md) and `./scripts/package-linux.sh`.
3. Commit, then tag: `git tag v0.1.0 && git push origin main --tags`.
4. The `Release` workflow builds Linux packages, attaches checksums and creates a GitHub release with generated notes; Windows and macOS jobs attach unsigned installers.
5. Smoke-test the artifacts on a clean machine before announcing.

The release workflow has not been executed yet (no tag has been pushed); treat it as unvalidated.
