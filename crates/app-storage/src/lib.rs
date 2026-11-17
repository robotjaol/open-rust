//! SQLite storage foundation.
//!
//! * versioned, transactional migrations,
//! * a cloneable [`Database`] handle (connection behind a mutex; SQLite is single-writer anyway),
//! * repositories for the sample domain (items, notes),
//! * online backup, validated restore, JSON/CSV export and bounded JSON import.

mod migrations;
mod repo;
mod transfer;

pub use migrations::{MIGRATIONS, Migration};
pub use repo::{Item, ItemQuery, ItemSort, ItemStats, NewItem, Note};

use app_errors::{AppError, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

fn db_err(e: rusqlite::Error) -> AppError {
    AppError::Storage(e.to_string())
}

pub(crate) trait DbResultExt<T> {
    fn db(self) -> Result<T>;
}

impl<T> DbResultExt<T> for std::result::Result<T, rusqlite::Error> {
    fn db(self) -> Result<T> {
        self.map_err(db_err)
    }
}

/// Shared handle to the application database.
#[derive(Debug, Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    path: Option<PathBuf>,
}

impl Database {
    /// Open (creating if needed) a database file and apply pending migrations.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| AppError::io(dir, e))?;
        }
        let conn = Connection::open(path).db()?;
        Self::init(conn, Some(path.to_path_buf()))
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory().db()?, None)
    }

    fn init(mut conn: Connection, path: Option<PathBuf>) -> Result<Self> {
        conn.busy_timeout(std::time::Duration::from_secs(5)).db()?;
        conn.pragma_update(None, "foreign_keys", "ON").db()?;
        if path.is_some() {
            conn.pragma_update(None, "journal_mode", "WAL").db()?;
            conn.pragma_update(None, "synchronous", "NORMAL").db()?;
        }
        migrations::run(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            path,
        })
    }

    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| AppError::Internal("database lock poisoned".into()))
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Current schema version (number of applied migrations).
    pub fn schema_version(&self) -> Result<u32> {
        let conn = self.lock()?;
        migrations::current_version(&conn)
    }

    /// Run `f` inside a transaction; commits on `Ok`, rolls back on `Err` or panic.
    pub fn with_tx<T>(&self, f: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<T>) -> Result<T> {
        let mut conn = self.lock()?;
        let tx = conn.transaction().db()?;
        let out = f(&tx)?;
        tx.commit().db()?;
        Ok(out)
    }

    /// `PRAGMA integrity_check` — returns `Ok(())` when SQLite reports "ok".
    pub fn integrity_check(&self) -> Result<()> {
        let conn = self.lock()?;
        let res: String = conn
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .db()?;
        if res == "ok" {
            Ok(())
        } else {
            Err(AppError::Storage(format!("integrity check failed: {res}")))
        }
    }

    /// Flush the WAL into the main file (call before shutdown or backup).
    pub fn checkpoint(&self) -> Result<()> {
        let conn = self.lock()?;
        let _ = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(())); // no-op for in-memory
        Ok(())
    }

    /// Hot backup using SQLite's online backup API. Refuses to overwrite an existing file.
    pub fn backup_to(&self, dest: &Path) -> Result<()> {
        if dest.exists() {
            return Err(AppError::Validation(
                "The backup file already exists.".into(),
            ));
        }
        let src = self.lock()?;
        let mut dst = Connection::open(dest).db()?;
        {
            let backup = rusqlite::backup::Backup::new(&src, &mut dst).db()?;
            backup
                .run_to_completion(256, std::time::Duration::from_millis(5), None)
                .db()?;
        }
        Ok(())
    }

    /// Replace the live database content with a validated backup file.
    ///
    /// The source must pass `integrity_check`, contain this application's schema and must
    /// not be newer than this build understands.
    pub fn restore_from(&self, src_path: &Path) -> Result<()> {
        let src = Connection::open_with_flags(src_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .db()?;
        let ok: String = src
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .db()?;
        if ok != "ok" {
            return Err(AppError::Validation(
                "The selected file is not a healthy database.".into(),
            ));
        }
        let version = migrations::current_version(&src).map_err(|_| {
            AppError::Validation("The selected file is not an Open Rust backup.".into())
        })?;
        if version == 0 || version as usize > MIGRATIONS.len() {
            return Err(AppError::Validation(
                "The backup was made by an incompatible version.".into(),
            ));
        }
        let mut dst = self.lock()?;
        {
            let backup = rusqlite::backup::Backup::new(&src, &mut dst).db()?;
            backup
                .run_to_completion(256, std::time::Duration::from_millis(5), None)
                .db()?;
        }
        migrations::run(&mut dst)?; // bring an older backup up to date
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_apply_once() {
        let db = Database::open_in_memory().unwrap();
        assert_eq!(db.schema_version().unwrap() as usize, MIGRATIONS.len());
        db.integrity_check().unwrap();
    }

    #[test]
    fn transaction_rolls_back_on_error() {
        let db = Database::open_in_memory().unwrap();
        let res: Result<()> = db.with_tx(|tx| {
            tx.execute(
                "INSERT INTO notes(title, body, created_at, updated_at) VALUES ('t','b',0,0)",
                [],
            )
            .db()?;
            Err(AppError::Validation("boom".into()))
        });
        assert!(res.is_err());
        assert_eq!(db.list_notes().unwrap().len(), 0);
    }

    #[test]
    fn backup_and_restore_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(&dir.path().join("main.db")).unwrap();
        db.create_note("Hello", "World").unwrap();
        let backup = dir.path().join("backup.db");
        db.backup_to(&backup).unwrap();
        assert!(
            db.backup_to(&backup).is_err(),
            "must not overwrite an existing backup"
        );

        db.create_note("Second", "Later").unwrap();
        assert_eq!(db.list_notes().unwrap().len(), 2);
        db.restore_from(&backup).unwrap();
        let notes = db.list_notes().unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, "Hello");
    }

    #[test]
    fn restore_rejects_garbage_files() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open_in_memory().unwrap();
        let junk = dir.path().join("junk.db");
        std::fs::write(&junk, b"definitely not sqlite").unwrap();
        assert!(db.restore_from(&junk).is_err());
    }

    #[test]
    fn file_database_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.db");
        {
            let db = Database::open(&path).unwrap();
            db.create_note("keep", "me").unwrap();
            db.checkpoint().unwrap();
        }
        let db = Database::open(&path).unwrap();
        assert_eq!(db.list_notes().unwrap()[0].title, "keep");
    }
}
