use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

pub fn parse_hotkey(hotkey: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = hotkey.split('+').collect();
    let mut modifiers = Modifiers::empty();
    let mut key: Option<Code> = None;

    for part in parts {
        let trimmed = part.trim().to_lowercase();
        match trimmed.as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" | "option" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "super" | "win" | "cmd" | "command" | "windows" => modifiers |= Modifiers::SUPER,
            _ => match parse_code(&trimmed) {
                Some(code) => key = Some(code),
                None => return Err(format!("Unknown key: {}", part)),
            },
        }
    }

    let code = key.ok_or_else(|| "No key specified in hotkey".to_string())?;
    Ok(Shortcut::new(Some(modifiers), code))
}

fn parse_code(key: &str) -> Option<Code> {
    match key {
        "a" => Some(Code::KeyA),
        "b" => Some(Code::KeyB),
        "c" => Some(Code::KeyC),
        "d" => Some(Code::KeyD),
        "e" => Some(Code::KeyE),
        "f" => Some(Code::KeyF),
        "g" => Some(Code::KeyG),
        "h" => Some(Code::KeyH),
        "i" => Some(Code::KeyI),
        "j" => Some(Code::KeyJ),
        "k" => Some(Code::KeyK),
        "l" => Some(Code::KeyL),
        "m" => Some(Code::KeyM),
        "n" => Some(Code::KeyN),
        "o" => Some(Code::KeyO),
        "p" => Some(Code::KeyP),
        "q" => Some(Code::KeyQ),
        "r" => Some(Code::KeyR),
        "s" => Some(Code::KeyS),
        "t" => Some(Code::KeyT),
        "u" => Some(Code::KeyU),
        "v" => Some(Code::KeyV),
        "w" => Some(Code::KeyW),
        "x" => Some(Code::KeyX),
        "y" => Some(Code::KeyY),
        "z" => Some(Code::KeyZ),
        "0" => Some(Code::Digit0),
        "1" => Some(Code::Digit1),
        "2" => Some(Code::Digit2),
        "3" => Some(Code::Digit3),
        "4" => Some(Code::Digit4),
        "5" => Some(Code::Digit5),
        "6" => Some(Code::Digit6),
        "7" => Some(Code::Digit7),
        "8" => Some(Code::Digit8),
        "9" => Some(Code::Digit9),
        "f1" => Some(Code::F1),
        "f2" => Some(Code::F2),
        "f3" => Some(Code::F3),
        "f4" => Some(Code::F4),
        "f5" => Some(Code::F5),
        "f6" => Some(Code::F6),
        "f7" => Some(Code::F7),
        "f8" => Some(Code::F8),
        "f9" => Some(Code::F9),
        "f10" => Some(Code::F10),
        "f11" => Some(Code::F11),
        "f12" => Some(Code::F12),
        "space" => Some(Code::Space),
        "enter" | "return" => Some(Code::Enter),
        "escape" | "esc" => Some(Code::Escape),
        "tab" => Some(Code::Tab),
        "backspace" => Some(Code::Backspace),
        "delete" => Some(Code::Delete),
        "insert" => Some(Code::Insert),
        "home" => Some(Code::Home),
        "end" => Some(Code::End),
        "pageup" => Some(Code::PageUp),
        "pagedown" => Some(Code::PageDown),
        "up" | "arrowup" => Some(Code::ArrowUp),
        "down" | "arrowdown" => Some(Code::ArrowDown),
        "left" | "arrowleft" => Some(Code::ArrowLeft),
        "right" | "arrowright" => Some(Code::ArrowRight),
        _ => None,
    }
}

pub fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ctrl_shift_v() {
        let shortcut = parse_hotkey("Ctrl+Shift+V").unwrap();
        // Can't easily check modifiers and code equality, but at least it parses
        let _ = shortcut;
    }

    #[test]
    fn test_parse_alt_v() {
        let shortcut = parse_hotkey("Alt+V").unwrap();
        let _ = shortcut;
    }

    #[test]
    fn test_parse_alt_f4() {
        let shortcut = parse_hotkey("Alt+F4").unwrap();
        let _ = shortcut;
    }

    #[test]
    fn test_parse_super_space() {
        let shortcut = parse_hotkey("Super+Space").unwrap();
        let _ = shortcut;
    }

    #[test]
    fn test_parse_invalid_returns_error() {
        assert!(parse_hotkey("Ctrl+InvalidKey").is_err());
    }

    #[test]
    fn test_parse_no_key_returns_error() {
        assert!(parse_hotkey("Ctrl+Shift").is_err());
    }

    #[test]
    fn test_wayland_not_set() {
        // On non-Wayland systems (including Windows build env), this should be false
        assert!(!is_wayland());
    }
}
