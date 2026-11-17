# Technology decision matrix

Scores: 3 = strong fit, 2 = acceptable, 1 = weak, 0 = blocks the requirement. Scores are the
author's judgement from [research/](../research/); they are not measurements. Weights reflect
the goals of a reusable starter.

| Criterion (weight) | egui/eframe | Tauri 2 | iced | Slint | Dioxus |
|---|---|---|---|---|---|
| Permissive licence for everyone (3) | 3 | 3 | 3 | 1 | 3 |
| Pure Rust, no JS toolchain (3) | 3 | 0 | 3 | 2 | 2 |
| Headless screenshot or UI test path (2) | 3 | 1 | 2 | 2 | 1 |
| Native look and feel (2) | 1 | 3 | 1 | 2 | 3 |
| Accessibility path (2) | 2 | 3 | 1 | 2 | 2 |
| Maturity signals (2) | 3 | 3 | 1 | 2 | 1 |
| **Weighted total (max 42)** | **36** | **29** | **28** | **25** | **29** |

Other layers: see ADR-0003 (concurrency), ADR-0004 (storage), ADR-0005 (config),
ADR-0006 (errors), ADR-0009 (packaging). Sensitivity: if "native look" weighs 5 and
"pure Rust" weighs 1, Tauri overtakes egui (the totals are close enough that the result is weight-dependent). Pick by your own weights.
