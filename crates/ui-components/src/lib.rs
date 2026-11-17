//! Open Rust design system for egui.
//!
//! * [`tokens`]: palette, spacing, radius and type scale (with WCAG contrast tests),
//! * [`theme`]: applies the tokens to an `egui::Context` (light, dark, system),
//! * [`icons`]: vector icons painted with strokes (no font or image assets),
//! * [`widgets`]: buttons, badges, cards, toggles, navigation and settings rows,
//! * [`charts`]: sparkline, bar and donut charts,
//! * [`overlays`]: toast host, command palette and confirmation dialog.

pub mod charts;
pub mod icons;
pub mod overlays;
pub mod theme;
pub mod tokens;
pub mod widgets;

pub use icons::Icon;
pub use tokens::{Palette, Tone};
