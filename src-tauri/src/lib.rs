mod clipboard;
mod config;
mod database;

use clipboard::ClipboardMonitor;
use config::Config;
use database::{Database, Entry};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tauri::State;

fn get_db_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("ctrl-c");
    fs::create_dir_all(&path).expect("Failed to create data directory");
    path.push("history.db");
    path
}

#[tauri::command]
fn add_entry(state: State<'_, Arc<Database>>, content: String, is_private: bool) -> Result<Entry, String> {
    state.add_entry(&content, is_private)
}

#[tauri::command]
fn get_entries(
    state: State<'_, Arc<Database>>,
    query: Option<String>,
    date_filter: Option<String>,
) -> Result<Vec<Entry>, String> {
    state.get_entries(query.as_deref(), date_filter.as_deref())
}

#[tauri::command]
fn delete_entry(state: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    state.delete_entry(id)
}

#[tauri::command]
fn toggle_pin(state: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    state.toggle_pin(id)
}

#[tauri::command]
fn get_config(state: State<'_, Mutex<Config>>) -> Result<Config, String> {
    state.lock().map_err(|e| e.to_string()).map(|c| c.clone())
}

#[tauri::command]
fn save_config(state: State<'_, Mutex<Config>>, config: Config) -> Result<(), String> {
    config::save_config(&config)?;
    let mut stored = state.lock().map_err(|e| e.to_string())?;
    *stored = config;
    Ok(())
}

#[tauri::command]
fn set_private_mode(
    monitor: State<'_, ClipboardMonitor>,
    locked: bool,
) -> Result<(), String> {
    monitor.private_mode.store(locked, Ordering::Relaxed);
    if locked {
        if let Ok(mut last) = monitor.last_content.lock() {
            *last = None;
        }
    }
    Ok(())
}

#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clip = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clip.set_text(text).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn check_clipboard(
    db: State<'_, Arc<Database>>,
    monitor: State<'_, ClipboardMonitor>,
) -> Result<Option<Entry>, String> {
    if monitor.private_mode.load(Ordering::Relaxed) {
        return Ok(None);
    }

    let text = match arboard::Clipboard::new() {
        Ok(mut clip) => match clip.get_text() {
            Ok(t) => t,
            Err(_) => return Ok(None),
        },
        Err(_) => return Ok(None),
    };

    let mut last = monitor.last_content.lock().map_err(|e| e.to_string())?;
    if last.as_ref() == Some(&text) {
        return Ok(None);
    }
    *last = Some(text.clone());
    drop(last);

    match db.add_entry(&text, false) {
        Ok(entry) => Ok(Some(entry)),
        Err(ref e) if e == "duplicate entry" => Ok(None),
        Err(e) => Err(e),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = get_db_path();
    let db_path_str = db_path.to_string_lossy().to_string();
    let db = Arc::new(Database::new(&db_path_str).expect("Failed to initialize database"));
    let config = Mutex::new(config::load_config());
    let monitor = ClipboardMonitor::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(db.clone())
        .manage(config)
        .manage(monitor)
        .invoke_handler(tauri::generate_handler![
            add_entry,
            get_entries,
            delete_entry,
            toggle_pin,
            get_config,
            save_config,
            set_private_mode,
            copy_to_clipboard,
            check_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
