# Source register

Every external claim in `research/` and `docs/` points to an entry here. Entries were
retrieved with a web search on **2026-11-17**. Dates are the publication or update date when
the page shows one; otherwise "not stated". A result was **not** opened in full unless noted;
claims rely on the snippet returned by the search and are labelled accordingly.

| ID | Source | URL | Date | Used for |
|----|--------|-----|------|----------|
| S1 | egui accessibility guide (egui repository) | https://raw.githubusercontent.com/emilk/egui/main/docs/accessibility.md | not stated | AccessKit enabled by default in eframe on native platforms |
| S2 | egui issue 4527, "high CPU with accesskit feature" | https://github.com/emilk/egui/issues/4527 | 2024 (egui 0.27 era) | Reported CPU cost of AccessKit in one app |
| S3 | eframe crate description (docs.rs) | https://docs.rs/crate/eframe/^0.27 | not stated (0.27 page) | eframe renders with egui_glow and winit natively |
| S4 | Slint licence file, v1.17.0 | https://components.espressif.com/components/slint/slint/versions/1.17.0/license?language=en | 2026 | Slint is GPL-3.0-only OR Royalty-free-2.0 OR Software-3.0; embedded needs commercial |
| S5 | Slint LICENSE.md, npm package 1.17.1 | https://unpkg.com/slint-ui@1.17.1/LICENSE.md | 2026 | Royalty-free licence requires disclosure of Slint use |
| S6 | Tauri 2 documentation index | https://v2.tauri.app/llms.txt | not stated | Tauri uses the OS web view; any web frontend |
| S7 | "Exploring system webviews in Tauri" | https://dev.to/shrsv/exploring-system-webviews-in-tauri-native-rendering-for-efficient-cross-platform-apps-9hl | not stated | WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux |
| S8 | Scott Logic, WebView2 and Electron challengers | https://blog.scottlogic.com/2023/02/01/webview2-electron-challengers-and-slightly-lighter-desktop-web-applications.html | 2023-02-01 | Trade-off: shared web view gets OS updates, but updates can break the app |
| S9 | iced 0.14.0 on docs.rs | https://docs.rs/crate/iced/latest | 0.14.0 | iced describes itself as experimental software |
| S10 | Phoronix, Iced 0.14 release | https://www.phoronix.com/news/Iced-0.14-Rust-GUI-LIbrary | not stated | iced 0.14 adds headless testing; used by COSMIC |
| S11 | cargo-packager repository | https://github.com/crabnebula-dev/cargo-packager | not stated | Formats: deb, AppImage, pacman, NSIS, MSI, dmg, app |
| S12 | cargo-packager documentation | https://docs.crabnebula.dev/packager/ | not stated | Config in Packager.toml; `cargo packager --release` |
| L1 | `cargo metadata` of this repository | local, 2026-11-17 | 2026-11-17 | Dependency count and licence mix (reproducible) |
| L2 | Local measurements in `docs/benchmarking.md` | local | 2026-11-17 | Binary size, startup, benchmarks |

Facts without an S or L reference in the other files are the author's engineering knowledge
and are labelled **Unverified**.
