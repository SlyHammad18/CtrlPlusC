//! Wayland window-focus capture/restore via the `window-calls` GNOME Shell
//! extension (https://extensions.gnome.org/extension/4724/window-calls/).
//!
//! On GNOME Wayland there is no client-accessible way to read or set focus:
//! `org.gnome.Shell.Eval` is disabled since GNOME 41 (returns `(false, '')`
//! for every expression), `org.gnome.Shell.Introspect` is allowlisted to the
//! portal, and `xdotool` only sees XWayland windows. The `window-calls`
//! extension runs inside gnome-shell and exports a D-Bus API that works for
//! Wayland-native windows:
//!
//! ```text
//!   List()           -> s   JSON array; every window has `id`, `wm_class`,
//!                          `wm_class_instance`, `title`, `pid`, `focus`,
//!                          `in_current_workspace`, ...
//!   GetTitle(winid)  -> s   plain title string
//!   Activate(winid)  -> ()  raise + focus the window
//! ```
//!
//! Two operational modes:
//!
//! * `FocusMode::Extension` -- the extension is present. We capture the focused
//!   window *before* showing the picker, reactivate it after dismissal, and
//!   verify focus before injecting the paste keystroke. Keyboard navigation in
//!   the picker keeps working because the picker may take focus.
//!
//! * `FocusMode::NoExtension` -- no extension. The picker must *never* take
//!   focus (`accept_focus = false`), so the paste keystroke goes to whatever
//!   already has focus -- correct by construction (same reasoning as
//!   swaywm/sway#6368). The picker degrades to mouse-only selection.

use std::process::{Command, Output};
use std::sync::{Mutex, OnceLock};

const DEST: &str = "org.gnome.Shell";
const OBJ: &str = "/org/gnome/Shell/Extensions/Windows";
const IFACE: &str = "org.gnome.Shell.Extensions.Windows";

/// How focus is managed on this machine (Wayland only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusMode {
    /// window-calls extension present: capture / reactivate / verify.
    Extension,
    /// No extension: picker must never take focus (mouse-only).
    NoExtension,
}

static FOCUS_MODE: OnceLock<FocusMode> = OnceLock::new();

/// A captured Wayland-native window (the paste target).
#[derive(Debug, Clone)]
pub struct WaylandTarget {
    pub id: u32,
    pub pid: u32,
    pub wm_class: String,
    pub wm_class_instance: String,
}

static SAVED_TARGET: OnceLock<Mutex<Option<WaylandTarget>>> = OnceLock::new();

/// Run a window-calls D-Bus method via `dbus-send --print-reply=literal`.
/// Literal mode prints the returned string raw (no GVariant quoting), so the
/// JSON payloads can be parsed directly. Errors (e.g. `UnknownMethod` when the
/// extension is not loaded) come back as a non-zero exit + stderr message.
fn dbus_send(method: &str, args: &[&str]) -> Result<Output, String> {
    let mut cmd = Command::new("dbus-send");
    cmd.args(["--session", "--print-reply=literal", &format!("--dest={}", DEST)]);
    cmd.arg(OBJ);
    cmd.arg(format!("{}.{}", IFACE, method));
    cmd.args(args);
    let out = cmd.output().map_err(|e| format!("dbus-send exec: {}", e))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(out)
}

/// Parse a D-Bus literal reply as JSON. Tolerates leading whitespace/newlines
/// that dbus-send prepends to string replies.
fn parse_json(text: &str) -> Result<serde_json::Value, String> {
    let t = text.trim();
    if let Ok(v) = serde_json::from_str(t) {
        return Ok(v);
    }
    let start = t.find('[').or_else(|| t.find('{'));
    if let Some(i) = start {
        if let Ok(v) = serde_json::from_str(&t[i..]) {
            return Ok(v);
        }
    }
    let preview: String = t.chars().take(120).collect();
    Err(format!("cannot parse D-Bus reply as JSON: {:?}", preview))
}

/// Determine which focus mode this session is in. Call once at startup.
pub fn init() -> FocusMode {
    let mode = probe();
    let _ = FOCUS_MODE.set(mode);
    mode
}

/// Cached focus mode (probes on first access if `init` was never called).
pub fn mode() -> FocusMode {
    *FOCUS_MODE.get_or_init(probe)
}

/// The D-Bus node path exists on every recent GNOME Shell even without the
/// extension (the shell owns the object tree), so only a real method call
/// tells us whether window-calls is exporting its interface.
fn probe() -> FocusMode {
    match dbus_send("List", &[]) {
        Ok(_) => FocusMode::Extension,
        Err(_) => FocusMode::NoExtension,
    }
}

/// Ask window-calls which window currently has focus.
pub fn focused_window() -> Result<WaylandTarget, String> {
    let out = dbus_send("List", &[])?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let root = parse_json(&stdout)?;
    let arr = root.as_array().ok_or("window-calls List reply is not a JSON array")?;
    for w in arr {
        if w.get("focus").and_then(|f| f.as_bool()).unwrap_or(false) {
            let id = w.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            if id == 0 {
                continue;
            }
            let title = w.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let _ = title;
            let pid = w.get("pid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let wm_class = w.get("wm_class").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let wm_class_instance = w
                .get("wm_class_instance")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            return Ok(WaylandTarget {
                id,
                pid,
                wm_class,
                wm_class_instance,
            });
        }
    }
    Err("window-calls: no focused window in List()".to_string())
}

/// Capture the currently focused window and remember it for later restoration.
/// Call right before showing the picker, while the target still has focus.
pub fn save_current_focus() -> Option<WaylandTarget> {
    match focused_window() {
        Ok(t) => {
            let slot = SAVED_TARGET.get_or_init(|| Mutex::new(None));
            if let Ok(mut g) = slot.lock() {
                *g = Some(t.clone());
            }
            Some(t)
        }
        Err(_) => None,
    }
}

/// The last captured target, if any.
pub fn target() -> Option<WaylandTarget> {
    SAVED_TARGET
        .get_or_init(|| Mutex::new(None))
        .lock()
        .ok()
        .and_then(|g| g.clone())
}

/// Reactivate the saved target (raise + focus) via window-calls `Activate`.
pub fn restore_focus() -> Result<WaylandTarget, String> {
    let t = target().ok_or("no saved wayland target to restore")?;
    match dbus_send("Activate", &[&format!("uint32:{}", t.id)]) {
        Ok(_) => Ok(t),
        Err(e) => Err(format!("window-calls Activate failed: {}", e)),
    }
}

/// Ask window-calls whether `id` is currently the focused window.
pub fn verify_focus(id: u32) -> bool {
    let out = match dbus_send("List", &[]) {
        Ok(o) => o,
        Err(_) => return false,
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    let root = match parse_json(&stdout) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let Some(arr) = root.as_array() else {
        return false;
    };
    arr.iter().any(|w| {
        w.get("id").and_then(|v| v.as_u64()) == Some(id as u64)
            && w.get("focus").and_then(|f| f.as_bool()) == Some(true)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_handles_plain_json() {
        let v = parse_json(r#"[{"id": 123, "focus": true}]"#).unwrap();
        assert_eq!(v[0]["id"], 123);
        assert_eq!(v[0]["focus"], true);
    }

    #[test]
    fn parse_json_handles_dbussend_padding() {
        // dbus-send --print-reply=literal prefixes string replies with whitespace.
        let v = parse_json("\n  [{\"id\": 7, \"focus\": true, \"title\": \"a ] b\"}]").unwrap();
        assert_eq!(v[0]["id"], 7);
        assert_eq!(v[0]["title"], "a ] b");
    }

    #[test]
    fn parse_json_rejects_garbage() {
        assert!(parse_json("Error org.freedesktop.DBus.Error.UnknownMethod").is_err());
    }

    #[test]
    fn focused_window_selection_picks_focused_entry() {
        let payload = r#"[{"id":1,"wm_class":"A","focus":false},
                           {"id":2,"wm_class":"B","focus":true,"title":"Term","wm_class_instance":"b"},
                           {"id":3,"wm_class":"C","focus":false}]"#;
        let root = parse_json(payload).unwrap();
        let arr = root.as_array().unwrap();
        let w = arr
            .iter()
            .find(|w| w["focus"].as_bool() == Some(true))
            .unwrap();
        assert_eq!(w["id"], 2);
        assert_eq!(w["wm_class"], "B");
        assert_eq!(w["title"], "Term");
    }
}