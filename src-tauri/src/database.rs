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
    pub folder_id: Option<i64>,
    /// Folder the entry belongs to. Populated only when searching/filtering
    /// (results span all folders), so cards can show their origin.
    pub folder_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: i64,
    pub name: String,
    pub entry_count: i64,
}

pub struct Database {
    conn: Mutex<Connection>,
    max_entries: AtomicI64,
}

/// Truncate to at most `max_chars` characters, appending "..." when cut.
/// Slicing by bytes (`&s[..100]`) panics on multi-byte UTF-8 content.
fn truncate_preview(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    format!("{}...", s.chars().take(max_chars).collect::<String>())
}

/// Map a DB row (id, content, content_type, preview, timestamp, is_pinned,
/// is_private, source_app, name, folder_id, folder_name) to an Entry.
/// `folder_name` is passed through as-is; SELECTs that don't need it select
/// NULL for it.
fn entry_from_row(row: &rusqlite::Row) -> rusqlite::Result<Entry> {
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
        folder_id: row.get(9)?,
        folder_name: row.get(10)?,
    })
}

impl Database {
    pub fn new(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        // Don't let a transiently-locked DB (another instance, stale WAL from
        // a crash) block a command thread forever -- fail fast instead.
        conn.busy_timeout(std::time::Duration::from_secs(2))
            .map_err(|e| e.to_string())?;
        let db = Database {
            conn: Mutex::new(conn),
            max_entries: AtomicI64::new(0),
        };
        db.migrate()?;
        Ok(db)
    }

    /// In-memory database. Used as a fallback when the on-disk DB cannot be
    /// opened (corrupt/locked after a crash): the app still runs, history is
    /// just not persisted until the DB recovers.
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
        let _ = conn.execute("ALTER TABLE entries ADD COLUMN folder_id INTEGER", []);
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS folders (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_folder ON entries(folder_id);",
        )
        .map_err(|e| e.to_string())?;
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
            truncate_preview(content, 100)
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
            folder_id: None,
            folder_name: None,
        };

        drop(conn);
        self.cleanup(self.max_entries.load(Ordering::Relaxed))?;

        Ok(entry)
    }

    /// Fetch entries for the given view. `folder` selects the pane: `None`
    /// returns unfiled history (folder members excluded), `Some(id)` returns
    /// only that folder's entries.
    ///
    /// When ANY filter is active (search query, date filter, source app),
    /// the folder restriction is dropped so results span all folders, and
    /// each result carries its folder name (`folder_name`) so cards can
    /// show where they came from.
    pub fn get_entries(
        &self,
        query: Option<&str>,
        date_filter: Option<&str>,
        source_app: Option<&str>,
        folder: Option<i64>,
    ) -> Result<Vec<Entry>, String> {
        let filtering = query.map(|q| !q.is_empty()).unwrap_or(false)
            || date_filter.map(|f| !f.is_empty()).unwrap_or(false)
            || source_app.map(|a| !a.is_empty()).unwrap_or(false);

        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut sql = String::from(
            "SELECT e.id, e.content, e.content_type, e.preview, e.timestamp, e.is_pinned, e.is_private, e.source_app, e.name, e.folder_id, f.name FROM entries e
             LEFT JOIN folders f ON e.folder_id = f.id
             WHERE 1=1",
        );
        let mut param_values: Vec<String> = Vec::new();

        if !filtering {
            match folder {
                Some(id) => {
                    sql.push_str(" AND e.folder_id = ?");
                    param_values.push(id.to_string());
                }
                None => sql.push_str(" AND e.folder_id IS NULL"),
            }
        }

        if let Some(q) = query {
            if !q.is_empty() {
                sql.push_str(" AND (");
                sql.push_str("(e.content_type = 'text' AND e.content LIKE ?)");
                param_values.push(format!("%{}%", q));
                sql.push_str(" OR e.name LIKE ?");
                param_values.push(format!("%{}%", q));
                sql.push_str(")");
            }
        }

        if let Some(app) = source_app {
            if !app.is_empty() {
                sql.push_str(" AND e.source_app = ?");
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
                        " AND e.timestamp >= {} AND e.timestamp < datetime('now', 'start of day')",
                        date_cond
                    ));
                } else {
                    sql.push_str(&format!(" AND e.timestamp >= {}", date_cond));
                }
            }
        }

        sql.push_str(" ORDER BY e.is_pinned DESC, e.timestamp DESC");
        let max_entries = self.max_entries.load(Ordering::Relaxed);
        if max_entries > 0 {
            sql.push_str(&format!(" LIMIT {}", max_entries));
        }

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params_refs.as_slice(), entry_from_row)
            .map_err(|e| e.to_string())?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|e| e.to_string())?);
        }
        Ok(entries)
    }

    /// Fetch a single entry by id. Avoids transferring the entire history
    /// (with full contents) just to read one entry.
    pub fn get_entry(&self, id: i64) -> Result<Option<Entry>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, content, content_type, preview, timestamp, is_pinned, is_private, source_app, name, folder_id, NULL FROM entries WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        match rows.next().map_err(|e| e.to_string())? {
            Some(row) => Ok(Some(entry_from_row(row).map_err(|e| e.to_string())?)),
            None => Ok(None),
        }
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
        let folder_id: Option<i64> = conn
            .query_row(
                "SELECT folder_id FROM entries WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|_| "Entry not found".to_string())?;
        if folder_id.is_some() {
            return Err("Entry is in a folder — remove it from the folder first".to_string());
        }
        conn.execute("DELETE FROM entries WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_entry(&self, id: i64, content: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let preview = truncate_preview(content, 100);
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
            conn.execute("DELETE FROM entries WHERE is_pinned = 0 AND folder_id IS NULL", [])
                .map_err(|e| e.to_string())?;
        } else {
            conn.execute("DELETE FROM entries WHERE folder_id IS NULL", [])
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn delete_in_range(&self, start: &str, end: Option<&str>) -> Result<u32, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = match end {
            Some(end) => conn
                .execute(
                    "DELETE FROM entries WHERE is_pinned = 0 AND folder_id IS NULL AND timestamp >= ?1 AND timestamp < ?2",
                    params![start, end],
                )
                .map_err(|e| e.to_string())?,
            None => conn
                .execute(
                    "DELETE FROM entries WHERE is_pinned = 0 AND folder_id IS NULL AND timestamp >= ?1",
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

    // --- Folders ---

    pub fn get_folders(&self) -> Result<Vec<Folder>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT f.id, f.name, COUNT(e.id) FROM folders f
                 LEFT JOIN entries e ON e.folder_id = f.id
                 GROUP BY f.id, f.name, f.created_at
                 ORDER BY f.name COLLATE NOCASE ASC, f.id ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Folder {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    entry_count: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut folders = Vec::new();
        for row in rows {
            folders.push(row.map_err(|e| e.to_string())?);
        }
        Ok(folders)
    }

    pub fn create_folder(&self, name: &str) -> Result<Folder, String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Folder name cannot be empty".to_string());
        }
        if name.chars().count() > 64 {
            return Err("Folder name too long (max 64 characters)".to_string());
        }
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO folders (name) VALUES (?1)",
            params![name],
        )
        .map_err(|e| e.to_string())?;
        Ok(Folder {
            id: conn.last_insert_rowid(),
            name: name.to_string(),
            entry_count: 0,
        })
    }

    pub fn rename_folder(&self, id: i64, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Folder name cannot be empty".to_string());
        }
        if name.chars().count() > 64 {
            return Err("Folder name too long (max 64 characters)".to_string());
        }
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = conn
            .execute("UPDATE folders SET name = ?1 WHERE id = ?2", params![name, id])
            .map_err(|e| e.to_string())?;
        if affected == 0 {
            return Err("Folder not found".to_string());
        }
        Ok(())
    }

    /// Delete a folder. Its entries are kept -- they return to unfiled history.
    pub fn delete_folder(&self, id: i64) -> Result<(), String> {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute("UPDATE entries SET folder_id = NULL WHERE folder_id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        let affected = tx
            .execute("DELETE FROM folders WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        if affected == 0 {
            return Err("Folder not found".to_string());
        }
        Ok(())
    }

    /// Move an entry into a folder, or out of any folder with `folder_id = None`.
    pub fn move_entry_to_folder(&self, entry_id: i64, folder_id: Option<i64>) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        if let Some(fid) = folder_id {
            let exists: i64 = conn
                .query_row("SELECT COUNT(*) FROM folders WHERE id = ?1", params![fid], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            if exists == 0 {
                return Err("Folder not found".to_string());
            }
        }
        conn.execute(
            "UPDATE entries SET folder_id = ?1 WHERE id = ?2",
            params![folder_id, entry_id],
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
                WHERE is_pinned = 0 AND folder_id IS NULL
                ORDER BY timestamp ASC
                LIMIT MAX(0, (SELECT COUNT(*) FROM entries WHERE folder_id IS NULL) - ?1)
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

        let entries = db.get_entries(None, None, None, None).unwrap();
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

        let entries = db.get_entries(None, None, None, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content_type, "image");
    }

    #[test]
    fn test_delete_entry() {
        let db = setup();
        let entry = db.add_entry("to delete", false).unwrap();
        assert_eq!(db.get_entries(None, None, None, None).unwrap().len(), 1);
        db.delete_entry(entry.id).unwrap();
        assert_eq!(db.get_entries(None, None, None, None).unwrap().len(), 0);
    }

    #[test]
    fn test_toggle_pin() {
        let db = setup();
        let entry = db.add_entry("pin me", false).unwrap();
        db.toggle_pin(entry.id).unwrap();
        let entries = db.get_entries(None, None, None, None).unwrap();
        assert!(entries[0].is_pinned);
        db.toggle_pin(entry.id).unwrap();
        let entries = db.get_entries(None, None, None, None).unwrap();
        assert!(!entries[0].is_pinned);
    }

    #[test]
    fn test_delete_in_range_today() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let fmt = |d: chrono::NaiveDateTime| {
            d.and_local_timezone(chrono::Local)
                .unwrap()
                .with_timezone(&chrono::Utc)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        };
        // Insert at local noon converted to UTC so the entry stays within the
        // local day regardless of timezone offset (UTC-relative inserts can
        // land on a different local date).
        let today = chrono::Local::now().date_naive().and_hms_opt(12, 0, 0).unwrap();
        let before = (chrono::Local::now().date_naive() - chrono::Duration::days(2))
            .and_hms_opt(12, 0, 0)
            .unwrap();
        conn.execute(
            "INSERT INTO entries (content, content_type, preview, timestamp) VALUES ('in today', 'text', 'in today', ?1)",
            params![fmt(today)],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO entries (content, content_type, preview, timestamp) VALUES ('before today', 'text', 'before today', ?1)",
            params![fmt(before)],
        )
        .unwrap();
        drop(conn);

        let start = chrono::Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
        let start_utc = fmt(start);
        let deleted = db.delete_in_range(&start_utc, None).unwrap();
        assert_eq!(deleted, 1);
        let remaining = db.get_entries(None, None, None, None).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].content, "before today");
    }

    #[test]
    fn test_delete_in_range_yesterday_excludes_pinned() {
        let db = Database::new_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let fmt = |d: chrono::NaiveDateTime| {
            d.and_local_timezone(chrono::Local)
                .unwrap()
                .with_timezone(&chrono::Utc)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        };
        let yesterday_noon = (chrono::Local::now().date_naive() - chrono::Duration::days(1))
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let yesterday_morning = (chrono::Local::now().date_naive() - chrono::Duration::days(1))
            .and_hms_opt(9, 0, 0)
            .unwrap();
        conn.execute(
            "INSERT INTO entries (content, content_type, preview, timestamp) VALUES ('yesterday normal', 'text', 'yesterday normal', ?1)",
            params![fmt(yesterday_noon)],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO entries (content, content_type, preview, timestamp, is_pinned) VALUES ('yesterday pinned', 'text', 'yesterday pinned', ?1, 1)",
            params![fmt(yesterday_morning)],
        )
        .unwrap();
        drop(conn);

        let local_now = chrono::Local::now();
        let today_start = local_now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let yesterday_start = (local_now.date_naive() - chrono::Duration::days(1)).and_hms_opt(0, 0, 0).unwrap();
        let deleted = db
            .delete_in_range(&fmt(yesterday_start), Some(&fmt(today_start)))
            .unwrap();
        assert_eq!(deleted, 1);
        let remaining = db.get_entries(None, None, None, None).unwrap();
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
        let entries = db.get_entries(None, None, None, None).unwrap();
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
        let entries = db.get_entries(None, None, None, None).unwrap();

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
        let entries = db.get_entries(None, None, None, None).unwrap();
        assert_eq!(entries.len(), 150);
    }

    #[test]
    fn test_capped_max_entries() {
        let db = Database::new_in_memory().unwrap();
        db.set_max_entries(50);
        for i in 0..75 {
            db.add_entry(&format!("entry {}", i), false).unwrap();
        }
        let entries = db.get_entries(None, None, None, None).unwrap();
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
        let entries = db.get_entries(None, None, None, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].is_pinned);

        db.clear_all(false).unwrap();
        let entries = db.get_entries(None, None, None, None).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_search() {
        let db = setup();
        db.add_entry("apple pie", false).unwrap();
        db.add_entry("banana bread", false).unwrap();
        db.add_entry("apple tart", false).unwrap();

        let results = db.get_entries(Some("apple"), None, None, None).unwrap();
        assert_eq!(results.len(), 2);

        let results = db.get_entries(Some("banana"), None, None, None).unwrap();
        assert_eq!(results.len(), 1);

        let results = db.get_entries(Some("orange"), None, None, None).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_date_filter_today() {
        let db = setup();
        db.add_entry("today entry", false).unwrap();
        let entries = db.get_entries(None, Some("today"), None, None).unwrap();
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
        let entries = db.get_entries(None, None, None, None).unwrap();
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

    #[test]
    fn test_folder_create_and_rename() {
        let db = setup();
        let folder = db.create_folder("Work").unwrap();
        assert_eq!(folder.name, "Work");
        assert_eq!(folder.entry_count, 0);

        db.rename_folder(folder.id, "Personal").unwrap();
        let folders = db.get_folders().unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].name, "Personal");
    }

    #[test]
    fn test_folder_name_validation() {
        let db = setup();
        assert!(db.create_folder("   ").is_err());
        assert!(db.create_folder(&"x".repeat(65)).is_err());
        // Duplicate names are allowed; folders are identified by id.
        db.create_folder("Work").unwrap();
        db.create_folder("Work").unwrap();
        assert_eq!(db.get_folders().unwrap().len(), 2);
    }

    #[test]
    fn test_move_entry_to_folder() {
        let db = setup();
        let folder = db.create_folder("Work").unwrap();
        let entry = db.add_entry("filed", false).unwrap();

        db.move_entry_to_folder(entry.id, Some(folder.id)).unwrap();
        let history = db.get_entries(None, None, None, None).unwrap();
        assert!(history.is_empty(), "filed entries leave the main view");

        let members = db.get_entries(None, None, None, Some(folder.id)).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].id, entry.id);
        assert_eq!(members[0].folder_id, Some(folder.id));

        db.move_entry_to_folder(entry.id, None).unwrap();
        let history = db.get_entries(None, None, None, None).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].folder_id, None);
    }

    #[test]
    fn test_move_entry_to_missing_folder() {
        let db = setup();
        let entry = db.add_entry("entry", false).unwrap();
        let result = db.move_entry_to_folder(entry.id, Some(999));
        assert!(result.is_err());
        // Entry unchanged on failure
        let history = db.get_entries(None, None, None, None).unwrap();
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_folder_members_protected_from_delete() {
        let db = setup();
        let folder = db.create_folder("Work").unwrap();
        let filed = db.add_entry("safe", false).unwrap();
        let plain = db.add_entry("deletable", false).unwrap();
        db.move_entry_to_folder(filed.id, Some(folder.id)).unwrap();

        assert!(db.delete_entry(filed.id).is_err());
        let members = db.get_entries(None, None, None, Some(folder.id)).unwrap();
        assert_eq!(members.len(), 1, "filed entry survives delete");

        db.delete_entry(plain.id).unwrap();
        let history = db.get_entries(None, None, None, None).unwrap();
        assert!(history.is_empty(), "unfiled entry deletes normally");
    }

    #[test]
    fn test_folder_members_survive_clear_all() {
        let db = setup();
        let folder = db.create_folder("Work").unwrap();
        let filed = db.add_entry("filed", false).unwrap();
        let pinned = db.add_entry("pinned one", false).unwrap();
        db.toggle_pin(pinned.id).unwrap();
        let plain = db.add_entry("plain", false).unwrap();
        db.move_entry_to_folder(filed.id, Some(folder.id)).unwrap();
        assert!(db.get_entries(None, None, None, None).unwrap().iter().any(|e| e.id == plain.id));

        db.clear_all(true).unwrap();
        let remaining = db.get_entries(None, None, None, None).unwrap();
        assert_eq!(remaining.len(), 1, "plain cleared, pinned kept");
        assert!(remaining[0].is_pinned);
        assert_eq!(
            db.get_entries(None, None, None, Some(folder.id)).unwrap().len(),
            1,
            "filed entry survives keep-pinned clear"
        );

        db.clear_all(false).unwrap();
        assert_eq!(
            db.get_entries(None, None, None, Some(folder.id)).unwrap().len(),
            1,
            "filed entry survives full clear"
        );
        assert!(db.get_entries(None, None, None, None).unwrap().is_empty());
    }

    #[test]
    fn test_folder_members_survive_delete_in_range() {
        let db = setup();
        let folder = db.create_folder("Work").unwrap();
        let filed = db.add_entry("filed", false).unwrap();
        db.add_entry("plain", false).unwrap();
        db.move_entry_to_folder(filed.id, Some(folder.id)).unwrap();

        // Filed versus unfiled: only the unfiled one is deleted.
        let deleted = db.delete_in_range("0000-01-01 00:00:00", None).unwrap();
        assert_eq!(deleted, 1, "only the unfiled entry is deleted");
        assert_eq!(
            db.get_entries(None, None, None, Some(folder.id)).unwrap().len(),
            1
        );
    }

    #[test]
    fn test_fifo_cleanup_skips_folder_members() {
        let db = Database::new_in_memory().unwrap();
        let folder = db.create_folder("Work").unwrap();
        let filed = db.add_entry("filed old", false).unwrap();

        // 12 fresh unfiled entries (newer than the filed one)
        for i in 0..12 {
            db.add_entry(&format!("entry {}", i), false).unwrap();
        }
        db.move_entry_to_folder(filed.id, Some(folder.id)).unwrap();

        db.set_max_entries(10);
        db.cleanup(10).unwrap();

        let history = db.get_entries(None, None, None, None).unwrap();
        assert_eq!(history.len(), 10, "history capped at 10 unfiled entries");
        assert_eq!(
            db.get_entries(None, None, None, Some(folder.id)).unwrap().len(),
            1,
            "filed entry never evicted"
        );
    }

    #[test]
    fn test_delete_folder_unfiles_entries() {
        let db = setup();
        let folder = db.create_folder("Work").unwrap();
        let entry = db.add_entry("entry", false).unwrap();
        db.move_entry_to_folder(entry.id, Some(folder.id)).unwrap();

        db.delete_folder(folder.id).unwrap();
        let history = db.get_entries(None, None, None, None).unwrap();
        assert_eq!(history.len(), 1, "entries return to history");
        assert_eq!(history[0].folder_id, None);
        assert!(db.get_folders().unwrap().is_empty());
    }

    #[test]
    fn test_delete_folder_missing() {
        let db = setup();
        assert!(db.delete_folder(999).is_err());
    }

    #[test]
    fn test_folder_counts() {
        let db = setup();
        let folder = db.create_folder("Work").unwrap();
        let a = db.add_entry("a", false).unwrap();
        let b = db.add_entry("b", false).unwrap();
        db.move_entry_to_folder(a.id, Some(folder.id)).unwrap();
        db.move_entry_to_folder(b.id, Some(folder.id)).unwrap();

        let folders = db.get_folders().unwrap();
        assert_eq!(folders[0].entry_count, 2);
    }

    #[test]
    fn test_pinned_entry_in_folder() {
        let db = setup();
        let folder = db.create_folder("Work").unwrap();
        let entry = db.add_entry("important", false).unwrap();
        db.toggle_pin(entry.id).unwrap();
        db.move_entry_to_folder(entry.id, Some(folder.id)).unwrap();

        // Pinned members stay in the folder and appear in its Pinned group
        let members = db.get_entries(None, None, None, Some(folder.id)).unwrap();
        assert_eq!(members.len(), 1);
        assert!(members[0].is_pinned);

        // Pinned members are included in the folder count
        let folders = db.get_folders().unwrap();
        assert_eq!(folders[0].entry_count, 1);
    }
}
