# ADR-0005: Layered TOML configuration with schema version

Status: Accepted (2026-11-17)

## Context
Settings must survive upgrades and bad edits.

## Decision
Order: defaults, environment profile, user TOML file, OPENRUST_* variables. A schema_version field drives migrations (v1 to v2 exists). Saves are atomic (temp file then rename). A file that fails to parse is kept with a .corrupt suffix and defaults are used.

## Consequences
Positive: user data is never silently destroyed. Negative: the user loses unsaved settings until they fix or delete the corrupt file.
