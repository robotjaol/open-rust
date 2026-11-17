# Executive summary

**Fact levels used in this folder:** `Verified` (source in the register or reproduced locally),
`Recommendation` (engineering judgement), `Assumption`, `Unresolved`.

## Result

The starterpack uses **egui/eframe 0.37** on a UI-agnostic Rust core. That choice is a
recommendation, not the only defensible one; the reasoning is in
[desktop-framework-comparison.md](desktop-framework-comparison.md) and ADR-0001.

## Why this framework

| Criterion | egui/eframe | Why it matters here |
|-----------|-------------|---------------------|
| Licence | MIT OR Apache-2.0 (L1) | Starter must be reusable in closed and open projects |
| Pure Rust, no web view | Verified (S3) | One toolchain, no JS build |
| Automated screenshots | Verified locally | `request_screenshot` under Xvfb produced all images in `docs/screenshots` |
| Accessibility | AccessKit on by default natively (S1) | Real, but less mature than native toolkits (Recommendation) |

## Main trade-offs (honest)

1. egui draws its own widgets, so it does not look or behave exactly like native controls.
2. AccessKit costs CPU in at least one reported case (S2); measure before enabling in a heavy app.
3. Immediate-mode code re-runs every frame; heavy work must stay off the UI thread (enforced by `TaskManager`).

## Unresolved

- Behaviour on Windows and macOS is **not validated** by the author. CI builds and tests there; no human has run the app.
- Screen-reader behaviour was not tested.
