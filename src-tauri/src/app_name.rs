//! Linux application-name resolution.
//!
//! Given a process id (and optionally a Wayland `wm_class_instance` / app-id),
//! resolve a human-friendly application name by matching the process against
//! the installed `.desktop` files (using their `Name=`, `Exec=`,
//! `StartupWMClass=` and desktop-file id). Falls back to the executable
//! basename when no desktop file matches.

use std::fs;
use std::path::Path;
use std::sync::OnceLock;

/// A parsed `.desktop` entry, reduced to the fields we need for matching.
struct DesktopAppInfo {
    /// The `Name=` value (human-friendly, e.g. "GNOME Terminal").
    name: String,
    /// Basename of the first token of `Exec=` (the real binary).
    exec_bin: String,
    /// `StartupWMClass=` value (matches a window's WM_CLASS / app-id).
    wm_class: String,
    /// Desktop-file id: filename without the `.desktop` extension
    /// (e.g. `org.gnome.TextEditor`).
    id: String,
}

static DESKTOP_APPS: OnceLock<Vec<DesktopAppInfo>> = OnceLock::new();

/// XDG data directories that may contain an `applications/` subtree of
/// `.desktop` files, in lookup-priority order.
fn desktop_app_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();

    if let Some(home) = dirs_home() {
        dirs.push(home.join(".local/share/applications"));
        dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
    }

    dirs.push(std::path::PathBuf::from("/usr/local/share/applications"));
    dirs.push(std::path::PathBuf::from("/usr/share/applications"));
    dirs.push(std::path::PathBuf::from(
        "/var/lib/flatpak/exports/share/applications",
    ));

    dirs
}

/// Portable `$HOME` lookup without pulling in the `dirs` crate here.
fn dirs_home() -> Option<std::path::PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return Some(std::path::PathBuf::from(home));
        }
    }
    None
}

/// Parse a single `.desktop` file into a `DesktopAppInfo`.
///
/// Only the `[Desktop Entry]` group is considered; localization suffixes
/// (e.g. `Name[en]`) are ignored so we get the untranslated default.
fn parse_desktop(path: &Path) -> Option<DesktopAppInfo> {
    let content = fs::read_to_string(path).ok()?;
    let id = path.file_stem()?.to_string_lossy().to_string();

    let mut name = String::new();
    let mut exec_bin = String::new();
    let mut wm_class = String::new();
    let mut in_entry = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[Desktop Entry]" {
            in_entry = true;
            continue;
        }
        if line.starts_with('[') {
            // Entering a different group (e.g. [Desktop Action ...]).
            in_entry = false;
            continue;
        }
        if !in_entry {
            continue;
        }
        if let Some(v) = kv(line, "Name") {
            if name.is_empty() {
                name = v;
            }
        } else if let Some(v) = kv(line, "Exec") {
            exec_bin = exec_binary(&v);
        } else if let Some(v) = kv(line, "StartupWMClass") {
            wm_class = v;
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(DesktopAppInfo {
        name,
        exec_bin,
        wm_class,
        id,
    })
}

/// Extract the leading executable basename from an `Exec=` value, stripping
/// environment assignments (`FOO=bar command ...`), quoting, and freedesktop
/// field codes (`%f`, `%U`, etc.).
fn exec_binary(exec_value: &str) -> String {
    let mut tokens = exec_value.split_whitespace().peekable();
    // Handle `env [KEY=VALUE ...] COMMAND ...` style launchers: consume the
    // `env` token, then any leading `KEY=VALUE` assignments.
    if let Some(t) = tokens.peek() {
        if *t == "env" {
            let _ = tokens.next();
            while let Some(t) = tokens.peek() {
                if t.contains('=') && !t.contains('/') {
                    let _ = tokens.next();
                } else {
                    break;
                }
            }
        }
    }
    let first = match tokens.next() {
        Some(t) => t,
        None => return String::new(),
    };
    // Strip a leading `~` or variable; we only care about the basename.
    let first = first.trim_matches('"');
    let path = std::path::Path::new(first);
    let base = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(first);
    // Drop any field codes that survived (shouldn't, but be safe).
    base.trim_end_matches(|c| c == '%').to_string()
}

/// Parse `Key=Value`, ignoring localized keys (`Key[en]=...`).
fn kv(line: &str, key: &str) -> Option<String> {
    let mut parts = line.splitn(2, '=');
    let k = parts.next()?;
    if k != key {
        return None;
    }
    let v = parts.next()?.trim().to_string();
    Some(v)
}

/// Lazily load and cache all `.desktop` app descriptions.
fn load_desktop_apps() -> &'static Vec<DesktopAppInfo> {
    DESKTOP_APPS.get_or_init(|| {
        let mut apps = Vec::new();
        for dir in desktop_app_dirs() {
            let entries = match fs::read_dir(&dir) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                    if let Some(info) = parse_desktop(&path) {
                        apps.push(info);
                    }
                }
            }
        }
        apps
    })
}

/// Resolve the executable basename for a pid via `/proc/<pid>/exe`, falling
/// back to `/proc/<pid>/comm` (the kernel process name).
fn exe_basename(pid: u32) -> Option<String> {
    let exe_link = format!("/proc/{}/exe", pid);
    if let Ok(target) = fs::read_link(&exe_link) {
        if let Some(name) = target.file_name().and_then(|n| n.to_str()) {
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    let comm_path = format!("/proc/{}/comm", pid);
    if let Ok(comm) = fs::read_to_string(&comm_path) {
        let comm = comm.trim().to_string();
        if !comm.is_empty() {
            return Some(comm);
        }
    }
    None
}

/// Resolve a friendly application name for the given process.
///
/// `wm_class_instance` is the Wayland/KDE window class or app-id (e.g.
/// `org.gnome.TextEditor`); it may be empty on X11 where only the pid is
/// available.
pub fn resolve_linux_app_name(pid: u32, wm_class_instance: &str) -> String {
    let apps = load_desktop_apps();
    let wc = wm_class_instance.trim();

    // 1. StartupWMClass match (X11 WM_CLASS / KDE Wayland app-id).
    if !wc.is_empty() {
        if let Some(app) = apps
            .iter()
            .find(|a| !a.wm_class.is_empty() && a.wm_class.eq_ignore_ascii_case(wc))
        {
            return app.name.clone();
        }
    }

    // 2. Desktop-file id == wm_class_instance (Wayland app-id, Flatpak/Snap).
    if !wc.is_empty() {
        if let Some(app) = apps
            .iter()
            .find(|a| a.id.eq_ignore_ascii_case(wc))
        {
            return app.name.clone();
        }
    }

    if let Some(exe) = exe_basename(pid) {
        // 3. Exec binary match.
        if let Some(app) = apps
            .iter()
            .find(|a| !a.exec_bin.is_empty() && a.exec_bin.eq_ignore_ascii_case(&exe))
        {
            return app.name.clone();
        }
        // 4. Desktop-file id == exe basename.
        if let Some(app) = apps.iter().find(|a| a.id.eq_ignore_ascii_case(&exe)) {
            return app.name.clone();
        }
        // 5. Fallback: prettify the raw binary name.
        return prettify(&exe);
    }

    if !wc.is_empty() {
        return prettify(wc);
    }

    String::new()
}

/// Light prettification for raw binary names used only as a last resort
/// (e.g. `gnome-terminal-server` -> `Gnome Terminal Server`).
fn prettify(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    trimmed
        .split(|c| c == '-' || c == '_')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_binary_strips_args_and_field_codes() {
        assert_eq!(exec_binary("firefox %u"), "firefox");
        assert_eq!(exec_binary("/usr/bin/code --unity-launch %F"), "code");
        assert_eq!(
            exec_binary("env GTK_DEBUG=interactive gnome-terminal --window"),
            "gnome-terminal"
        );
    }

    #[test]
    fn kv_ignores_localized_keys() {
        assert_eq!(kv("Name=Foo", "Name"), Some("Foo".to_string()));
        assert_eq!(kv("Name[en]=Foo", "Name"), None);
    }

    #[test]
    fn prettify_handles_dashes_and_underscores() {
        assert_eq!(prettify("gnome-terminal-server"), "Gnome Terminal Server");
        assert_eq!(prettify("foo_bar"), "Foo Bar");
    }
}
