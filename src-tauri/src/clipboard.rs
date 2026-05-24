use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;

pub struct ClipboardMonitor {
    pub private_mode: Arc<AtomicBool>,
    pub last_content: Arc<Mutex<Option<String>>>,
}

impl ClipboardMonitor {
    pub fn new() -> Self {
        ClipboardMonitor {
            private_mode: Arc::new(AtomicBool::new(false)),
            last_content: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for ClipboardMonitor {
    fn default() -> Self {
        Self::new()
    }
}
