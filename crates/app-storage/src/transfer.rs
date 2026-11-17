//! Data export and import.

use crate::Database;
use crate::repo::{Item, NewItem};
use app_errors::{AppError, Result};
use app_security::csv_safe;

/// Hard cap on imported rows and payload size: untrusted input must be bounded.
pub const MAX_IMPORT_ROWS: usize = 50_000;
pub const MAX_IMPORT_BYTES: usize = 16 * 1024 * 1024;

pub fn items_to_csv(items: &[Item]) -> String {
    let mut out = String::from("id,name,category,status,amount,updated_at\n");
    for i in items {
        out.push_str(&format!(
            "{},{},{},{},{:.2},{}\n",
            i.id,
            csv_safe(&i.name),
            csv_safe(&i.category),
            csv_safe(&i.status),
            i.amount,
            i.updated_at
        ));
    }
    out
}

pub fn items_to_json(items: &[Item]) -> Result<String> {
    serde_json::to_string_pretty(items).map_err(|e| AppError::Internal(format!("json export: {e}")))
}

/// Parse and validate a JSON array of items. Nothing is written until every row is valid.
pub fn parse_import_json(text: &str) -> Result<Vec<NewItem>> {
    if text.len() > MAX_IMPORT_BYTES {
        return Err(AppError::Validation("The import file is too large.".into()));
    }
    #[derive(serde::Deserialize)]
    struct Row {
        name: String,
        category: String,
        status: String,
        amount: f64,
    }
    let rows: Vec<Row> = serde_json::from_str(text).map_err(|_| {
        AppError::Validation("The file is not a valid Open Rust JSON export.".into())
    })?;
    if rows.len() > MAX_IMPORT_ROWS {
        return Err(AppError::Validation(format!(
            "An import may contain at most {MAX_IMPORT_ROWS} rows."
        )));
    }
    let items: Vec<NewItem> = rows
        .into_iter()
        .map(|r| NewItem {
            name: r.name,
            category: r.category,
            status: r.status,
            amount: r.amount,
        })
        .collect();
    for (n, i) in items.iter().enumerate() {
        i.validate()
            .map_err(|e| AppError::Validation(format!("Row {}: {}", n + 1, e.user_message())))?;
    }
    Ok(items)
}

impl Database {
    pub fn export_items_csv(&self) -> Result<String> {
        Ok(items_to_csv(&self.all_items()?))
    }

    pub fn export_items_json(&self) -> Result<String> {
        items_to_json(&self.all_items()?)
    }

    /// Import items from JSON text atomically (all rows or none).
    pub fn import_items_json(&self, text: &str) -> Result<usize> {
        let items = parse_import_json(text)?;
        self.insert_items(&items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.seed_demo_data(20).unwrap();
        db
    }

    #[test]
    fn json_export_import_round_trip() {
        let src = seeded();
        let json = src.export_items_json().unwrap();
        let dst = Database::open_in_memory().unwrap();
        assert_eq!(dst.import_items_json(&json).unwrap(), 20);
        assert_eq!(dst.count_items().unwrap(), 20);
    }

    #[test]
    fn import_is_all_or_nothing() {
        let db = Database::open_in_memory().unwrap();
        let bad = r#"[{"name":"ok","category":"c","status":"active","amount":1},
                      {"name":"","category":"c","status":"active","amount":1}]"#;
        assert!(db.import_items_json(bad).is_err());
        assert_eq!(db.count_items().unwrap(), 0);
    }

    #[test]
    fn import_rejects_malformed_and_oversized_input() {
        let db = Database::open_in_memory().unwrap();
        assert!(db.import_items_json("{not json").is_err());
        assert!(parse_import_json(&" ".repeat(MAX_IMPORT_BYTES + 1)).is_err());
    }

    #[test]
    fn csv_export_neutralises_formulas() {
        let db = Database::open_in_memory().unwrap();
        db.insert_item(&NewItem {
            name: "=cmd|' /C calc'!A0".into(),
            category: "c".into(),
            status: "active".into(),
            amount: 1.0,
        })
        .unwrap();
        let csv = db.export_items_csv().unwrap();
        assert!(csv.contains("'=cmd"), "formula must be prefixed: {csv}");
    }
}
