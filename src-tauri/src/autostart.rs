fn get_app_name() -> &'static str {
    "Ctrl+C"
}

#[cfg(target_os = "windows")]
pub fn enable_autostart() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let path = exe.to_string_lossy();

    let output = std::process::Command::new("reg")
        .args([
            "add",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            get_app_name(),
            "/t",
            "REG_SZ",
            "/d",
            &path,
            "/f",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to enable autostart: {}", stderr))
    }
}

#[cfg(target_os = "windows")]
pub fn disable_autostart() -> Result<(), String> {
    let output = std::process::Command::new("reg")
        .args([
            "delete",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            get_app_name(),
            "/f",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to disable autostart: {}", stderr))
    }
}

#[cfg(target_os = "windows")]
pub fn is_autostart_enabled() -> Result<bool, String> {
    let output = std::process::Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            get_app_name(),
        ])
        .output()
        .map_err(|e| e.to_string())?;

    Ok(output.status.success())
}

#[cfg(target_os = "linux")]
fn autostart_desktop_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(home);
    path.push(".config");
    path.push("autostart");
    path.push("ctrl-c.desktop");
    path
}

#[cfg(target_os = "linux")]
pub fn enable_autostart() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let path = exe.to_string_lossy();

    let desktop_path = autostart_desktop_path();
    if let Some(parent) = desktop_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Ctrl+C\n\
         Exec={}\n\
         X-GNOME-Autostart-enabled=true\n\
         NoDisplay=true\n",
        path
    );

    std::fs::write(&desktop_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn disable_autostart() -> Result<(), String> {
    let desktop_path = autostart_desktop_path();
    if desktop_path.exists() {
        std::fs::remove_file(&desktop_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn is_autostart_enabled() -> Result<bool, String> {
    Ok(autostart_desktop_path().exists())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn enable_autostart() -> Result<(), String> {
    Err("Autostart not supported on this platform".to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn disable_autostart() -> Result<(), String> {
    Err("Autostart not supported on this platform".to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn is_autostart_enabled() -> Result<bool, String> {
    Ok(false)
}
