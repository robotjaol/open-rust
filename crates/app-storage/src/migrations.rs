//! Ordered, append-only schema migrations. Never edit a released migration: add a new one.

use crate::DbResultExt;
use app_errors::{AppError, Result};
use rusqlite::Connection;

#[derive(Debug, Clone, Copy)]
pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "create_items",
        sql: "CREATE TABLE items (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                name       TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 200),
                category   TEXT NOT NULL,
                status     TEXT NOT NULL CHECK (status IN ('active','paused','archived')),
                amount     REAL NOT NULL CHECK (amount >= 0),
                updated_at INTEGER NOT NULL
              );
              CREATE INDEX idx_items_category ON items(category);
              CREATE INDEX idx_items_name ON items(name);",
    },
    Migration {
        version: 2,
        name: "create_notes",
        sql: "CREATE TABLE notes (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                title      TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 200),
                body       TEXT NOT NULL DEFAULT '',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
              );",
    },
];

fn ensure_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version    INTEGER PRIMARY KEY,
            name       TEXT NOT NULL,
            applied_at INTEGER NOT NULL
         );",
    )
    .db()
}

pub fn current_version(conn: &Connection) -> Result<u32> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_migrations')",
            [],
            |r| r.get(0),
        )
        .db()?;
    if !exists {
        return Ok(0);
    }
    let v: Option<u32> = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
            r.get(0)
        })
        .db()?;
    Ok(v.unwrap_or(0))
}

/// Apply every migration newer than the database's version, each in its own transaction.
pub fn run(conn: &mut Connection) -> Result<()> {
    ensure_table(conn)?;
    let current = current_version(conn)?;
    if current as usize > MIGRATIONS.len() {
        return Err(AppError::Storage(format!(
            "database schema {current} is newer than this application supports ({})",
            MIGRATIONS.len()
        )));
    }
    for m in MIGRATIONS.iter().filter(|m| m.version > current) {
        let tx = conn.transaction().db()?;
        tx.execute_batch(m.sql).db()?;
        tx.execute(
            "INSERT INTO schema_migrations(version, name, applied_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![m.version, m.name, app_errors::time::now_unix() as i64],
        )
        .db()?;
        tx.commit().db()?;
        tracing::info!(version = m.version, name = m.name, "applied migration");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_are_contiguous_and_ascending() {
        for (i, m) in MIGRATIONS.iter().enumerate() {
            assert_eq!(m.version as usize, i + 1);
        }
    }

    #[test]
    fn running_twice_is_idempotent() {
        let mut c = Connection::open_in_memory().unwrap();
        run(&mut c).unwrap();
        run(&mut c).unwrap();
        assert_eq!(current_version(&c).unwrap() as usize, MIGRATIONS.len());
    }

    #[test]
    fn newer_database_is_refused() {
        let mut c = Connection::open_in_memory().unwrap();
        run(&mut c).unwrap();
        c.execute(
            "INSERT INTO schema_migrations VALUES (999, 'future', 0)",
            [],
        )
        .unwrap();
        assert!(run(&mut c).is_err());
    }
}
