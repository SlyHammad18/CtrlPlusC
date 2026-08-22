use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: i64,
    pub content: String,
    pub content_type: String,
    pub preview: String,
    pub timestamp: String,
    pub is_pinned: bool,
    pub is_private: bool,
    pub source_app: String,
    pub name: String,
}

pub struct Database {
    conn: Mutex<Connection>,
    max_entries: AtomicI64,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let db = Database {
            conn: Mutex::new(conn),
            max_entries: AtomicI64::new(0),
        };
        db.migrate()?;
        Ok(db)
    }

    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        let db = Database {
            conn: Mutex::new(conn),
            max_entries: AtomicI64::new(0),
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn set_max_entries(&self, max_entries: i64) {
        self.max_entries.store(max_entries, Ordering::Relaxed);
    }

    fn migrate(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS entries (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                content     TEXT NOT NULL DEFAULT '',
                content_type TEXT NOT NULL DEFAULT 'text',
                preview     TEXT NOT NULL DEFAULT '',
                image_data  BLOB,
                width       INTEGER NOT NULL DEFAULT 0,
                height      INTEGER NOT NULL DEFAULT 0,
                timestamp   TEXT NOT NULL DEFAULT (datetime('now')),
                is_pinned   INTEGER NOT NULL DEFAULT 0,
                is_private  INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_timestamp ON entries(timestamp);
            CREATE INDEX IF NOT EXISTS idx_pinned ON entries(is_pinned);",
        )
        .map_err(|e| e.to_string())?;
        let _ = conn.execute("ALTER TABLE entries ADD COLUMN content_type TEXT NOT NULL DEFAULT 'text'", []);
        let _ = conn.execute("ALTER TABLE entries ADD COLUMN image_data BLOB", []);
        let _ = conn.execute("ALTER TABLE entries ADD COLUMN width INTEGER NOT NULL DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE entries ADD COLUMN height INTEGER NOT NULL DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE entries ADD COLUMN source_app TEXT NOT NULL DEFAULT ''", []);
        let _ = conn.execute("ALTER TABLE entries ADD COLUMN name TEXT NOT NULL DEFAULT ''", []);
        Ok(())
    }

    pub fn add_entry(&self, content: &str, is_private: bool) -> Result<Entry, String> {
        self.add_entry_ext(content, "text", None, 0, 0, is_private, "")
    }

    pub fn add_entry_with_app(&self, content: &str, is_private: bool, source_app: &str) -> Result<Entry, String> {
        self.add_entry_ext(content, "text", None, 0, 0, is_private, source_app)
    }

    #[cfg(test)]
    pub fn add_image_entry(&self, raw_rgba: &[u8], width: u32, height: u32, is_private: bool) -> Result<Entry, String> {
        self.add_entry_ext("", "image", Some(raw_rgba), width as i32, height as i32, is_private, "")
    }

    pub fn add_image_entry_with_app(&self, raw_rgba: &[u8], width: u32, height: u32, is_private: bool, source_app: &str) -> Result<Entry, String> {
        self.add_entry_ext("", "image", Some(raw_rgba), width as i32, height as i32, is_private, source_app)
    }

    fn add_entry_ext(
        &self,
        content: &str,
        content_type: &str,
        image_data: Option<&[u8]>,
        width: i32,
        height: i32,
        is_private: bool,
        source_app: &str,
    ) -> Result<Entry, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let preview = if content_type == "image" {
            if width > 0 && height > 0 {
                format!("Image {}x{}", width, height)
            } else {
                "Image".to_string()
            }
        } else if content.len() > 100 {
            format!("{}...", &content[..100])
        } else {
            content.to_string()
        };

        conn.execute(
            "INSERT INTO entries (content, content_type, preview, image_data, width, height, is_private, source_app) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![content, content_type, preview, image_data, width, height, is_private as i32, source_app],
        )
        .map_err(|e| e.to_string())?;

        let id = conn.last_insert_rowid();

        let entry = Entry {
            id,
            content: content.to_string(),
            content_type: content_type.to_string(),
            preview,
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            is_pinned: false,
            is_private,
            source_app: source_app.to_string(),
            name: String::new(),
        };

        drop(conn);
        self.cleanup(self.max_entries.load(Ordering::Relaxed))?;

        Ok(entry)
    }

    pub fn get_entries(
        &self,
        query: Option<&str>,
        date_filter: Option<&str>,
        source_app: Option<&str>,
    ) -> Result<Vec<Entry>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut sql = String::from(
            "SELECT id, content, content_type, preview, timestamp, is_pinned, is_private, source_app, name FROM entries WHERE 1=1",
        );
        let mut param_values: Vec<String> = Vec::new();

        if let Some(q) = query {
            if !q.is_empty() {
                sql.push_str(" AND (");
                sql.push_str("(content_type = 'text' AND content LIKE ?)");
                param_values.push(format!("%{}%", q));
                sql.push_str(" OR name LIKE ?");
                param_values.push(format!("%{}%", q));
                sql.push_str(")");
            }
        }

        if let Some(app) = source_app {
            if !app.is_empty() {
                sql.push_str(" AND source_app = ?");
                param_values.push(app.to_string());
            }
        }

        if let Some(filter) = date_filter {
            let date_cond = match filter {
                "today" => "datetime('now', 'start of day')",
                "yesterday" => "datetime('now', '-1 day', 'start of day')",
                "7days" => "datetime('now', '-7 days')",
                "30days" => "datetime('now', '-30 days')",
                _ => "",
            };
            if !date_cond.is_empty() {
                if filter == "yesterday" {
                    sql.push_str(&format!(
                        " AND timestamp >= {} AND timestamp < datetime('now', 'start of day')",
                        date_cond
                    ));
                } else {
                    sql.push_str(&format!(" AND timestamp >= {}", date_cond));
                }
            }
        }

        sql.push_str(" ORDER BY is_pinned DESC, timestamp DESC");
        let max_entries = self.max_entries.load(Ordering::Relaxed);
        if max_entries > 0 {
            sql.push_str(&format!(" LIMIT {}", max_entries));
        }

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(Entry {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    content_type: row.get(2)?,
                    preview: row.get(3)?,
                    timestamp: row.get(4)?,
                    is_pinned: row.get::<_, i32>(5)? != 0,
                    is_private: row.get::<_, i32>(6)? != 0,
                    source_app: row.get(7)?,
                    name: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|e| e.to_string())?);
        }
        Ok(entries)
    }

    pub fn set_entry_name(&self, id: i64, name: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let content_type: String = conn
            .query_row(
                "SELECT content_type FROM entries WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|_| "Entry not found".to_string())?;
        if content_type == "text" {
            return Err("Text entries cannot have names".to_string());
        }
        conn.execute(
            "UPDATE entries SET name = ?1 WHERE id = ?2",
            params![name, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_app_names(&self) -> Result<Vec<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT DISTINCT source_app FROM entries WHERE source_app != '' ORDER BY source_app ASC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let mut names = Vec::new();
        for row in rows {
            names.push(row.map_err(|e| e.to_string())?);
        }
        Ok(names)
    }

    pub fn get_entry_image_data(&self, id: i64) -> Result<Option<(Vec<u8>, u32, u32)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT image_data, width, height FROM entries WHERE id = ?1 AND content_type = 'image'")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        match rows.next().map_err(|e| e.to_string())? {
            Some(row) => {
                let data: Option<Vec<u8>> = row.get(0).map_err(|e| e.to_string())?;
                let w: i32 = row.get(1).map_err(|e| e.to_string())?;
                let h: i32 = row.get(2).map_err(|e| e.to_string())?;
                match data {
                    Some(bytes) => Ok(Some((bytes, w as u32, h as u32))),
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    pub fn delete_entry(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM entries WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_entry(&self, id: i64, content: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let preview = if content.len() > 100 {
            format!("{}...", &content[..100])
        } else {
            content.to_string()
        };
        let affected = conn
            .execute(
                "UPDATE entries SET content = ?1, preview = ?2 WHERE id = ?3 AND content_type = 'text'",
                params![content, preview, id],
            )
            .map_err(|e| e.to_string())?;
        if affected == 0 {
            return Err("Entry not found or is not editable".to_string());
        }
        Ok(())
    }

    pub fn clear_all(&self, keep_pinned: bool) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        if keep_pinned {
            conn.execute("DELETE FROM entries WHERE is_pinned = 0", []).map_err(|e| e.to_string())?;
        } else {
            conn.execute("DELETE FROM entries", []).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn delete_in_range(&self, start: &str, end: Option<&str>) -> Result<u32, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = match end {
            Some(end) => conn
                .execute(
                    "DELETE FROM entries WHERE is_pinned = 0 AND timestamp >= ?1 AND timestamp < ?2",
                    params![start, end],
                )
                .map_err(|e| e.to_string())?,
            None => conn
                .execute(
                    "DELETE FROM entries WHERE is_pinned = 0 AND timestamp >= ?1",
                    params![start],
                )
                .map_err(|e| e.to_string())?,
        };
        Ok(affected as u32)
    }

    pub fn toggle_pin(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE entries SET is_pinned = CASE WHEN is_pinned = 0 THEN 1 ELSE 0 END WHERE id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn cleanup(&self, max_entries: i64) -> Result<(), String> {
        if max_entries <= 0 {
            return Ok(());
        }
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM entries WHERE id IN (
                SELECT id FROM entries
                WHERE is_pinned = 0
                ORDER BY timestamp ASC
                LIMIT MAX(0, (SELECT COUNT(*) - ?1 FROM entries))
            )",
            params![max_entries],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Database {
        Database::new_in_memory().unwrap()
    }

    #[test]
    fn test_add_and_get_entry() {
        let db = setup();
        let entry = db.add_entry("hello world", false).unwrap();
        assert_eq!(entry.content, "hello world");
        assert_eq!(entry.content_type, "text");
        assert_eq!(entry.preview, "hello world");
        assert!(!entry.is_pinned);
        assert!(!entry.is_private);

        let entries = db.get_entries(None, None, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "hello world");
        assert_eq!(entries[0].content_type, "text");
    }

    #[test]
    fn test_add_image_entry() {
        let db = setup();
        let rgba = vec![0u8; 16]; // 2x2 RGBA image
        let entry = db.add_image_entry(&rgba, 2, 2, false).unwrap();
        assert_eq!(entry.content_type, "image");
        assert!(entry.preview.contains("Image"));
        assert!(entry.preview.contains("2x2"));
        assert!(!entry.is_pinned);

        let stored = db.get_entry_image_data(entry.id).unwrap();
        assert!(stored.is_some());
        let (data, w, h) = stored.unwrap();
        assert_eq!(data, rgba);
        assert_eq!(w, 2);
        assert_eq!(h, 2);

        let entries = db.get_entries(None, None, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content_type, "image");
    }

    #[test]
    fn test_delete_entry() {
        let db = setup();
        let entry = db.add_entry("to delete", false).unwrap();
        assert_eq!(db.get_entries(None, None, None).unwrap().len(), 1);
        db.delete_entry(entry.id).unwrap();
        assert_eq!(db.get_entries(None, None, None).unwrap().len(), 0);
    }

    #[test]
    fn test_toggle_pin() {
        let db = setup();
        let entry = db.add_entry("pin me", false).unwrap();
        db.toggle_pin(entry.id).unwrap();
        let entries = db.get_entries(None, None, None).unwrap();
        assert!(entries[0].is_pinned);
        db.toggle_pin(entry.id).unwrap();
        let entries = db.get_entries(None, None, None).unwrap();
        assert!(!entries[0].is_pinned);
    }

    #[test]
    fn test_delete_in_range_today() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO entries (content, content_type, preview, timestamp) VALUES ('in today', 'text', 'in today', datetime('now', '-1 hour'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO entries (content, content_type, preview, timestamp) VALUES ('before today', 'text', 'before today', datetime('now', '-2 days'))",
            [],
        )
        .unwrap();
        drop(conn);

        let start = chrono::Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
        let start_utc = start
            .and_local_timezone(chrono::Local)
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let deleted = db.delete_in_range(&start_utc, None).unwrap();
        assert_eq!(deleted, 1);
        let remaining = db.get_entries(None, None, None).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].content, "before today");
    }

    #[test]
    fn test_delete_in_range_yesterday_excludes_pinned() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO entries (content, content_type, preview, timestamp) VALUES ('yesterday normal', 'text', 'yesterday normal', datetime('now', '-1 day', '-2 hours'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO entries (content, content_type, preview, timestamp, is_pinned) VALUES ('yesterday pinned', 'text', 'yesterday pinned', datetime('now', '-1 day', '-3 hours'), 1)",
            [],
        )
        .unwrap();
        drop(conn);

        let local_now = chrono::Local::now();
        let today_start = local_now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let yesterday_start = (local_now.date_naive() - chrono::Duration::days(1)).and_hms_opt(0, 0, 0).unwrap();
        let fmt =
            |d: chrono::NaiveDateTime| d.and_local_timezone(chrono::Local).unwrap().format("%Y-%m-%d %H:%M:%S").to_string();
        let deleted = db
            .delete_in_range(&fmt(yesterday_start), Some(&fmt(today_start)))
            .unwrap();
        assert_eq!(deleted, 1);
        let remaining = db.get_entries(None, None, None).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].content, "yesterday pinned");
    }

    #[test]
    fn test_fifo_cleanup() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();

        for i in 0..105 {
            conn.execute(
                "INSERT INTO entries (content, content_type, preview, timestamp) VALUES (?1, 'text', ?2, datetime('now', ?3))",
                params![
                    format!("entry {}", i),
                    format!("entry {}", i),
                    format!("-{} minutes", 105 - i),
                ],
            )
            .unwrap();
        }
        drop(conn);

        db.cleanup(100).unwrap();
        let entries = db.get_entries(None, None, None).unwrap();
        assert_eq!(entries.len(), 100);
        let oldest = entries.last().unwrap();
        assert_eq!(oldest.content, "entry 5");
    }

    #[test]
    fn test_fifo_keeps_pinned() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();

        for i in 0..110 {
            let pinned = if i < 15 { 1 } else { 0 };
            conn.execute(
                "INSERT INTO entries (content, content_type, preview, timestamp, is_pinned) VALUES (?1, 'text', ?2, datetime('now', ?3), ?4)",
                params![
                    format!("entry {}", i),
                    format!("entry {}", i),
                    format!("-{} minutes", 110 - i),
                    pinned,
                ],
            )
            .unwrap();
        }
        drop(conn);

        db.cleanup(100).unwrap();
        let entries = db.get_entries(None, None, None).unwrap();

        assert_eq!(entries.len(), 100);
        let pinned_count = entries.iter().filter(|e| e.is_pinned).count();
        assert_eq!(pinned_count, 15);
    }

    #[test]
    fn test_unlimited_max_entries() {
        let db = Database::new_in_memory().unwrap();
        db.set_max_entries(0);
        for i in 0..150 {
            db.add_entry(&format!("entry {}", i), false).unwrap();
        }
        let entries = db.get_entries(None, None, None).unwrap();
        assert_eq!(entries.len(), 150);
    }

    #[test]
    fn test_capped_max_entries() {
        let db = Database::new_in_memory().unwrap();
        db.set_max_entries(50);
        for i in 0..75 {
            db.add_entry(&format!("entry {}", i), false).unwrap();
        }
        let entries = db.get_entries(None, None, None).unwrap();
        assert_eq!(entries.len(), 50);
    }

    #[test]
    fn test_clear_all_keeps_pinned() {
        let db = Database::new_in_memory().unwrap();
        let pinned = db.add_entry("pinned one", false).unwrap();
        db.toggle_pin(pinned.id).unwrap();
        db.add_entry("unpinned one", false).unwrap();
        db.add_entry("unpinned two", false).unwrap();

        db.clear_all(true).unwrap();
        let entries = db.get_entries(None, None, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].is_pinned);

        db.clear_all(false).unwrap();
        let entries = db.get_entries(None, None, None).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_search() {
        let db = setup();
        db.add_entry("apple pie", false).unwrap();
        db.add_entry("banana bread", false).unwrap();
        db.add_entry("apple tart", false).unwrap();

        let results = db.get_entries(Some("apple"), None, None).unwrap();
        assert_eq!(results.len(), 2);

        let results = db.get_entries(Some("banana"), None, None).unwrap();
        assert_eq!(results.len(), 1);

        let results = db.get_entries(Some("orange"), None, None).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_date_filter_today() {
        let db = setup();
        db.add_entry("today entry", false).unwrap();
        let entries = db.get_entries(None, Some("today"), None).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_preview_truncation() {
        let db = setup();
        let long = "a".repeat(150);
        let entry = db.add_entry(&long, false).unwrap();
        assert_eq!(entry.preview.len(), 103);
        assert!(entry.preview.ends_with("..."));
    }

    #[test]
    fn test_image_entry_preview() {
        let db = setup();
        let entry = db.add_image_entry(&[0u8; 16], 4, 4, false).unwrap();
        assert_eq!(entry.preview, "Image 4x4");
    }

    #[test]
    fn test_get_entry_image_nonexistent() {
        let db = setup();
        let result = db.get_entry_image_data(999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_entry() {
        let db = setup();
        let entry = db.add_entry("original text", false).unwrap();
        db.update_entry(entry.id, "edited text").unwrap();
        let entries = db.get_entries(None, None, None).unwrap();
        assert_eq!(entries[0].content, "edited text");
        assert_eq!(entries[0].preview, "edited text");
    }

    #[test]
    fn test_update_entry_nonexistent() {
        let db = setup();
        let result = db.update_entry(999, "text");
        assert!(result.is_err());
    }

    #[test]
    fn test_update_entry_image_rejected() {
        let db = setup();
        let entry = db.add_image_entry(&[0u8; 16], 2, 2, false).unwrap();
        let result = db.update_entry(entry.id, "text");
        assert!(result.is_err());
    }
}
