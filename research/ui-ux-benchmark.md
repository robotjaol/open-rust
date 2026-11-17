# UI/UX benchmark

No competitor applications were run or measured. The benchmark below is a **checklist** the
design system follows, with the verification method for each row.

| Practice | Implemented | How verified |
|---|---|---|
| Text contrast at least 4.5:1 (WCAG 2.x AA) | Yes, all text tokens, both themes, five accents | Unit tests in `ui-components` compute the ratio |
| Light, dark and system theme | Yes | Screenshots in `docs/screenshots` |
| Keyboard navigation and shortcuts | Command palette Ctrl+K, Ctrl+B, Alt+Left/Right, Ctrl+, | Manual; shortcut handling in `app.rs` |
| Visible focus | Focus ring on custom buttons, toggles | Manual |
| Destructive actions confirmed, named button | Yes (`Delete note`, `Reset settings`) | Manual |
| Empty and error states | Yes (every list page) | Screenshots |
| Reduced motion | **Not implemented** | Listed in Known gaps |
| Screen reader labels | Partial: `widget_info` on custom widgets | **Not tested** with a screen reader |
