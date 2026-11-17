# State management

Three kinds of state, each with one owner:

| State | Owner | Mutation |
|---|---|---|
| Navigation and shell (page, history, sidebar, palette, notification panel) | `AppState` in app-core | `reduce(Action)` only; transitions are unit-tested |
| Application data (config, database, tasks, notifications) | `AppContext` | Through its methods and services |
| Page-local UI state (search text, selected row, form drafts) | The page struct | Direct, inside `ui()` |

Pages receive `Cx`, which exposes `app: &mut AppContext`, `goto(page)` and `notify(...)`.
A page that changes settings sets `cx.config_dirty = true`; the shell saves after a 1.5 s
debounce and again on exit. Pages never save files themselves.

Cached data (for example the Data table rows) is reloaded in `on_enter` or when a `stale`
flag is set; there is no global cache invalidation layer, by design.
