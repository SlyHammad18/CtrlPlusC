use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::Emitter;

pub struct ClipboardMonitor {
    pub private_mode: Arc<AtomicBool>,
}

impl ClipboardMonitor {
    pub fn new() -> Self {
        ClipboardMonitor {
            private_mode: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for ClipboardMonitor {
    fn default() -> Self {
        Self::new()
    }
}

pub fn start_monitoring(
    app_handle: tauri::AppHandle,
    db: Arc<crate::database::Database>,
    poll_interval_ms: u64,
    private_mode: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        let mut clipboard = match arboard::Clipboard::new() {
                Ok(c) => {
                    eprintln!("Clipboard monitor started (polling every {}ms)", poll_interval_ms);
                    Some(c)
                }
            Err(e) => {
                eprintln!("Failed to initialize clipboard: {}", e);
                None
            }
        };

        let mut last_content: Option<String> = None;

        loop {
            thread::sleep(Duration::from_millis(poll_interval_ms));

            if private_mode.load(Ordering::Relaxed) {
                last_content = None;
                continue;
            }

            if let Some(ref mut clip) = clipboard {
                match clip.get_text() {
                    Ok(text) => {
                        if last_content.as_ref() != Some(&text) {
                            last_content = Some(text.clone());
                            match db.add_entry(&text, false) {
                                Ok(entry) => {
                                    let _ = app_handle.emit("clipboard-changed", &entry);
                                }
                                Err(ref e) if e == "duplicate entry" => {}
                                Err(e) => {
                                    eprintln!("Failed to save clipboard entry: {}", e);
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            } else {
                clipboard = match arboard::Clipboard::new() {
                    Ok(c) => {
                        eprintln!("Clipboard re-initialized");
                        Some(c)
                    }
                    Err(e) => {
                        eprintln!("Failed to re-initialize clipboard: {}", e);
                        None
                    }
                };
            }
        }
    });
}
