# Ctrl+C — Changelog

> This file is updated after every task/prompt. It tracks what changed, what didn't, and any errors encountered.

---

## [Pre-Task] — Design Document Created — 2026-05-24

### ✅ What Changed
- Created `Docs/DESIGN.md` — full technical design document
  - Tech stack decision: **Tauri v2** (Rust + Vanilla JS) for cross-platform + low resource usage
  - Architecture diagram, database schema, project structure
  - 14 tasks broken down with clear acceptance criteria
  - Platform-specific notes for Arch, Debian, Fedora, and Windows
  - Wayland compatibility matrix for clipboard, tray, and hotkeys
  - Theme config specification in TOML format
  - Agent workflow instructions (one task at a time, update changelog after each)
- Created `Docs/changelog.md` — this file

### ⏭️ What Was Not Changed
- No code written yet — design phase only
- Previous PyQt6 prototype (in `/Coding/Hammad/Ctrl+C/`) is separate; this is a fresh start in `/Summer/Ctrl+C/`

### ❌ Errors Faced
- None

### 📝 Notes
- Chose Tauri v2 over PyQt6 (previous attempt) for significantly lower memory footprint and native Windows support
- Wayland global shortcuts are the biggest risk — fallback strategy documented
- Frontend is intentionally framework-free (no React/Vue) to keep bundle minimal
- Ready to begin **Task 1: Project Initialization** on next prompt

---

## [Task 1] — Project Initialization — 2026-05-24

### ✅ What Changed
- Created `package.json` — minimal, with `@tauri-apps/cli` as devDep
- Created `src/index.html` — minimal vanilla HTML shell
- Created `src-tauri/Cargo.toml` — all Rust dependencies (tauri, rusqlite, serde, toml, argon2, aes-gcm, arboard, chrono, dirs)
- Created `src-tauri/build.rs` — standard `tauri_build::build()` entry
- Created `src-tauri/src/main.rs` — binary entry calling `ctrl_c_lib::run()`
- Created `src-tauri/src/lib.rs` — Tauri builder with `generate_context!()`
- Created `src-tauri/tauri.conf.json` — app name "Ctrl+C", window 420x600, no dev server
- Created `src-tauri/capabilities/default.json` — permissions for core, window, event, clipboard-manager, global-shortcut
- Created `src-tauri/icons/` — placeholder clipboard icons (32, 128, 256 PNG + ICO)
- Created `.gitignore` — Rust/Node/OS artifacts

### ⏭️ What Was Not Changed
- No Rust module files yet (clipboard.rs, database.rs, etc.)
- Frontend remains a single placeholder HTML page

### ❌ Errors Faced
- First build failed: missing system `-dev` packages (`libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, etc.)
- Resolved: user installed them via apt
- `cargo tauri dev` not available (CLI installed via npm, not cargo); use `npm run tauri dev` instead

### 📝 Notes
- App launches successfully with `npm run tauri dev`
- Verify command: `npm run tauri dev` (not `cargo tauri dev`)
- Build takes ~5 min on first run due to full dependency compilation

---

## [Task 2] — Database Layer — 2026-05-24

### ✅ What Changed
- Created `src-tauri/src/database.rs` — full database module:
  - `Database` struct wrapping `Mutex<Connection>` for thread-safe SQLite access
  - Schema auto-migration on init (entries table + indexes for timestamp, pinned)
  - CRUD: `add_entry()`, `get_entries()`, `delete_entry()`, `toggle_pin()`
  - FIFO cleanup: deletes oldest unpinned entries when count exceeds 100
  - Search: case-insensitive substring match via `LIKE`
  - Date filter: today, yesterday, 7 days, 30 days filters via SQLite datetime
  - Preview generation (first 100 chars + "..." if longer)
  - Duplicate detection (skips if content matches most recent entry)
- Updated `src-tauri/src/lib.rs`:
  - `mod database` declaration
  - 4 Tauri commands: `add_entry`, `get_entries`, `delete_entry`, `toggle_pin`
  - Database init at startup using `dirs::data_dir()` path (~/.local/share/ctrl-c/history.db)
  - Creates data directory automatically

### ✅ Tests
- 9 unit tests all pass: add/get, duplicate detection, delete, toggle_pin, FIFO cleanup (2), FIFO keeps pinned, search, date_filter_today, preview_truncation

### ⏭️ What Was Not Changed
- No config.rs, clipboard.rs, hotkey.rs, or other modules yet
- Frontend unchanged

### ❌ Errors Faced
- None

### 📝 Notes
- `new_in_memory()` is `#[cfg(test)]` — only compiled in test builds
- Search uses `LIKE '%query%'` (case-insensitive for ASCII by default in SQLite; for Unicode use `LIKE` is case-insensitive only for ASCII, but this suffices MVP)
- Date filters are string-based match on SQLite filter names
