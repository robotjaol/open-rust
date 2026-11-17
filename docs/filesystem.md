# Filesystem

All file access goes through `FileService` (app-platform) and helpers in `app-security`.

- `list_dir` returns entries with size, modified time and hidden flag; hidden files are shown only on request.
- `read_text` enforces `max_file_size_mb` (setting, default 16) before reading, and rejects directories.
- `write_atomic` writes a temporary file in the target directory, syncs, then renames.
- `PathPolicy` canonicalises a path and checks it stays under an allowed root; symlink escapes are rejected (tested).
- `validate_file_name` rejects separators, `..`, reserved Windows names, control characters and overlong names.
- Native dialogs run on a worker thread and return the choice as an event; if no dialog backend exists the UI says so.

The Files page is read-only on purpose: it shows how to browse and preview safely without
offering delete or write actions that would need more policy.
