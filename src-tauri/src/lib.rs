mod autostart;
mod clipboard;
mod config;
mod database;
mod hotkey;
mod private_mode;

use clipboard::ClipboardMonitor;
use config::Config;
use database::{Database, Entry};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[derive(Debug, Clone, serde::Serialize)]
pub struct PrivateModeStatus {
    pub locked: bool,
    pub has_password: bool,
}

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
fn get_private_mode_status(
    config: State<'_, Mutex<Config>>,
    monitor: State<'_, ClipboardMonitor>,
) -> Result<PrivateModeStatus, String> {
    let cfg = config.lock().map_err(|e| e.to_string())?;
    Ok(PrivateModeStatus {
        locked: cfg.private_mode_locked || monitor.private_mode.load(Ordering::Relaxed),
        has_password: !cfg.private_mode_password_hash.is_empty(),
    })
}

#[tauri::command]
fn set_private_mode_password(
    config: State<'_, Mutex<Config>>,
    monitor: State<'_, ClipboardMonitor>,
    password: String,
) -> Result<(), String> {
    if password.len() < 4 {
        return Err("Password must be at least 4 characters".to_string());
    }
    let hash = private_mode::hash_password(&password)?;
    let mut cfg = config.lock().map_err(|e| e.to_string())?;
    cfg.private_mode_password_hash = hash;
    cfg.private_mode_locked = true;
    config::save_config(&cfg)?;
    monitor.private_mode.store(true, Ordering::Relaxed);
    if let Ok(mut last) = monitor.last_content.lock() {
        *last = None;
    }
    Ok(())
}

#[tauri::command]
fn lock_private_mode(
    config: State<'_, Mutex<Config>>,
    monitor: State<'_, ClipboardMonitor>,
) -> Result<(), String> {
    let mut cfg = config.lock().map_err(|e| e.to_string())?;
    if cfg.private_mode_password_hash.is_empty() {
        return Err("No password set".to_string());
    }
    cfg.private_mode_locked = true;
    config::save_config(&cfg)?;
    monitor.private_mode.store(true, Ordering::Relaxed);
    if let Ok(mut last) = monitor.last_content.lock() {
        *last = None;
    }
    Ok(())
}

#[tauri::command]
fn unlock_private_mode(
    config: State<'_, Mutex<Config>>,
    monitor: State<'_, ClipboardMonitor>,
    password: String,
) -> Result<bool, String> {
    let cfg = config.lock().map_err(|e| e.to_string())?;
    let hash = cfg.private_mode_password_hash.clone();
    drop(cfg);

    let valid = private_mode::verify_password(&password, &hash)?;
    if !valid {
        return Ok(false);
    }

    let mut cfg = config.lock().map_err(|e| e.to_string())?;
    cfg.private_mode_locked = false;
    config::save_config(&cfg)?;
    monitor.private_mode.store(false, Ordering::Relaxed);
    Ok(true)
}

#[tauri::command]
fn enable_autostart() -> Result<(), String> {
    autostart::enable_autostart()
}

#[tauri::command]
fn disable_autostart() -> Result<(), String> {
    autostart::disable_autostart()
}

#[tauri::command]
fn is_autostart_enabled() -> Result<bool, String> {
    autostart::is_autostart_enabled()
}

#[cfg(target_os = "windows")]
fn simulate_paste() {
    unsafe {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
        const VK_CONTROL: u16 = 0x11;
        const VK_V: u16 = 0x56;
        keybd_event(VK_CONTROL as u8, 0, 0, 0);
        keybd_event(VK_V as u8, 0, 0, 0);
        std::thread::sleep(std::time::Duration::from_millis(15));
        keybd_event(VK_V as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_KEYUP, 0);
    }
}

#[cfg(target_os = "linux")]
fn simulate_paste() {
    let _ = std::process::Command::new("xdotool")
        .args(["key", "ctrl+v"])
        .spawn();
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn simulate_paste() {}

#[tauri::command]
fn copy_and_paste(text: String, monitor: State<'_, ClipboardMonitor>) -> Result<(), String> {
    let mut clip = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clip.set_text(&text).map_err(|e| e.to_string())?;
    drop(clip);
    if let Ok(mut last) = monitor.last_app_copy.lock() {
        *last = Some(text.clone());
    }
    simulate_paste();
    Ok(())
}

#[tauri::command]
fn copy_to_clipboard(text: String, monitor: State<'_, ClipboardMonitor>) -> Result<(), String> {
    let mut clip = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clip.set_text(&text).map_err(|e| e.to_string())?;
    if let Ok(mut last) = monitor.last_app_copy.lock() {
        *last = Some(text);
    }
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

    {
        let mut last = monitor.last_content.lock().map_err(|e| e.to_string())?;
        if last.as_ref() == Some(&text) {
            return Ok(None);
        }
        *last = Some(text.clone());
    }

    // If this text was just copied from within the app, don't re-add it
    {
        let mut app = monitor.last_app_copy.lock().map_err(|e| e.to_string())?;
        if app.as_ref() == Some(&text) {
            *app = None;
            return Ok(None);
        }
    }

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
    let loaded_config = config::load_config();
    let is_locked = loaded_config.private_mode_locked;
    let monitor = ClipboardMonitor::new();

    if is_locked {
        monitor.private_mode.store(true, Ordering::Relaxed);
    }

    if loaded_config.autostart {
        if let Err(e) = autostart::enable_autostart() {
            eprintln!("Ctrl+C: Failed to enable autostart: {}", e);
        }
    }

    let config = Mutex::new(loaded_config);

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .manage(db.clone())
        .manage(config)
        .manage(monitor)
        .setup(|app| {
            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::TrayIconBuilder;

            let show_hide = MenuItemBuilder::with_id("show_hide", "Show/Hide")
                .build(app)?;
            let private_mode = MenuItemBuilder::with_id("lock_private", "Lock Private Mode")
                .build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit")
                .build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show_hide)
                .separator()
                .item(&private_mode)
                .separator()
                .item(&quit)
                .build()?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Ctrl+C — Clipboard Manager")
                .menu(&menu)
                .on_tray_icon_event(|tray, event| {
                    use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show_hide" => {
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                        "lock_private" => {
                            let monitor: State<'_, ClipboardMonitor> = app.state();
                            monitor.private_mode.store(true, Ordering::Relaxed);
                            if let Ok(mut last) = monitor.last_content.lock() {
                                *last = None;
                            }
                            let config: State<'_, Mutex<Config>> = app.state();
                            if let Ok(mut cfg) = config.lock() {
                                cfg.private_mode_locked = true;
                                let _ = config::save_config(&cfg);
                            }
                            let _ = app.emit("private-mode-locked", false);
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            let hotkey_str = {
                let cfg = app.state::<Mutex<Config>>();
                let guard = cfg.lock().map_err(|e| e.to_string())?;
                guard.hotkey.toggle_window.clone()
            };

            if hotkey::is_wayland() {
                eprintln!(
                    "Ctrl+C: Wayland detected. Global shortcuts require xdg-desktop-portal \
                     or manual DE keybind configuration. Set a custom keybind in your \
                     desktop environment to run 'ctrl-c toggle' or similar."
                );
            } else {
                match hotkey::parse_hotkey(&hotkey_str) {
                    Ok(shortcut) => {
                        if let Err(e) = app.global_shortcut().register(shortcut) {
                            eprintln!("Ctrl+C: Failed to register global hotkey: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Ctrl+C: Invalid hotkey config '{}': {}", hotkey_str, e);
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_entry,
            get_entries,
            delete_entry,
            toggle_pin,
            get_config,
            save_config,
            get_private_mode_status,
            set_private_mode_password,
            lock_private_mode,
            unlock_private_mode,
            enable_autostart,
            disable_autostart,
            is_autostart_enabled,
            copy_and_paste,
            copy_to_clipboard,
            check_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
