# Architecture

```
apps/desktop            eframe App, shell, 11 pages (FeatureModule)
   |        \
   |         ui-components      design tokens, widgets, charts, overlays (egui only)
   v
app-core                AppContext::boot, EventBus, TaskManager, AppState, notifications, crash reports
   |-- app-config       layered settings and migrations
   |-- app-storage      SQLite, migrations, repositories, import/export
   |-- app-platform     paths, file service, dialogs, platform info
   |-- app-security     path policy, limits, atomic write, CSV guard, Redacted, SecretStore
   |-- app-logging      tracing setup, in-memory ring buffer, rotating file
   `-- app-errors       AppError, ErrorReport, time helpers
```

## Rules

1. Crates below `ui-components` never import egui. The core runs in tests without a window.
2. The UI never blocks. It reads state, sends `Action`s, and starts work through `TaskManager`.
3. Background threads talk to the UI only through `EventBus`; the bus wakes the UI with `request_repaint`.
4. All failures become `AppError`; the UI shows `user_message()` and logs the detail.

## Start-up sequence (`AppContext::boot`)

paths, then config load (with migration or corrupt-file fallback), logging, crash hook and
previous-crash check, database open plus migrations, optional demo seed, task pool. The result is one
`AppContext` handed to the shell. Shutdown saves settings and drains the task pool (timeout).

## Data flow for a user action

Click, then page code calls a service or spawns a task, then the task sends `AppEvent`s,
then the shell drains them next frame and updates notifications and toasts.
