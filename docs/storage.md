# Storage

SQLite via rusqlite (bundled), foreign keys on, WAL journal for file databases. `Database` is a cloneable handle.

- **Migrations**: ordered list in `migrations.rs` (v1 items, v2 notes). Each runs in a transaction and records the version. Never edit a released migration; add a new one.
- **Repositories**: items (name, category, status, amount, updated_at) and notes.
- **Queries**: `ItemQuery` supports search, status filter, sort (enum allow-list), direction, limit and offset. Values are bound parameters.
- **Backup**: `backup_to(path)` uses the SQLite online backup API. **Restore**: `restore_from(path)` validates the source (integrity check and schema version) before replacing data; an invalid file leaves the live database untouched (tested).
- **Transfer**: CSV export (cells guarded against formula injection), JSON export, JSON import (at most 16 MiB and 50,000 rows) that applies all rows or none.
- **Demo data**: `seed_demo_data(rows)` is deterministic, used on first start (240 rows by default, `BootOptions::seed_rows`) and for benchmarks.

Data location: see `About` page or `AppPaths` (`OPENRUST_HOME` overrides).
