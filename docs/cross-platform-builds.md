# Cross-platform builds

CI (`.github/workflows/ci.yml`) runs clippy and tests on Ubuntu, Windows and macOS. The author
has run only Linux. Treat Windows and macOS as **unvalidated** until a human runs the app there.

| Target | Command | Notes |
|---|---|---|
| Linux x86_64 | `cargo build --release -p open-rust` | Needs a C compiler |
| Windows MSVC | same, on a Windows runner | Install Visual Studio Build Tools |
| macOS | same, on a macOS runner | Xcode command line tools |

Cross-compiling from Linux to Windows or macOS is not supported here (bundled SQLite and
windowing crates make it fragile); use native runners.

Platform differences in code: `process_rss_bytes` is Linux only (others show n/a); native dialogs use the XDG portal on Linux.
