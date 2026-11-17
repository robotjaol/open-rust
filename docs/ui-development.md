# UI development

- Colours and sizes come from `Palette` and `tokens::{space, radius, text}`. No hard-coded colours in pages.
- Use widgets from `ui_components::widgets` (`button`, `card`, `page_header`, `setting_row`, `segmented`, `badge`, `empty_state`, `progress_bar`, ...).
- Every list has an empty state; every destructive action uses `confirm_dialog` with a verb label.
- Custom widgets call `widget_info` so accessibility trees get a role and name.
- `card` lays its content out top-down and left-aligned, even inside `ui.columns`.
- Do not call `set_max_width` with a value larger than `available_width()`; it can overflow the window
  (this bug was found and fixed during development).
- egui 0.37 notes: panels are `egui::Panel` and `show` takes the parent `Ui`; `Id::new` is deprecated in favour of `Id::unique`; use `Frame::NONE`.

Screenshots: `open-rust --screenshot-dir <dir>` renders every page in both themes at 1280x800.
