# Competitor analysis

"Competitors" are other starting points a developer might pick. Only what the register backs
is stated as fact.

| Starting point | What it gives | Gap this starterpack fills | Evidence |
|---|---|---|---|
| `eframe_template` (egui) | Minimal app, web and native build | No storage, tasks, config migration, security, packaging | S3 |
| `create-tauri-app` | Web frontend scaffold | Requires a JS stack; no Rust UI design system | S6 |
| Plain `cargo new` plus a GUI crate | Total freedom | Every team rebuilds the same plumbing | Recommendation |

**Not researched:** commercial starters and closed-source kits; GitHub star counts and issue
activity were not collected, so nothing here claims relative popularity.

**Assumption:** developers who want a pure Rust desktop app value a runnable, tested
architecture more than extra widgets.
