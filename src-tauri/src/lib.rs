mod database;

use database::{Database, Entry};
use std::fs;
use std::path::PathBuf;
use tauri::State;

fn get_db_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("ctrl-c");
    fs::create_dir_all(&path).expect("Failed to create data directory");
    path.push("history.db");
    path
}

#[tauri::command]
fn add_entry(state: State<'_, Database>, content: String, is_private: bool) -> Result<Entry, String> {
    state.add_entry(&content, is_private)
}

#[tauri::command]
fn get_entries(
    state: State<'_, Database>,
    query: Option<String>,
    date_filter: Option<String>,
) -> Result<Vec<Entry>, String> {
    state.get_entries(query.as_deref(), date_filter.as_deref())
}

#[tauri::command]
fn delete_entry(state: State<'_, Database>, id: i64) -> Result<(), String> {
    state.delete_entry(id)
}

#[tauri::command]
fn toggle_pin(state: State<'_, Database>, id: i64) -> Result<(), String> {
    state.toggle_pin(id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = get_db_path();
    let db_path_str = db_path.to_string_lossy().to_string();
    let db = Database::new(&db_path_str).expect("Failed to initialize database");

    tauri::Builder::default()
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            add_entry,
            get_entries,
            delete_entry,
            toggle_pin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
