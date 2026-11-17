# Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Window does not open on Linux | Missing windowing or GL library at run time | Install `libxkbcommon`, X11/xcb or Wayland client libraries and Mesa |
| Black or garbled window under Xvfb | No GPU | Run with `LIBGL_ALWAYS_SOFTWARE=1` |
| Build fails in `wayland-sys` | `rfd` default features enabled | Keep `rfd` with `default-features = false, features = ["xdg-portal"]` |
| "Dialogs unavailable" toast | No portal backend | Install `xdg-desktop-portal` and a backend for your desktop |
| Settings reset after start | Settings file unreadable | Look for `config.toml.corrupt` next to it (see About for paths) |
| Recovery page on start | Previous crash | Read the report, then Dismiss report |
| Deprecation errors after upgrading egui | API renames | See `docs/ui-development.md` (0.37 notes) |
| First build very slow | ~430 crates | Use `Swatinem/rust-cache` in CI; do not clean `target/` needlessly |
| Unexpected data location | `OPENRUST_HOME` set | Unset it or point it where you want |
