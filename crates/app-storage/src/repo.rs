//! Repositories for the sample domain. Plain SQL with bound parameters only.

use crate::{Database, DbResultExt};
use app_errors::{AppError, Result, time};
use rusqlite::{Row, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub status: String,
    pub amount: f64,
    pub updated_at: i64,
}

/// Fields required to create an item. Validated by [`NewItem::validate`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewItem {
    pub name: String,
    pub category: String,
    pub status: String,
    pub amount: f64,
}

pub const STATUSES: [&str; 3] = ["active", "paused", "archived"];

impl NewItem {
    pub fn validate(&self) -> Result<()> {
        let name = self.name.trim();
        if name.is_empty() || name.chars().count() > 200 {
            return Err(AppError::Validation(
                "Name must be 1 to 200 characters.".into(),
            ));
        }
        if self.category.trim().is_empty() || self.category.chars().count() > 60 {
            return Err(AppError::Validation(
                "Category must be 1 to 60 characters.".into(),
            ));
        }
        if !STATUSES.contains(&self.status.as_str()) {
            return Err(AppError::Validation(
                "Status must be active, paused or archived.".into(),
            ));
        }
        if !self.amount.is_finite() || self.amount < 0.0 || self.amount > 1.0e12 {
            return Err(AppError::Validation(
                "Amount must be a non-negative number.".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItemSort {
    Id,
    #[default]
    Name,
    Category,
    Status,
    Amount,
    Updated,
}

impl ItemSort {
    /// Column name from a fixed allow-list: never interpolate user text into SQL.
    fn column(self) -> &'static str {
        match self {
            ItemSort::Id => "id",
            ItemSort::Name => "name COLLATE NOCASE",
            ItemSort::Category => "category COLLATE NOCASE",
            ItemSort::Status => "status",
            ItemSort::Amount => "amount",
            ItemSort::Updated => "updated_at",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ItemQuery {
    pub search: String,
    pub status: Option<String>,
    pub sort: ItemSort,
    pub descending: bool,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ItemStats {
    pub total: u64,
    pub active: u64,
    pub paused: u64,
    pub archived: u64,
    pub total_amount: f64,
    /// (category, count) sorted by count descending.
    pub by_category: Vec<(String, u64)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub created_at: i64,
    pub updated_at: i64,
}

fn item_from_row(r: &Row<'_>) -> rusqlite::Result<Item> {
    Ok(Item {
        id: r.get(0)?,
        name: r.get(1)?,
        category: r.get(2)?,
        status: r.get(3)?,
        amount: r.get(4)?,
        updated_at: r.get(5)?,
    })
}

fn like_pattern(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

impl Database {
    pub fn insert_item(&self, new: &NewItem) -> Result<i64> {
        new.validate()?;
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO items(name, category, status, amount, updated_at) VALUES (?1,?2,?3,?4,?5)",
            params![
                new.name.trim(),
                new.category.trim(),
                new.status,
                new.amount,
                time::now_unix() as i64
            ],
        )
        .db()?;
        Ok(conn.last_insert_rowid())
    }

    pub fn insert_items(&self, items: &[NewItem]) -> Result<usize> {
        for i in items {
            i.validate()?;
        }
        self.with_tx(|tx| {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO items(name, category, status, amount, updated_at) VALUES (?1,?2,?3,?4,?5)",
                )
                .db()?;
            for i in items {
                stmt.execute(params![i.name.trim(), i.category.trim(), i.status, i.amount, time::now_unix() as i64])
                    .db()?;
            }
            Ok(items.len())
        })
    }

    pub fn delete_item(&self, id: i64) -> Result<bool> {
        Ok(self
            .lock()?
            .execute("DELETE FROM items WHERE id = ?1", [id])
            .db()?
            > 0)
    }

    pub fn set_item_status(&self, id: i64, status: &str) -> Result<()> {
        if !STATUSES.contains(&status) {
            return Err(AppError::Validation("Unknown status.".into()));
        }
        self.lock()?
            .execute(
                "UPDATE items SET status = ?1, updated_at = ?2 WHERE id = ?3",
                params![status, time::now_unix() as i64, id],
            )
            .db()?;
        Ok(())
    }

    pub fn count_items(&self) -> Result<u64> {
        self.lock()?
            .query_row("SELECT COUNT(*) FROM items", [], |r| r.get::<_, i64>(0))
            .db()
            .map(|n| n as u64)
    }

    /// Search, sort and paginate. Returns the page and the total number of matching rows.
    pub fn query_items(&self, q: &ItemQuery) -> Result<(Vec<Item>, u64)> {
        let conn = self.lock()?;
        let pattern = like_pattern(q.search.trim());
        let status = q.status.clone().unwrap_or_default();
        let where_clause = "(name LIKE ?1 ESCAPE '\\' OR category LIKE ?1 ESCAPE '\\') AND (?2 = '' OR status = ?2)";
        let total: i64 = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM items WHERE {where_clause}"),
                params![pattern, status],
                |r| r.get(0),
            )
            .db()?;
        let dir = if q.descending { "DESC" } else { "ASC" };
        let sql = format!(
            "SELECT id, name, category, status, amount, updated_at FROM items WHERE {where_clause}
             ORDER BY {} {dir}, id ASC LIMIT ?3 OFFSET ?4",
            q.sort.column()
        );
        let limit = q.limit.clamp(1, 1000) as i64;
        let mut stmt = conn.prepare(&sql).db()?;
        let rows = stmt
            .query_map(
                params![pattern, status, limit, q.offset as i64],
                item_from_row,
            )
            .db()?
            .collect::<rusqlite::Result<Vec<_>>>()
            .db()?;
        Ok((rows, total as u64))
    }

    pub fn all_items(&self) -> Result<Vec<Item>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare("SELECT id, name, category, status, amount, updated_at FROM items ORDER BY id")
            .db()?;
        stmt.query_map([], item_from_row)
            .db()?
            .collect::<rusqlite::Result<Vec<_>>>()
            .db()
    }

    pub fn item_stats(&self) -> Result<ItemStats> {
        let conn = self.lock()?;
        let mut s = ItemStats::default();
        conn.query_row(
            "SELECT COUNT(*),
                    COALESCE(SUM(status='active'),0), COALESCE(SUM(status='paused'),0),
                    COALESCE(SUM(status='archived'),0), COALESCE(SUM(amount),0.0) FROM items",
            [],
            |r| {
                s.total = r.get::<_, i64>(0)? as u64;
                s.active = r.get::<_, i64>(1)? as u64;
                s.paused = r.get::<_, i64>(2)? as u64;
                s.archived = r.get::<_, i64>(3)? as u64;
                s.total_amount = r.get(4)?;
                Ok(())
            },
        )
        .db()?;
        let mut stmt = conn
            .prepare("SELECT category, COUNT(*) c FROM items GROUP BY category ORDER BY c DESC, category LIMIT 8")
            .db()?;
        s.by_category = stmt
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u64))
            })
            .db()?
            .collect::<rusqlite::Result<Vec<_>>>()
            .db()?;
        Ok(s)
    }

    // ---- notes ----

    pub fn create_note(&self, title: &str, body: &str) -> Result<i64> {
        let title = title.trim();
        if title.is_empty() || title.chars().count() > 200 {
            return Err(AppError::Validation(
                "Title must be 1 to 200 characters.".into(),
            ));
        }
        if body.len() > 1_000_000 {
            return Err(AppError::Validation(
                "A note can hold at most 1 MB of text.".into(),
            ));
        }
        let now = time::now_unix() as i64;
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO notes(title, body, created_at, updated_at) VALUES (?1,?2,?3,?3)",
            params![title, body, now],
        )
        .db()?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_note(&self, id: i64, title: &str, body: &str) -> Result<()> {
        let title = title.trim();
        if title.is_empty() || title.chars().count() > 200 {
            return Err(AppError::Validation(
                "Title must be 1 to 200 characters.".into(),
            ));
        }
        if body.len() > 1_000_000 {
            return Err(AppError::Validation(
                "A note can hold at most 1 MB of text.".into(),
            ));
        }
        let n = self
            .lock()?
            .execute(
                "UPDATE notes SET title=?1, body=?2, updated_at=?3 WHERE id=?4",
                params![title, body, time::now_unix() as i64, id],
            )
            .db()?;
        if n == 0 {
            Err(AppError::Validation("That note no longer exists.".into()))
        } else {
            Ok(())
        }
    }

    pub fn delete_note(&self, id: i64) -> Result<bool> {
        Ok(self
            .lock()?
            .execute("DELETE FROM notes WHERE id = ?1", [id])
            .db()?
            > 0)
    }

    pub fn list_notes(&self) -> Result<Vec<Note>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare("SELECT id, title, body, created_at, updated_at FROM notes ORDER BY updated_at DESC, id DESC")
            .db()?;
        stmt.query_map([], |r| {
            Ok(Note {
                id: r.get(0)?,
                title: r.get(1)?,
                body: r.get(2)?,
                created_at: r.get(3)?,
                updated_at: r.get(4)?,
            })
        })
        .db()?
        .collect::<rusqlite::Result<Vec<_>>>()
        .db()
    }

    /// Insert deterministic sample rows when the table is empty (first run demo data).
    pub fn seed_demo_data(&self, rows: usize) -> Result<usize> {
        if self.count_items()? > 0 {
            return Ok(0);
        }
        const CATS: [&str; 6] = [
            "Infrastructure",
            "Security",
            "Billing",
            "Analytics",
            "Support",
            "Research",
        ];
        const NOUNS: [&str; 8] = [
            "Gateway",
            "Pipeline",
            "Dashboard",
            "Scheduler",
            "Exporter",
            "Indexer",
            "Monitor",
            "Archive",
        ];
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15; // fixed seed: deterministic screenshots and tests
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let items: Vec<NewItem> = (0..rows)
            .map(|i| {
                let r = next();
                NewItem {
                    name: format!("{} {:03}", NOUNS[(r % 8) as usize], i + 1),
                    category: CATS[((r >> 8) % 6) as usize].to_string(),
                    status: STATUSES[(((r >> 16) % 10).min(8) / 3) as usize % 3].to_string(),
                    amount: ((r >> 24) % 900_000) as f64 / 100.0 + 25.0,
                }
            })
            .collect();
        let n = self.insert_items(&items)?;
        if self.list_notes()?.is_empty() {
            self.create_note(
                "Welcome to Open Rust",
                "This note lives in a local SQLite database.\n\nEdit it, add more, or export everything from the Data page.",
            )?;
            self.create_note(
                "Architecture checklist",
                "- Keep UI code free of business rules\n- Run long work through the TaskManager\n- Never show internal errors to users",
            )?;
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn db() -> Database {
        Database::open_in_memory().unwrap()
    }

    fn sample(name: &str) -> NewItem {
        NewItem {
            name: name.into(),
            category: "Cat".into(),
            status: "active".into(),
            amount: 10.0,
        }
    }

    #[test]
    fn insert_query_delete() {
        let db = db();
        let id = db.insert_item(&sample("Alpha")).unwrap();
        db.insert_item(&sample("Beta")).unwrap();
        let (rows, total) = db
            .query_items(&ItemQuery {
                limit: 10,
                ..Default::default()
            })
            .unwrap();
        assert_eq!((rows.len(), total), (2, 2));
        assert!(db.delete_item(id).unwrap());
        assert!(!db.delete_item(id).unwrap());
        assert_eq!(db.count_items().unwrap(), 1);
    }

    #[test]
    fn validation_rejects_bad_input() {
        let db = db();
        assert!(db.insert_item(&sample("  ")).is_err());
        let mut bad = sample("x");
        bad.amount = f64::NAN;
        assert!(db.insert_item(&bad).is_err());
        bad.amount = 1.0;
        bad.status = "weird".into();
        assert!(db.insert_item(&bad).is_err());
    }

    #[test]
    fn search_sort_and_paginate() {
        let db = db();
        for (n, a) in [("Gamma", 3.0), ("alpha", 1.0), ("Beta", 2.0)] {
            let mut it = sample(n);
            it.amount = a;
            db.insert_item(&it).unwrap();
        }
        let q = ItemQuery {
            sort: ItemSort::Name,
            limit: 2,
            ..Default::default()
        };
        let (rows, total) = db.query_items(&q).unwrap();
        assert_eq!(total, 3);
        assert_eq!(
            rows.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
            ["alpha", "Beta"]
        );
        let q = ItemQuery {
            sort: ItemSort::Amount,
            descending: true,
            limit: 1,
            ..Default::default()
        };
        assert_eq!(db.query_items(&q).unwrap().0[0].name, "Gamma");
        let q = ItemQuery {
            search: "ALP".into(),
            limit: 10,
            ..Default::default()
        };
        assert_eq!(db.query_items(&q).unwrap().1, 1);
    }

    #[test]
    fn like_wildcards_in_search_are_literal() {
        let db = db();
        db.insert_item(&sample("100% pure")).unwrap();
        db.insert_item(&sample("plain")).unwrap();
        let q = ItemQuery {
            search: "%".into(),
            limit: 10,
            ..Default::default()
        };
        assert_eq!(db.query_items(&q).unwrap().1, 1);
    }

    #[test]
    fn sql_injection_attempt_is_inert() {
        let db = db();
        db.insert_item(&sample("safe")).unwrap();
        let q = ItemQuery {
            search: "'; DROP TABLE items; --".into(),
            limit: 10,
            ..Default::default()
        };
        assert_eq!(db.query_items(&q).unwrap().1, 0);
        assert_eq!(db.count_items().unwrap(), 1);
    }

    #[test]
    fn stats_aggregate_correctly() {
        let db = db();
        db.insert_item(&sample("a")).unwrap();
        let mut p = sample("b");
        p.status = "paused".into();
        db.insert_item(&p).unwrap();
        let s = db.item_stats().unwrap();
        assert_eq!((s.total, s.active, s.paused, s.archived), (2, 1, 1, 0));
        assert!((s.total_amount - 20.0).abs() < 1e-9);
    }

    #[test]
    fn notes_crud() {
        let db = db();
        let id = db.create_note("T", "B").unwrap();
        db.update_note(id, "T2", "B2").unwrap();
        assert_eq!(db.list_notes().unwrap()[0].title, "T2");
        assert!(db.update_note(9999, "x", "y").is_err());
        assert!(db.create_note("", "b").is_err());
        assert!(db.delete_note(id).unwrap());
    }

    #[test]
    fn seeding_is_deterministic_and_idempotent() {
        let a = db();
        let b = db();
        assert_eq!(a.seed_demo_data(50).unwrap(), 50);
        b.seed_demo_data(50).unwrap();
        assert_eq!(a.seed_demo_data(50).unwrap(), 0);
        assert_eq!(
            a.all_items()
                .unwrap()
                .iter()
                .map(|i| &i.name)
                .collect::<Vec<_>>(),
            b.all_items()
                .unwrap()
                .iter()
                .map(|i| &i.name)
                .collect::<Vec<_>>()
        );
    }

    proptest! {
        #[test]
        fn arbitrary_search_text_never_errors(search in ".{0,60}") {
            let db = Database::open_in_memory().unwrap();
            db.insert_item(&sample("x")).unwrap();
            let q = ItemQuery { search, limit: 5, ..Default::default() };
            prop_assert!(db.query_items(&q).is_ok());
        }

        #[test]
        fn valid_items_always_round_trip(name in "[a-zA-Z0-9 ]{1,50}", amount in 0.0f64..1.0e9) {
            prop_assume!(!name.trim().is_empty());
            let db = Database::open_in_memory().unwrap();
            let id = db.insert_item(&NewItem { name: name.clone(), category: "c".into(), status: "paused".into(), amount }).unwrap();
            let items = db.all_items().unwrap();
            let got = items.iter().find(|i| i.id == id).unwrap();
            prop_assert_eq!(&got.name, name.trim());
            prop_assert!((got.amount - amount).abs() < 1e-6);
        }
    }
}
