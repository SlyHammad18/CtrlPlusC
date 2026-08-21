//! Paste simulation for clipboard manager.
//!
//! Priority order on Linux:
//!   1. ydotool — uinput-based via ydotoold daemon. Works on GNOME Wayland
//!      where plain uinput virtual keyboards are silently dropped by Mutter.
//!   2. Native uinput — kernel-level input injection via /dev/uinput.
//!      Works on wlroots compositors (Sway, Hyprland). No external tools needed.
//!      User must be in the `input` group.
//!   3. wtype — virtual-keyboard protocol, wlroots compositors only.
//!   4. xdotool — X11 only; silently fails for native Wayland windows.
//!
//! On Windows, `windows-sys` keybd_event is used (defined in lib.rs).
//! On macOS/other, paste is a no-op.
//!
//! **Focus management** is handled by the caller (`hide_and_paste` in lib.rs):
//! focus is saved before the window is shown and restored after hiding, so the
//! paste keystroke lands in the correct window.

use std::sync::{Mutex, OnceLock};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Paste keystroke selection
// ---------------------------------------------------------------------------

/// Which paste keystroke to inject into the focused window.
///
/// * `CtrlV`       -- standard GUI paste (Ctrl+V).
/// * `CtrlShiftV`  -- standard terminal paste (Ctrl+Shift+V).
/// * `ShiftInsert` -- universal fallback; honored by most terminals *and* most
///                    GUI toolkits. Used when the target window is unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteKey {
    CtrlV,
    CtrlShiftV,
    ShiftInsert,
}

impl PasteKey {
    /// ydotool `key:N` press sequence (Linux input-event keycodes:
    /// 29 = LEFTCTRL, 42 = LEFTSHIFT, 47 = V, 110 = INSERT).
    fn ydotool(&self) -> &'static [&'static str] {
        match self {
            PasteKey::CtrlV => &["29:1", "47:1", "47:0", "29:0"],
            PasteKey::CtrlShiftV => &["29:1", "42:1", "47:1", "47:0", "42:0", "29:0"],
            PasteKey::ShiftInsert => &["42:1", "110:1", "110:0", "42:0"],
        }
    }

    /// `wtype` arguments (key must be `-M key` for a named key like Insert).
    fn wtype(&self) -> &'static [&'static str] {
        match self {
            PasteKey::CtrlV => &["-M", "ctrl", "v", "-m", "ctrl"],
            PasteKey::CtrlShiftV => &[
                "-M", "ctrl", "-M", "shift", "v", "-m", "ctrl", "-m", "shift",
            ],
            PasteKey::ShiftInsert => &["-M", "shift", "Insert", "-m", "shift"],
        }
    }

    /// `xdotool` keysym.
    fn xdotool(&self) -> &'static str {
        match self {
            PasteKey::CtrlV => "ctrl+v",
            PasteKey::CtrlShiftV => "ctrl+shift+v",
            PasteKey::ShiftInsert => "shift+Insert",
        }
    }
}

/// Best-effort terminal detection from a window's WM_CLASS. Terminals bind
/// Ctrl+Shift+V for paste instead of Ctrl+V, so the injected keystroke must
/// match the target application (only meaningful when the target is known,
/// i.e. window-calls extension mode).
pub fn looks_like_terminal(wm_class: &str) -> bool {
    let c = wm_class.to_ascii_lowercase();
    const TERMINALS: [&str; 16] = [
        "terminal", "kitty", "alacritty", "wezterm", "foot", "konsole", "xterm",
        "urxvt", "rxvt", "tilix", "terminator", "ghostty", "ptyxis", "kgx",
        "xfce4-terminal", "st-",
    ];
    TERMINALS.iter().any(|k| c.contains(k))
}

// ---------------------------------------------------------------------------
// Native uinput (Linux)
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
static VIRTUAL_KEYBOARD: OnceLock<Mutex<mouse_keyboard_input::VirtualDevice>> = OnceLock::new();

/// Lazily initialize the virtual keyboard. Returns an error message if
/// the device cannot be created (e.g. /dev/uinput not accessible).
#[cfg(target_os = "linux")]
fn ensure_virtual_keyboard() -> Result<(), String> {
    if VIRTUAL_KEYBOARD.get().is_some() {
        return Ok(());
    }

    let device = mouse_keyboard_input::VirtualDevice::default().map_err(|e| {
        format!(
            "Failed to create virtual keyboard: {}. \
             Ensure you are in the 'input' group:\n  \
             sudo usermod -aG input $USER && relogin",
            e
        )
    })?;

    // Give the kernel a moment to register the new input device.
    std::thread::sleep(Duration::from_millis(200));

    // Ignore error — another thread may have initialized concurrently.
    let _ = VIRTUAL_KEYBOARD.set(Mutex::new(device));
    Ok(())
}

/// Send the given paste keystroke via native /dev/uinput.
#[cfg(target_os = "linux")]
fn send_uinput(key: PasteKey, kb: &mut mouse_keyboard_input::VirtualDevice) -> Result<(), String> {
    use mouse_keyboard_input::key_codes::*;
    match key {
        PasteKey::CtrlV => {
            kb.press(KEY_LEFTCTRL)
                .map_err(|e| format!("uinput press Ctrl: {}", e))?;
            std::thread::sleep(Duration::from_millis(15));
            kb.click(KEY_V)
                .map_err(|e| format!("uinput click V: {}", e))?;
            std::thread::sleep(Duration::from_millis(15));
            kb.release(KEY_LEFTCTRL)
                .map_err(|e| format!("uinput release Ctrl: {}", e))?;
        }
        PasteKey::CtrlShiftV => {
            kb.press(KEY_LEFTCTRL)
                .map_err(|e| format!("uinput press Ctrl: {}", e))?;
            kb.press(KEY_LEFTSHIFT)
                .map_err(|e| format!("uinput press Shift: {}", e))?;
            std::thread::sleep(Duration::from_millis(15));
            kb.click(KEY_V)
                .map_err(|e| format!("uinput click V: {}", e))?;
            std::thread::sleep(Duration::from_millis(15));
            kb.release(KEY_LEFTSHIFT)
                .map_err(|e| format!("uinput release Shift: {}", e))?;
            kb.release(KEY_LEFTCTRL)
                .map_err(|e| format!("uinput release Ctrl: {}", e))?;
        }
        PasteKey::ShiftInsert => {
            kb.press(KEY_LEFTSHIFT)
                .map_err(|e| format!("uinput press Shift: {}", e))?;
            std::thread::sleep(Duration::from_millis(15));
            kb.click(KEY_INSERT)
                .map_err(|e| format!("uinput click Insert: {}", e))?;
            std::thread::sleep(Duration::from_millis(15));
            kb.release(KEY_LEFTSHIFT)
                .map_err(|e| format!("uinput release Shift: {}", e))?;
        }
    }
    Ok(())
}

/// Simulate a paste keystroke via native /dev/uinput.
#[cfg(target_os = "linux")]
fn simulate_paste_uinput(key: PasteKey) -> Result<(), String> {
    ensure_virtual_keyboard()?;

    let mut kb = VIRTUAL_KEYBOARD
        .get()
        .ok_or("uinput device not initialized")?
        .lock()
        .map_err(|e| format!("uinput lock: {}", e))?;

    send_uinput(key, &mut kb)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// External tool fallbacks (Linux)
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn try_ydotool(key: PasteKey) -> Result<(), String> {
    let socket_path = std::env::var("YDOTOOL_SOCKET").ok().filter(|p| std::path::Path::new(p).exists()).or_else(|| {
        std::env::var("XDG_RUNTIME_DIR").ok().map(|d| format!("{}/.ydotool_socket", d))
    }).filter(|p| std::path::Path::new(p).exists()).unwrap_or_else(|| "/tmp/.ydotool_socket".to_string());

    if !std::path::Path::new(&socket_path).exists() {
        return Err("ydotoold socket not found".into());
    }

    let mut args = vec!["key", "-d", "30"];
    args.extend_from_slice(key.ydotool());
    let output = std::process::Command::new("ydotool")
        .args(&args)
        .output()
        .map_err(|e| format!("ydotool exec failed: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("ydotool exited with {}", output.status))
    }
}

#[cfg(target_os = "linux")]
fn try_ydotool_type(text: &str) -> Result<(), String> {
    let output = std::process::Command::new("ydotool")
        .args(["type", "-d", "10", "--", text])
        .output()
        .map_err(|e| format!("ydotool type exec failed: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("ydotool type exited with {}", output.status))
    }
}

#[cfg(target_os = "linux")]
fn try_wtype(key: PasteKey) -> Result<(), String> {
    let output = std::process::Command::new("wtype")
        .args(key.wtype())
        .output()
        .map_err(|e| format!("wtype exec failed: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("wtype exited with {}", output.status))
    }
}

#[cfg(target_os = "linux")]
fn try_xdotool(key: PasteKey) -> Result<(), String> {
    let output = std::process::Command::new("xdotool")
        .args(["key", "--clearmodifiers", key.xdotool()])
        .output()
        .map_err(|e| format!("xdotool exec failed: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("xdotool exited with {}", output.status))
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Simulate a paste keystroke in the currently focused window.
///
/// Uses the best available method for the current platform and display server.
/// Returns `Ok(())` on success, or a user-facing error message on failure.
///
/// `text` is the clipboard content — used as a fallback via `ydotool type`
/// when key-combo injection fails (e.g. Mutter drops Ctrl+V/Shift+Insert).
/// `key` selects the exact keystroke; the caller picks it based on the target
/// window (e.g. Ctrl+Shift+V for terminals).
pub fn simulate_paste(text: &str, key: PasteKey) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        simulate_paste_windows();
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();

        if is_wayland {
            // 1. ydotool — same approach as GhostClip (works on GNOME Wayland)
            if try_ydotool(key).is_ok() {
                return Ok(());
            }

            // 2. ydotool type — bypass key combos entirely, type text directly
            if !text.is_empty() && try_ydotool_type(text).is_ok() {
                return Ok(());
            }

            // 3. Native uinput
            if simulate_paste_uinput(key).is_ok() {
                return Ok(());
            }

            // 4. wtype
            if try_wtype(key).is_ok() {
                return Ok(());
            }

            return Err(
                "All Wayland paste methods failed (ydotool, uinput, wtype)".to_string(),
            );
        }

        // X11: try xdotool first, then uinput (also works on X11).
        if try_xdotool(key).is_ok() || simulate_paste_uinput(key).is_ok() {
            return Ok(());
        }

        Err("Auto-paste failed. Install xdotool:\n  sudo apt install xdotool".to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = (text, key);
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn simulate_paste_windows() {
    unsafe {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
        const VK_CONTROL: u16 = 0x11;
        const VK_V: u16 = 0x56;
        keybd_event(VK_CONTROL as u8, 0, 0, 0);
        keybd_event(VK_V as u8, 0, 0, 0);
        std::thread::sleep(Duration::from_millis(15));
        keybd_event(VK_V as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_KEYUP, 0);
    }
}
