mod app_name;
mod autostart;
mod clipboard;
mod config;
mod database;
mod hotkey;
mod paste;
mod private_mode;
mod wayland_focus;

use clipboard::{hash_bytes, ClipboardMonitor};
use config::Config;
use database::{Database, Entry};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

// ---------------------------------------------------------------------------
// GTK direct hide (Linux) -- Tauri v2's WebviewWindow::hide() is broken on
// Wayland: it returns Ok() without actually unmapping the window.
//
// Strategy:
//   1. During setup (main thread), cache the raw GtkWidget pointer.
//   2. All hide/show/visibility calls use raw FFI from ANY thread --
//      bypassing the gtk-rs thread check that panics on non-main threads.
//   3. For paste, unmap with gtk_widget_hide (gtk_window_iconify is a silent
//      no-op on GNOME Wayland -- the ICONIFIED WM bit never appears) and
//      verify against the real WM state (WITHDRAWN | ICONIFIED), not the
//      GTK-internal visible flag.
//   4. Focus is managed per session (see wayland_focus.rs): with the
//      window-calls Shell extension we capture/restore/verify the target
//      window before injecting the paste keystroke; without it the picker
//      never takes focus, so the paste goes to whatever already has focus.
// ---------------------------------------------------------------------------

/// Cached raw GTK widget pointer -- set once on the main thread during setup.
#[cfg(target_os = "linux")]
static CACHED_GTK_WIDGET: std::sync::atomic::AtomicPtr<gtk::ffi::GtkWidget> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

/// Find the Ctrl+C GTK window by iterating all toplevel windows.
/// MUST be called from the main GTK thread.
#[cfg(target_os = "linux")]
fn find_our_gtk_window() -> Option<gtk::Window> {
    use gtk::prelude::*;
    for toplevel in gtk::Window::list_toplevels() {
        if let Ok(win) = toplevel.dynamic_cast::<gtk::Window>() {
            if win.title().as_deref() == Some("Ctrl+C") {
                return Some(win);
            }
        }
    }
    None
}

/// Cache the raw GTK widget pointer. Call once during app setup (main thread).
#[cfg(target_os = "linux")]
fn cache_our_gtk_widget() {
    use gtk::prelude::*;
    if let Some(gtk_win) = find_our_gtk_window() {
        let ptr = gtk_win.as_ptr() as *mut gtk::ffi::GtkWidget;
        CACHED_GTK_WIDGET.store(ptr, std::sync::atomic::Ordering::Relaxed);
        eprintln!("[GTK] Cached raw widget pointer for Ctrl+C window");
    } else {
        eprintln!("[GTK] WARNING: Could not find Ctrl+C GTK window during setup");
    }
}

/// Get the cached raw widget pointer.
#[cfg(target_os = "linux")]
fn get_widget_ptr() -> *mut gtk::ffi::GtkWidget {
    CACHED_GTK_WIDGET.load(std::sync::atomic::Ordering::Relaxed)
}

// --- tauri::Window variants (command handlers, blur) ---

#[cfg(target_os = "linux")]
fn force_hide(window: &tauri::Window) {
    let ptr = get_widget_ptr();
    if !ptr.is_null() {
        unsafe { gtk::ffi::gtk_widget_hide(ptr); }
    } else {
        let _ = window.hide();
    }
}

#[cfg(not(target_os = "linux"))]
fn force_hide(window: &tauri::Window) {
    let _ = window.hide();
}

/// Read the raw GDK window state of the cached Ctrl+C window (Linux).
/// Returns the GDK_WINDOW_STATE_* bitmask, or None if no window is cached.
#[cfg(target_os = "linux")]
fn gdk_window_state_raw() -> Option<u32> {
    let ptr = get_widget_ptr();
    if ptr.is_null() {
        return None;
    }
    unsafe {
        let gdk_win = gtk::ffi::gtk_widget_get_window(ptr as *mut gtk::ffi::GtkWidget);
        if gdk_win.is_null() {
            return None;
        }
        Some(gtk::gdk::ffi::gdk_window_get_state(gdk_win as *mut gtk::gdk::ffi::GdkWindow))
    }
}

#[cfg(not(target_os = "linux"))]
fn gdk_window_state_raw() -> Option<u32> {
    None
}

/// Real WM-state check: is the window unmapped (WITHDRAWN) or iconified per the
/// compositor? `gtk_widget_get_visible()` is a GTK-internal flag and is NOT
/// flipped by `gtk_window_iconify()`, so it cannot be used to verify iconify.
#[cfg(target_os = "linux")]
fn force_wm_hidden(window: &tauri::Window) -> bool {
    match gdk_window_state_raw() {
        Some(state) => {
            state & (gtk::gdk::ffi::GDK_WINDOW_STATE_WITHDRAWN | gtk::gdk::ffi::GDK_WINDOW_STATE_ICONIFIED) != 0
        }
        None => !window.is_visible().unwrap_or(true),
    }
}

#[cfg(not(target_os = "linux"))]
fn force_wm_hidden(window: &tauri::Window) -> bool {
    !window.is_visible().unwrap_or(true)
}

/// Set the accept-focus / focus-on-map hints on the cached GTK window.
/// While the picker is open we want keyboard focus (search/arrow-nav/Enter);
/// the instant a selection is made we declare the window non-focus-taking so
/// dismissal doesn't fight the compositor's focus restoration.
#[cfg(target_os = "linux")]
fn set_accept_focus(accept: bool) {
    let ptr = get_widget_ptr();
    if ptr.is_null() {
        return;
    }
    let setting = accept as i32; // gboolean == c_int
    unsafe {
        let win = ptr as *mut gtk::ffi::GtkWindow;
        gtk::ffi::gtk_window_set_accept_focus(win, setting);
        gtk::ffi::gtk_window_set_focus_on_map(win, setting);
    }
}

#[cfg(not(target_os = "linux"))]
fn set_accept_focus(_accept: bool) {}

// --- WebviewWindow variants (tray/hotkey/socket handlers) ---

#[cfg(target_os = "linux")]
fn force_hide_wv(_window: &tauri::WebviewWindow) {
    let ptr = get_widget_ptr();
    if !ptr.is_null() {
        unsafe { gtk::ffi::gtk_widget_hide(ptr); }
    }
}

#[cfg(not(target_os = "linux"))]
fn force_hide_wv(window: &tauri::WebviewWindow) {
    let _ = window.hide();
}

#[cfg(target_os = "linux")]
fn force_is_visible_wv(_window: &tauri::WebviewWindow) -> bool {
    let ptr = get_widget_ptr();
    if !ptr.is_null() {
        unsafe { gtk::ffi::gtk_widget_get_visible(ptr) != 0 }
    } else {
        true
    }
}

#[cfg(not(target_os = "linux"))]
fn force_is_visible_wv(window: &tauri::WebviewWindow) -> bool {
    window.is_visible().unwrap_or(true)
}

// ---------------------------------------------------------------------------
// Focus management (Linux) -- save/restore the window that had focus before
// we showed our window, so we can return focus before simulating paste.
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
static SAVED_FOCUS: OnceLock<Mutex<Option<String>>> = OnceLock::new();

/// Save the currently focused window ID using xdotool.
/// Called before showing the Ctrl+C window so we can restore focus later.
#[cfg(target_os = "linux")]
fn save_focus() {
    let id = std::process::Command::new("xdotool")
        .args(["getactivewindow"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    if let Some(id) = id {
        let slot = SAVED_FOCUS.get_or_init(|| Mutex::new(None));
        if let Ok(mut guard) = slot.lock() {
            *guard = Some(id);
        }
    }
}

/// Restore focus to the previously saved window ID using xdotool.
/// Returns Ok(()) if activation was sent, Err if no window was saved or
/// xdotool failed.
#[cfg(target_os = "linux")]
fn restore_focus() -> Result<(), String> {
    let slot = SAVED_FOCUS.get_or_init(|| Mutex::new(None));
    let id = {
        let guard = slot.lock().map_err(|e| format!("focus lock: {}", e))?;
        guard.clone()
    };
    let id = id.ok_or("no saved window to restore")?;

    let status = std::process::Command::new("xdotool")
        .args(["windowactivate", "--sync", &id])
        .status()
        .map_err(|e| format!("xdotool windowactivate: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("xdotool windowactivate exited {}", status))
    }
}

#[cfg(target_os = "linux")]
fn get_socket_path() -> PathBuf {
    let base = dirs::runtime_dir()
        .unwrap_or_else(|| dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp")));
    let dir = base.join("ctrl-c");
    let _ = fs::create_dir_all(&dir);
    dir.join("ctrl-c.sock")
}

#[cfg(target_os = "linux")]
fn is_already_running() -> bool {
    std::os::unix::net::UnixStream::connect(&get_socket_path()).is_ok()
}

pub fn send_toggle() {
    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        let path = get_socket_path();
        if let Ok(stream) = std::os::unix::net::UnixStream::connect(&path) {
            let _ = (&stream).write_all(b"toggle");
        } else {
            eprintln!("Ctrl+C is not running");
            std::process::exit(1);
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("Toggle is only supported on Linux");
        std::process::exit(1);
    }
}

#[cfg(target_os = "linux")]
fn start_socket_listener() {
    use std::io::Read;
    use std::os::unix::net::UnixListener;

    std::thread::spawn(move || {
        let path = get_socket_path();
        let _ = fs::remove_file(&path);
        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Ctrl+C: Failed to bind socket: {}", e);
                return;
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    let mut buf = [0u8; 64];
                    if s.read(&mut buf).is_ok() {
                        let msg = String::from_utf8_lossy(&buf[..]);
                        if msg.starts_with("toggle") {
                            if let Some(handle) = APP_HANDLE.get() {
                                if let Some(window) = handle.get_webview_window("main") {
                                    if force_is_visible_wv(&window) {
                                        force_hide_wv(&window);
                                    } else {
                                        show_window(handle);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Ctrl+C: Socket accept error: {}", e);
                    break;
                }
            }
        }
    });
}

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
    source_app: Option<String>,
) -> Result<Vec<Entry>, String> {
    state.get_entries(query.as_deref(), date_filter.as_deref(), source_app.as_deref())
}

#[tauri::command]
fn get_app_names(state: State<'_, Arc<Database>>) -> Result<Vec<String>, String> {
    state.get_app_names()
}

/// Compute the UTC [start, end) timestamp bounds for a UI date-group label,
/// mirroring the frontend's `getGroupLabel()` bucketing exactly:
/// timestamps are stored UTC ('YYYY-MM-DD HH:MM:SS') and compared against
/// *local* midnight boundaries.
fn group_range_utc(label: &str) -> Result<(String, Option<String>), String> {
    use chrono::{Datelike, Duration, Local};

    let now = Local::now();
    let today_midnight = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let fmt = |d: chrono::NaiveDateTime| -> String {
        d.and_local_timezone(Local)
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    };

    let week_start = today_midnight
        - Duration::days(today_midnight.weekday().num_days_from_sunday() as i64);

    match label {
        "Today" => Ok((fmt(today_midnight), None)),
        "Yesterday" => Ok((fmt(today_midnight - Duration::days(1)), Some(fmt(today_midnight)))),
        "This Week" => Ok((fmt(week_start), Some(fmt(today_midnight)))),
        "Last Week" => Ok((fmt(week_start - Duration::days(7)), Some(fmt(week_start)))),
        "Older" => Ok(("0000-01-01 00:00:00".to_string(), Some(fmt(week_start - Duration::days(7))))),
        _ => Err(format!("Unknown group: {}", label)),
    }
}

#[tauri::command]
fn delete_group(state: State<'_, Arc<Database>>, label: String) -> Result<u32, String> {
    let (start, end) = group_range_utc(&label)?;
    state.delete_in_range(&start, end.as_deref())
}

/// Report how paste focus is managed on this session:
/// `"x11"` (xdotool focus restore), `"extension"` (window-calls extension),
/// or `"no-focus"` (no extension: picker never takes focus, mouse-only).
#[tauri::command]
fn get_focus_mode() -> String {
    #[cfg(target_os = "linux")]
    {
        if !hotkey::is_wayland() {
            return "x11".to_string();
        }
        match wayland_focus::mode() {
            wayland_focus::FocusMode::Extension => "extension".to_string(),
            wayland_focus::FocusMode::NoExtension => "no-focus".to_string(),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        "x11".to_string()
    }
}

#[tauri::command]
fn delete_entry(state: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    state.delete_entry(id)
}

#[tauri::command]
fn clear_all(state: State<'_, Arc<Database>>, monitor: State<'_, ClipboardMonitor>, keep_pinned: bool) -> Result<(), String> {
    state.clear_all(keep_pinned)?;
    if let Ok(mut clip) = arboard::Clipboard::new() {
        if let Ok(t) = clip.get_text() {
            if !t.trim().is_empty() {
                if let Ok(mut last) = monitor.last_content.lock() {
                    *last = Some(t);
                }
            }
        }
        if let Ok(img) = clip.get_image() {
            let w = img.width as u32;
            let h = img.height as u32;
            if w > 0 && h > 0 && w <= 3840 && h <= 2160 {
                let raw = img.bytes.to_vec();
                let expected = (w as usize) * (h as usize) * 4;
                if raw.len() == expected {
                    if let Ok(mut last) = monitor.last_image_hash.lock() {
                        *last = Some(hash_bytes(&raw));
                    }
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn toggle_pin(state: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    state.toggle_pin(id)
}

#[tauri::command]
fn update_entry(state: State<'_, Arc<Database>>, id: i64, content: String) -> Result<(), String> {
    state.update_entry(id, &content)
}

#[tauri::command]
fn set_entry_name(state: State<'_, Arc<Database>>, id: i64, name: String) -> Result<(), String> {
    state.set_entry_name(id, &name)
}

#[tauri::command]
fn get_config(state: State<'_, Mutex<Config>>) -> Result<Config, String> {
    state.lock().map_err(|e| e.to_string()).map(|c| c.clone())
}

#[tauri::command]
fn save_config(state: State<'_, Mutex<Config>>, db: State<'_, Arc<Database>>, config: Config) -> Result<(), String> {
    config::save_config(&config)?;
    let mut stored = state.lock().map_err(|e| e.to_string())?;
    *stored = config.clone();
    db.set_max_entries(config.behavior.max_entries as i64);
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
    if let Ok(mut last) = monitor.last_image_hash.lock() {
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
    if let Ok(mut last) = monitor.last_image_hash.lock() {
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

    // Sync last_content with current clipboard to prevent re-adding on next poll
    if let Ok(mut clip) = arboard::Clipboard::new() {
        if let Ok(text) = clip.get_text() {
            if let Ok(mut last) = monitor.last_content.lock() {
                *last = Some(text);
            }
        }
        if let Ok(img) = clip.get_image() {
            let w = img.width as u32;
            let h = img.height as u32;
            if w > 0 && h > 0 && w <= 3840 && h <= 2160 {
                let raw = img.bytes.to_vec();
                let expected = (w as usize) * (h as usize) * 4;
                if raw.len() == expected {
                    if let Ok(mut last) = monitor.last_image_hash.lock() {
                        *last = Some(hash_bytes(&raw));
                    }
                }
            }
        }
    }

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

fn simulate_paste(text: &str, key: paste::PasteKey) -> Result<(), String> {
    paste::simulate_paste(text, key)
}

fn get_foreground_app() -> String {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            use windows_sys::Win32::Foundation::CloseHandle;
            use windows_sys::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION};
            use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return String::new();
            }

            let mut pid: u32 = 0;
            let _ = GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == 0 {
                return String::new();
            }

            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if process.is_null() {
                return String::new();
            }

            let mut buf = [0u16; 260];
            let mut size = buf.len() as u32;
            let result = QueryFullProcessImageNameW(process, 0, buf.as_mut_ptr(), &mut size);
            let _ = CloseHandle(process);

            if result != 0 && size > 0 {
                let path = String::from_utf16_lossy(&buf[..size as usize]);
                if let Some(stem) = std::path::Path::new(&path).file_stem() {
                    let name = stem.to_string_lossy().to_string();
                    if !name.is_empty() {
                        return name;
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if hotkey::is_wayland() {
            // xdotool cannot see Wayland-native windows. Use the window-calls
            // GNOME extension (already used for focus management) to obtain the
            // focused window's pid + app-id, then resolve a friendly name.
            if wayland_focus::mode() == wayland_focus::FocusMode::Extension {
                if let Ok(target) = wayland_focus::focused_window() {
                    return app_name::resolve_linux_app_name(target.pid, &target.wm_class_instance);
                }
            }
            // NoExtension (non-GNOME / extension missing): no reliable capture.
            return String::new();
        }

        // X11: xdotool can read the active window's pid directly.
        if let Ok(out) = std::process::Command::new("xdotool")
            .args(["getactivewindow", "getwindowpid"])
            .output()
        {
            if out.status.success() {
                if let Ok(pid) = String::from_utf8_lossy(&out.stdout).trim().parse::<u32>() {
                    let name = app_name::resolve_linux_app_name(pid, "");
                    if !name.is_empty() {
                        return name;
                    }
                }
            }
        }
    }

    String::new()
}

#[tauri::command]
fn copy_and_paste(
    text: String,
    monitor: State<'_, ClipboardMonitor>,
    window: tauri::Window,
) -> Result<(), String> {
    // Set dedup flags BEFORE writing to clipboard to prevent the polling interval
    // from detecting the new text as a new entry in the race window
    if let Ok(mut last) = monitor.last_content.lock() {
        *last = Some(text.clone());
    }
    if let Ok(mut app) = monitor.last_app_copy.lock() {
        *app = Some(text.clone());
    }
    let mut clip = arboard::Clipboard::new().map_err(|e| format!("clipboard.new FAILED: {}", e))?;
    clip.set_text(&text).map_err(|e| format!("clipboard.write FAILED: {}", e))?;
    drop(clip);
    hide_and_paste(&window, &text);
    Ok(())
}

fn show_window(app: &tauri::AppHandle) {
    #[cfg(target_os = "linux")]
    let mode = wayland_focus::mode();
    #[cfg(not(target_os = "linux"))]
    let mode = wayland_focus::FocusMode::NoExtension;

    #[cfg(target_os = "linux")]
    {
        if hotkey::is_wayland() {
            match mode {
                wayland_focus::FocusMode::Extension => {
                    // Extension present: the picker may take focus for keyboard
                    // nav; capture the target BEFORE it loses focus.
                    set_accept_focus(true);
                    wayland_focus::save_current_focus();
                }
                wayland_focus::FocusMode::NoExtension => {
                    // No extension: the picker must never take focus, so the
                    // target keeps it and paste goes to the right place.
                    set_accept_focus(false);
                }
            }
        } else {
            // X11: xdotool can read/write focus directly.
            save_focus();
            set_accept_focus(true);
        }
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        #[cfg(target_os = "linux")]
        let take_focus =
            !hotkey::is_wayland() || mode == wayland_focus::FocusMode::Extension;
        #[cfg(not(target_os = "linux"))]
        let take_focus = true;
        if take_focus {
            let _ = window.set_focus();
        }
        let _ = app.emit("hotkey-show", ());
    }
}

/// Pick the paste keystroke for the current target window. In window-calls
/// extension mode we know the target's WM_CLASS: terminals get Ctrl+Shift+V,
/// other GUI apps get Ctrl+V. Everywhere else the target is unknown and we
/// keep the universal Shift+Insert.
#[cfg(target_os = "linux")]
fn paste_key_for_target(is_wayland: bool) -> paste::PasteKey {
    if is_wayland && wayland_focus::mode() == wayland_focus::FocusMode::Extension {
        if let Some(t) = wayland_focus::target() {
            if paste::looks_like_terminal(&t.wm_class)
                || paste::looks_like_terminal(&t.wm_class_instance)
            {
                return paste::PasteKey::CtrlShiftV;
            }
            return paste::PasteKey::CtrlV;
        }
    }
    paste::PasteKey::ShiftInsert
}

#[cfg(not(target_os = "linux"))]
fn paste_key_for_target(_is_wayland: bool) -> paste::PasteKey {
    paste::PasteKey::CtrlV
}

fn hide_and_paste(window: &tauri::Window, text: &str) {
    #[cfg(target_os = "linux")]
    let is_wayland = hotkey::is_wayland();
    #[cfg(not(target_os = "linux"))]
    let is_wayland = false;

    // Decide the keystroke before hiding, while the target is still known.
    let key = paste_key_for_target(is_wayland);

    // Dynamic accept-focus: while the picker is open we keep keyboard focus so
    // search/arrow-nav/Enter work. The instant a selection is made we disable
    // accept-focus so the window is not a focus candidate during dismissal --
    // this declares to Mutter that focus must go elsewhere once we unmap.
    if is_wayland {
        set_accept_focus(false);
    }

    // Unmap with gtk_widget_hide. On this setup gtk_window_iconify is a silent
    // no-op (verified via the WM-state poll: the ICONIFIED bit never appears),
    // while hide() properly transitions the surface to WITHDRAWN. Verification
    // below uses the REAL WM state, not gtk_widget_get_visible().
    force_hide(window);

    // Poll real WM state -- wait up to 400ms for the surface to unmap
    // (GDK_WINDOW_STATE_WITHDRAWN / ICONIFIED).
    for _ in 1..=20 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        if force_wm_hidden(window) {
            break;
        }
    }

    #[cfg(target_os = "linux")]
    {
        if is_wayland {
            match wayland_focus::mode() {
                wayland_focus::FocusMode::Extension => {
                    // Restore focus to the captured target via window-calls,
                    // then wait for it to actually take focus before pasting.
                    if let Ok(t) = wayland_focus::restore_focus() {
                        for _ in 0..20 {
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            if wayland_focus::verify_focus(t.id) {
                                break;
                            }
                        }
                    }
                }
                wayland_focus::FocusMode::NoExtension => {
                    // The picker never took focus, so the target still has it.
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        } else {
            let _ = restore_focus();
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
    }

    if let Err(msg) = simulate_paste(text, key) {
        let _ = window.emit("paste-error", &msg);
    }
}

fn rgba_to_png_thumbnail(raw_rgba: &[u8], w: u32, h: u32, max_w: u32, max_h: u32) -> Result<Vec<u8>, String> {
    let img = image::RgbaImage::from_raw(w, h, raw_rgba.to_vec())
        .ok_or_else(|| "Failed to create image from RGBA data".to_string())?;
    let (new_w, new_h) = if w > max_w || h > max_h {
        let ratio = ((w as f64 / max_w as f64).max(h as f64 / max_h as f64)).ceil() as u32;
        if ratio == 0 { (w, h) } else { (w / ratio, h / ratio) }
    } else {
        (w, h)
    };
    let dyn_img = image::DynamicImage::from(img);
    let thumb = dyn_img.thumbnail(new_w, new_h);
    let mut buf = Cursor::new(Vec::new());
    thumb.write_to(&mut buf, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}

fn base64_data_uri(bytes: &[u8]) -> String {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    format!("data:image/png;base64,{}", b64)
}

#[tauri::command]
fn get_entry_image(db: State<'_, Arc<Database>>, id: i64) -> Result<Option<String>, String> {
    let data = db.get_entry_image_data(id)?;
    match data {
        Some((raw_rgba, w, h)) => {
            let thumb = rgba_to_png_thumbnail(&raw_rgba, w, h, 300, 200)?;
            Ok(Some(base64_data_uri(&thumb)))
        }
        None => Ok(None),
    }
}

#[tauri::command]
fn copy_image_and_paste(
    db: State<'_, Arc<Database>>,
    monitor: State<'_, ClipboardMonitor>,
    window: tauri::Window,
    id: i64,
) -> Result<(), String> {
    let data = db.get_entry_image_data(id)?.ok_or("Image not found")?;
    let (raw_rgba, w, h) = data;

    // Validate buffer size before passing to arboard
    let expected_len = (w as usize) * (h as usize) * 4;
    if raw_rgba.len() != expected_len || w == 0 || h == 0 {
        return Err(format!("Invalid image data: {}x{} buffer {} (expected {})", w, h, raw_rgba.len(), expected_len));
    }

    let hash = hash_bytes(&raw_rgba);

    if let Ok(mut last) = monitor.last_image_hash.lock() {
        *last = Some(hash);
    }
    if let Ok(mut last) = monitor.last_app_copy.lock() {
        *last = Some("__image__".to_string());
    }

    let mut clip = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let img_data = arboard::ImageData {
        width: w as usize,
        height: h as usize,
        bytes: std::borrow::Cow::Owned(raw_rgba),
    };
    clip.set_image(img_data).map_err(|e| e.to_string())?;
    drop(clip);
    hide_and_paste(&window, "");
    Ok(())
}

#[tauri::command]
fn register_hotkey(
    app_handle: tauri::AppHandle,
    hotkey_str: String,
) -> Result<(), String> {
    if hotkey::is_wayland() {
        let exe = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default();
        return Err(format!(
            "Wayland does not support global shortcuts via this method.\n\
             Add a custom keybind in your DE settings to run:\n  {} toggle",
            exe
        ));
    }
    let shortcut = hotkey::parse_hotkey(&hotkey_str)?;
    app_handle.global_shortcut().register(shortcut).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn copy_to_clipboard(text: String, monitor: State<'_, ClipboardMonitor>) -> Result<(), String> {
    if let Ok(mut last) = monitor.last_content.lock() {
        *last = Some(text.clone());
    }
    if let Ok(mut last) = monitor.last_app_copy.lock() {
        *last = Some(text.clone());
    }
    let mut clip = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clip.set_text(&text).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_ignore_blur(monitor: State<'_, ClipboardMonitor>, ignore: bool) -> Result<(), String> {
    monitor.ignore_blur.store(ignore, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
fn set_monitoring(monitor: State<'_, ClipboardMonitor>, active: bool) -> Result<(), String> {
    let was_paused = monitor.paused.swap(!active, Ordering::Relaxed);
    if active && was_paused {
        if let Ok(mut clip) = arboard::Clipboard::new() {
            if let Ok(t) = clip.get_text() {
                if !t.trim().is_empty() {
                    if let Ok(mut last) = monitor.last_content.lock() {
                        *last = Some(t);
                    }
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn check_clipboard(
    db: State<'_, Arc<Database>>,
    monitor: State<'_, ClipboardMonitor>,
) -> Result<Option<Entry>, String> {
    if monitor.private_mode.load(Ordering::Relaxed) || monitor.paused.load(Ordering::Relaxed) {
        return Ok(None);
    }

    let mut clip = match arboard::Clipboard::new() {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };

    // Check for text first (fast path, stays on main thread)
    if let Ok(text) = clip.get_text() {
        if !text.trim().is_empty() {
            {
                let mut last = monitor.last_content.lock().map_err(|e| e.to_string())?;
                if last.as_ref() == Some(&text) {
                    return Ok(None);
                }
                *last = Some(text.clone());
            }
            {
                let mut app = monitor.last_app_copy.lock().map_err(|e| e.to_string())?;
                if app.as_ref() == Some(&text) || app.as_deref() == Some("__image__") {
                    *app = None;
                    return Ok(None);
                }
            }
            let app = get_foreground_app();
            match db.add_entry_with_app(&text, false, &app) {
                Ok(entry) => return Ok(Some(entry)),
                Err(ref e) if e == "duplicate entry" => return Ok(None),
                Err(e) => return Err(e),
            }
        }
    }

    // Check for image
    if let Ok(img) = clip.get_image() {
        let w = img.width as u32;
        let h = img.height as u32;

        // Skip images larger than 4K to avoid memory/performance issues
        if w > 3840 || h > 2160 {
            return Ok(None);
        }

        // Skip zero-size images
        if w == 0 || h == 0 {
            return Ok(None);
        }

        let raw_bytes = img.bytes.to_vec();

        // Validate buffer size before hashing/storing
        let expected_len = (w as usize) * (h as usize) * 4;
        if raw_bytes.len() != expected_len {
            return Err(format!("Clipboard image has unexpected buffer size: {} (expected {} for {}x{})", raw_bytes.len(), expected_len, w, h));
        }

        let hash = hash_bytes(&raw_bytes);

        {
            let mut last = monitor.last_image_hash.lock().map_err(|e| e.to_string())?;
            if last.as_ref() == Some(&hash) {
                return Ok(None);
            }
            *last = Some(hash);
        }
        {
            let mut app = monitor.last_app_copy.lock().map_err(|e| e.to_string())?;
            if app.as_deref() == Some("__image__") {
                *app = None;
                return Ok(None);
            }
        }

        let app = get_foreground_app();
        match db.add_image_entry_with_app(&raw_bytes, w, h, false, &app) {
            Ok(entry) => return Ok(Some(entry)),
            Err(ref e) if e == "duplicate entry" => return Ok(None),
            Err(e) => return Err(e),
        }
    }

    Ok(None)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = get_db_path();
    let db_path_str = db_path.to_string_lossy().to_string();
    let db = Arc::new(Database::new(&db_path_str).expect("Failed to initialize database"));
    let loaded_config = config::load_config();
    db.set_max_entries(loaded_config.behavior.max_entries as i64);
    let is_locked = loaded_config.private_mode_locked;
    let monitor = ClipboardMonitor::new();

    if is_locked {
        monitor.private_mode.store(true, Ordering::Relaxed);
    } else if let Ok(mut clip) = arboard::Clipboard::new() {
        if let Ok(t) = clip.get_text() {
            if !t.trim().is_empty() {
                if let Ok(mut last) = monitor.last_content.lock() {
                    *last = Some(t);
                }
            }
        }
        if let Ok(img) = clip.get_image() {
            let w = img.width as u32;
            let h = img.height as u32;
            if w > 0 && h > 0 && w <= 3840 && h <= 2160 {
                let raw = img.bytes.to_vec();
                let expected = (w as usize) * (h as usize) * 4;
                if raw.len() == expected {
                    if let Ok(mut last) = monitor.last_image_hash.lock() {
                        *last = Some(hash_bytes(&raw));
                    }
                }
            }
        }
    }

    if loaded_config.autostart {
        if let Err(e) = autostart::enable_autostart() {
            eprintln!("Ctrl+C: Failed to enable autostart: {}", e);
        }
    }

    let config = Mutex::new(loaded_config);

    #[cfg(target_os = "linux")]
    if is_already_running() {
        eprintln!("Ctrl+C is already running");
        std::process::exit(0);
    }

    #[cfg(target_os = "linux")]
    start_socket_listener();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            if force_is_visible_wv(&window) {
                                force_hide_wv(&window);
                            } else {
                                show_window(app);
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
            let _ = APP_HANDLE.set(app.handle().clone());

            #[cfg(target_os = "linux")]
            {
                cache_our_gtk_widget();
                let focus_mode = wayland_focus::init();
                if hotkey::is_wayland() && focus_mode == wayland_focus::FocusMode::NoExtension {
                    // Picker must never take focus on no-extension Wayland.
                    set_accept_focus(false);
                }
            }

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
                .tooltip("Ctrl+C -- Clipboard Manager")
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
                            if force_is_visible_wv(&window) {
                                force_hide_wv(&window);
                            } else {
                                show_window(app);
                            }
                        }
                    }
                })
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show_hide" => {
                            if let Some(window) = app.get_webview_window("main") {
                                if force_is_visible_wv(&window) {
                                    force_hide_wv(&window);
                                } else {
                                    show_window(app);
                                }
                            }
                        }
                        "lock_private" => {
                            let monitor: State<'_, ClipboardMonitor> = app.state();
                            monitor.private_mode.store(true, Ordering::Relaxed);
                            if let Ok(mut last) = monitor.last_content.lock() {
                                *last = None;
                            }
                            if let Ok(mut last) = monitor.last_image_hash.lock() {
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
                let exe_path = std::env::current_exe().unwrap_or_default();
                let toggle_cmd = format!("{} toggle", exe_path.display());
                eprintln!(
                    "Ctrl+C: Wayland detected. Global shortcuts require xdg-desktop-portal \
                     or manual DE keybind configuration. To use a custom keybind, set your \
                     DE shortcut to run '{}'.",
                    toggle_cmd
                );
                let _ = app.emit("wayland-hotkey-info", &toggle_cmd);
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
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(false) = event {
                let monitor: State<'_, ClipboardMonitor> = window.state();
                if !monitor.ignore_blur.load(Ordering::Relaxed) {
                    force_hide(window);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            add_entry,
            get_entries,
            delete_entry,
            delete_group,
            clear_all,
            toggle_pin,
            update_entry,
            set_entry_name,
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
            copy_image_and_paste,
            get_entry_image,
            check_clipboard,
            set_monitoring,
            set_ignore_blur,
            register_hotkey,
            get_app_names,
            get_focus_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
