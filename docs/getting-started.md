# Getting started

## Requirements

- Rust stable (the workspace sets `rust-version = 1.95`; built and tested here with 1.97).
- A C compiler (SQLite is compiled from source) and `pkg-config` on Linux.
- Linux runtime: an X11 or Wayland session with OpenGL. The binary links only libc, libm and
  libgcc at build time (checked with `ldd`); windowing and GL libraries are loaded at run time.
  If the window does not open, install `libxkbcommon`, `libx11`/`libxcb` or `libwayland-client`, and Mesa.
- Windows and macOS: no extra libraries expected. **Not validated** by the author.

## Run

```bash
git clone https://github.com/robotjaol/open-rust
cd open-rust
cargo run -p open-rust                     # debug build, demo data
cargo run -p open-rust --release           # optimised
cargo run -p open-rust -- --home ./.demo   # keep all data in ./.demo (portable mode)
cargo run -p open-rust -- --page tasks     # start on a page
```

First build compiles about 430 crates; expect several minutes on a small machine.

## Check your changes

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Headless (CI, servers)

```bash
xvfb-run -a -s "-screen 0 1280x800x24" env LIBGL_ALWAYS_SOFTWARE=1 \
  cargo run -p open-rust -- --home /tmp/or --screenshot-dir docs/screenshots
```

Next: [architecture.md](architecture.md), [workspace-structure.md](workspace-structure.md).
