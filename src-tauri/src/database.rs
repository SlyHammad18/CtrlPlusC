use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
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
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
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
        Ok(())
    }

    pub fn add_entry(&self, content: &str, is_private: bool) -> Result<Entry, String> {
        self.add_entry_ext(content, "text", None, 0, 0, is_private)
    }

    pub fn add_image_entry(&self, raw_rgba: &[u8], width: u32, height: u32, is_private: bool) -> Result<Entry, String> {
        self.add_entry_ext("", "image", Some(raw_rgba), width as i32, height as i32, is_private)
    }

    fn add_entry_ext(
        &self,
        content: &str,
        content_type: &str,
        image_data: Option<&[u8]>,
        width: i32,
        height: i32,
        is_private: bool,
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
            "INSERT INTO entries (content, content_type, preview, image_data, width, height, is_private) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![content, content_type, preview, image_data, width, height, is_private as i32],
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
        };

        drop(conn);
        self.cleanup(100)?;

        Ok(entry)
    }

    pub fn get_entries(
        &self,
        query: Option<&str>,
        date_filter: Option<&str>,
    ) -> Result<Vec<Entry>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut sql = String::from(
            "SELECT id, content, content_type, preview, timestamp, is_pinned, is_private FROM entries WHERE 1=1",
        );
        let mut param_values: Vec<String> = Vec::new();

        if let Some(q) = query {
            if !q.is_empty() {
                sql.push_str(" AND content_type = 'text' AND content LIKE ?");
                param_values.push(format!("%{}%", q));
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

        sql.push_str(" ORDER BY is_pinned DESC, timestamp DESC LIMIT 100");

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
                })
            })
            .map_err(|e| e.to_string())?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|e| e.to_string())?);
        }
        Ok(entries)
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

    pub fn clear_all(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM entries", []).map_err(|e| e.to_string())?;
        Ok(())
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

    fn cleanup(&self, max_entries: usize) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM entries WHERE id IN (
                SELECT id FROM entries
                WHERE is_pinned = 0
                ORDER BY timestamp ASC
                LIMIT MAX(0, (SELECT COUNT(*) - ?1 FROM entries))
            )",
            params![max_entries as i64],
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

        let entries = db.get_entries(None, None).unwrap();
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

        let entries = db.get_entries(None, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content_type, "image");
    }

    #[test]
    fn test_delete_entry() {
        let db = setup();
        let entry = db.add_entry("to delete", false).unwrap();
        assert_eq!(db.get_entries(None, None).unwrap().len(), 1);
        db.delete_entry(entry.id).unwrap();
        assert_eq!(db.get_entries(None, None).unwrap().len(), 0);
    }

    #[test]
    fn test_toggle_pin() {
        let db = setup();
        let entry = db.add_entry("pin me", false).unwrap();
        db.toggle_pin(entry.id).unwrap();
        let entries = db.get_entries(None, None).unwrap();
        assert!(entries[0].is_pinned);
        db.toggle_pin(entry.id).unwrap();
        let entries = db.get_entries(None, None).unwrap();
        assert!(!entries[0].is_pinned);
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
        let entries = db.get_entries(None, None).unwrap();
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
        let entries = db.get_entries(None, None).unwrap();

        assert_eq!(entries.len(), 100);
        let pinned_count = entries.iter().filter(|e| e.is_pinned).count();
        assert_eq!(pinned_count, 15);
    }

    #[test]
    fn test_search() {
        let db = setup();
        db.add_entry("apple pie", false).unwrap();
        db.add_entry("banana bread", false).unwrap();
        db.add_entry("apple tart", false).unwrap();

        let results = db.get_entries(Some("apple"), None).unwrap();
        assert_eq!(results.len(), 2);

        let results = db.get_entries(Some("banana"), None).unwrap();
        assert_eq!(results.len(), 1);

        let results = db.get_entries(Some("orange"), None).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_does_not_include_images() {
        let db = setup();
        db.add_entry("apple pie", false).unwrap();
        db.add_image_entry(&[0u8; 4], 1, 1, false).unwrap();

        let results = db.get_entries(Some("apple"), None).unwrap();
        assert_eq!(results.len(), 1);

        let results = db.get_entries(None, None).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_date_filter_today() {
        let db = setup();
        db.add_entry("today entry", false).unwrap();
        let entries = db.get_entries(None, Some("today")).unwrap();
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
        let entries = db.get_entries(None, None).unwrap();
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
