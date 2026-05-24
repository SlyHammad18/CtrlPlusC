use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_bg_primary")]
    pub bg_primary: String,
    #[serde(default = "default_bg_secondary")]
    pub bg_secondary: String,
    #[serde(default = "default_bg_card")]
    pub bg_card: String,
    #[serde(default = "default_text_primary")]
    pub text_primary: String,
    #[serde(default = "default_text_secondary")]
    pub text_secondary: String,
    #[serde(default = "default_accent")]
    pub accent: String,
    #[serde(default = "default_accent_hover")]
    pub accent_hover: String,
    #[serde(default = "default_danger")]
    pub danger: String,
    #[serde(default = "default_success")]
    pub success: String,
    #[serde(default = "default_border")]
    pub border: String,
    #[serde(default = "default_border_radius")]
    pub border_radius: String,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        ThemeConfig {
            bg_primary: default_bg_primary(),
            bg_secondary: default_bg_secondary(),
            bg_card: default_bg_card(),
            text_primary: default_text_primary(),
            text_secondary: default_text_secondary(),
            accent: default_accent(),
            accent_hover: default_accent_hover(),
            danger: default_danger(),
            success: default_success(),
            border: default_border(),
            border_radius: default_border_radius(),
            font_family: default_font_family(),
            font_size: default_font_size(),
        }
    }
}

fn default_bg_primary() -> String { "#0F172A".to_string() }
fn default_bg_secondary() -> String { "#1E293B".to_string() }
fn default_bg_card() -> String { "#334155".to_string() }
fn default_text_primary() -> String { "#F8FAFC".to_string() }
fn default_text_secondary() -> String { "#94A3B8".to_string() }
fn default_accent() -> String { "#3B82F6".to_string() }
fn default_accent_hover() -> String { "#2563EB".to_string() }
fn default_danger() -> String { "#EF4444".to_string() }
fn default_success() -> String { "#22C55E".to_string() }
fn default_border() -> String { "#475569".to_string() }
fn default_border_radius() -> String { "12px".to_string() }
fn default_font_family() -> String { "Inter, system-ui, sans-serif".to_string() }
fn default_font_size() -> String { "14px".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_window_width")]
    pub width: u32,
    #[serde(default = "default_window_height")]
    pub height: u32,
    #[serde(default = "default_window_opacity")]
    pub opacity: f64,
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            width: default_window_width(),
            height: default_window_height(),
            opacity: default_window_opacity(),
        }
    }
}

fn default_window_width() -> u32 { 420 }
fn default_window_height() -> u32 { 600 }
fn default_window_opacity() -> f64 { 0.97 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    #[serde(default = "default_max_entries")]
    pub max_entries: u32,
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
    #[serde(default = "default_search_debounce_ms")]
    pub search_debounce_ms: u64,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        BehaviorConfig {
            max_entries: default_max_entries(),
            poll_interval_ms: default_poll_interval_ms(),
            search_debounce_ms: default_search_debounce_ms(),
        }
    }
}

fn default_max_entries() -> u32 { 100 }
fn default_poll_interval_ms() -> u64 { 500 }
fn default_search_debounce_ms() -> u64 { 300 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    #[serde(default = "default_toggle_window")]
    pub toggle_window: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        HotkeyConfig {
            toggle_window: default_toggle_window(),
        }
    }
}

fn default_toggle_window() -> String { "Ctrl+Shift+V".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub behavior: BehaviorConfig,
    #[serde(default)]
    pub hotkey: HotkeyConfig,
    #[serde(default = "default_autostart")]
    pub autostart: bool,
    #[serde(default)]
    pub private_mode_password_hash: String,
    #[serde(default)]
    pub private_mode_locked: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            theme: ThemeConfig::default(),
            window: WindowConfig::default(),
            behavior: BehaviorConfig::default(),
            hotkey: HotkeyConfig::default(),
            autostart: default_autostart(),
            private_mode_password_hash: String::new(),
            private_mode_locked: false,
        }
    }
}

fn default_autostart() -> bool { false }

pub fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("ctrl-c");
    path.push("config.toml");
    path
}

pub fn load_config() -> Config {
    let path = get_config_path();

    if !path.exists() {
        let config = Config::default();
        if let Err(e) = save_config(&config) {
            eprintln!("Failed to create default config: {}", e);
        }
        return config;
    }

    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to parse config (using defaults): {}", e);
                Config::default()
            }
        },
        Err(e) => {
            eprintln!("Failed to read config (using defaults): {}", e);
            Config::default()
        }
    }
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let path = get_config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let content = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.theme.bg_primary, "#0F172A");
        assert_eq!(config.window.width, 420);
        assert_eq!(config.behavior.max_entries, 100);
        assert_eq!(config.hotkey.toggle_window, "Ctrl+Shift+V");
        assert!(!config.autostart);
        assert!(config.private_mode_password_hash.is_empty());
    }

    #[test]
    fn test_config_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.theme.bg_primary, config.theme.bg_primary);
        assert_eq!(parsed.window.width, config.window.width);
        assert_eq!(parsed.behavior.poll_interval_ms, config.behavior.poll_interval_ms);
        assert_eq!(parsed.hotkey.toggle_window, config.hotkey.toggle_window);
    }

    #[test]
    fn test_config_path_is_absolute() {
        let path = get_config_path();
        assert!(path.is_absolute());
        assert!(path.to_string_lossy().contains("ctrl-c"));
        assert!(path.to_string_lossy().contains("config.toml"));
    }

    #[test]
    fn test_partial_config_uses_defaults() {
        let partial = r##"
            [theme]
            bg_primary = "#FF0000"
        "##;
        let config: Config = toml::from_str(partial).unwrap();
        assert_eq!(config.theme.bg_primary, "#FF0000");
        assert_eq!(config.theme.bg_secondary, "#1E293B");
        assert_eq!(config.window.width, 420);
    }
}
