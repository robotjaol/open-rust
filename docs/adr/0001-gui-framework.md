# ADR-0001: Use egui/eframe for the GUI

Status: Accepted (2026-11-17)

## Context
Candidates: egui/eframe, Tauri 2, iced, Slint, Dioxus (see research/desktop-framework-comparison.md). Needs: permissive licence, pure Rust, scriptable screenshots, no JavaScript toolchain.

## Decision
Use egui/eframe 0.37 with the glow renderer. Keep every UI-independent rule in crates that do not depend on egui, so the GUI can be replaced.

## Consequences
Positive: one toolchain, MIT OR Apache-2.0, headless screenshots work. Negative: widgets are not native; accessibility rests on AccessKit, which the author did not test with a screen reader; immediate-mode code needs discipline to keep work off the UI thread; egui APIs change between releases (observed during this build).
