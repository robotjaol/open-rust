# Dependency evaluation

Reproduce with `cargo metadata --format-version 1` (L1, 2026-11-17).

- Packages in lockfile: **434** (all targets, including dev dependencies).
- Licence families: MIT / Apache-2.0 dual (about 70 percent of crates), plain MIT, Zlib,
  BSD, ISC, Unlicense, BSL-1.0, Unicode-3.0, one MPL-2.0 crate, and the egui font bundle
  (OFL-1.1 and Ubuntu font licence).
- `deny.toml` allows exactly these licences; any new family fails CI.
- Duplicate versions exist (for example `thiserror` 1.x and 2.x via transitive crates); `cargo deny` warns.

## Rules for adding a dependency

1. Licence on the `deny.toml` allow list.
2. Does the standard library or an existing crate already do it?
3. Is it maintained (recent release, open issue triage)? Check manually; not automated.
4. Does it need `unsafe` build steps or system libraries? Document them in `docs/getting-started.md`.
