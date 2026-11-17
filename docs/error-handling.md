# Error handling

`AppError` variants: `Config`, `Storage`, `Security`, `Io { path, source }`, `Validation`,
`Cancelled`, `Platform`, `Internal`. Each has:

- `code()`: stable id such as `E-STORAGE`, safe to quote in bug reports.
- `user_message()`: sanitised sentence for the UI. Io errors reveal the file name, not the full path.
- `is_recoverable()`: whether the user can retry.
- `report()`: `ErrorReport` for the log and the Recovery page.

## Pattern

```rust
match cx.app.db.delete_item(id) {
    Ok(_) => cx.notify(Notification::info("Record deleted", name)),
    Err(e) => cx.fail("Could not delete record", &e),   // logs detail, shows safe text
}
```

Panics: a panic hook writes a crash report (message, location, thread, backtrace) to the crash
directory; the next start opens the Recovery page. Panics inside tasks are caught and
reported as a failed task. Dismissing a crash report archives the file.
