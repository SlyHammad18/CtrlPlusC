use std::hash::{Hash, Hasher};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;

pub struct ClipboardMonitor {
    pub private_mode: Arc<AtomicBool>,
    pub paused: Arc<AtomicBool>,
    pub last_content: Arc<Mutex<Option<String>>>,
    pub last_app_copy: Arc<Mutex<Option<String>>>,
    pub last_image_hash: Arc<Mutex<Option<u64>>>,
}

impl ClipboardMonitor {
    pub fn new() -> Self {
        ClipboardMonitor {
            private_mode: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            last_content: Arc::new(Mutex::new(None)),
            last_app_copy: Arc::new(Mutex::new(None)),
            last_image_hash: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for ClipboardMonitor {
    fn default() -> Self {
        Self::new()
    }
}

pub fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}
