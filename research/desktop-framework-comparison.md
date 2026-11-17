# Desktop framework comparison

Scope: five options for a Rust desktop starter. Evidence is limited to the sources in the
[register](source-register.md). Items marked *Unverified* come from the author's experience.

| | egui/eframe | Tauri 2 | iced | Slint | Dioxus desktop |
|---|---|---|---|---|---|
| Rendering | Own painter on glow/wgpu (S3) | OS web view (S6, S7) | Own renderer | Own renderer | Web view (Unverified) |
| UI language | Rust | HTML/CSS/JS plus Rust | Rust (Elm style) | `.slint` DSL plus Rust | Rust (RSX) |
| Licence | MIT OR Apache-2.0 (L1) | MIT OR Apache-2.0 (Unverified) | MIT (Unverified) | GPL-3.0, Royalty-free (needs disclosure), or commercial (S4, S5) | MIT OR Apache-2.0 (Unverified) |
| Maturity signal | Widely used (Unverified) | 2.x stable (S6) | Self-described experimental (S9) | 1.x (S4) | Unverified |
| Headless testing | Screenshot API (local) | Needs web tooling | Added in 0.14 (S10) | Testing backend (Unverified) | Unverified |

## Decision

egui/eframe wins on licence simplicity, a pure Rust toolchain, and reproducible screenshots.

## Rejected, with the cost of the rejection

- **Tauri 2:** best path to a native-feeling, web-designed UI and small installers (S6). Cost accepted: the app would carry a JS toolchain and per-OS web view differences (S7); web view updates can break the app (S8).
- **iced:** attractive architecture (S10). Self-described experimental status (S9) is a risk for a starter.
- **Slint:** excellent tooling. Licence needs a decision from every adopter: GPL, disclosure, or commercial (S4, S5). That conflicts with "reusable by anyone".
- **Dioxus, GPUI:** not evaluated beyond reputation. **Unresolved:** a fair comparison needs a prototype of each.

## When to choose something else

Pick Tauri if your team is web-first or needs native look and mobile later. Pick Slint if you
accept its licence and need embedded targets.
