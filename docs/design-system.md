# Design system

Defined in `crates/ui-components/src/tokens.rs`.

- **Palette**: `Palette::new(dark, accent)` with surfaces (`bg_base`, `bg_surface`, `bg_elevated`, hover, active), borders, three text levels, accent set, and status colours.
- **Accents**: Indigo, Teal, Amber, Rose, Emerald (`Accent::ALL`). `on_accent` is chosen automatically between white and dark ink so text on accent stays above 4.5:1.
- **Themes**: System, Light, Dark. `theme::apply` configures egui visuals, fonts and zoom (`ui_scale`).
- **Spacing** 4, 8, 12, 16, 24, 32. **Radius** 4, 6, 10. **Type** 11.5, 13.5, 15, 20, 30.
- **Icons**: 37 vector icons drawn with the painter (no image assets, crisp at any scale).
- **Charts**: `sparkline`, `bar_chart`, `donut`, using `Palette::series(i)`.
- **Overlays**: toasts, command palette (`filter_items` is a case-insensitive subsequence match), confirm dialog.

Contrast is enforced by unit tests for both themes and every accent (WCAG AA 4.5:1 for text).
See [screenshots](screenshots/).
