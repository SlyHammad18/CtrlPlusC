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

---

## [Task 3] — Config Manager — 2026-05-24

### ✅ What Changed
- Created `src-tauri/src/config.rs` — full config module:
  - `Config` struct with nested `ThemeConfig`, `WindowConfig`, `BehaviorConfig`, `HotkeyConfig`
  - All fields match DESIGN.md §4.6 spec with `Default` impls and serde defaults
  - Top-level fields: `autostart` (bool), `private_mode_password_hash` (string)
  - `get_config_path()` — platform-aware path using `dirs::config_dir()`:
    - Linux: `~/.config/ctrl-c/config.toml`
    - Windows: `%APPDATA%/ctrl-c/config.toml`
  - `load_config()` — reads TOML from disk, auto-creates default if missing
  - `save_config()` — writes TOML to disk, creates parent dirs
- Updated `src-tauri/src/lib.rs`:
  - `mod config` declaration
  - 2 new Tauri commands: `get_config`, `save_config`
  - Config loaded at startup, managed as `Mutex<Config>` Tauri state
  - `save_config` persists to disk and updates in-memory state

### ✅ Tests
- 4 unit tests all pass: `test_default_config`, `test_config_roundtrip`, `test_config_path_is_absolute`, `test_partial_config_uses_defaults`

### ⏭️ What Was Not Changed
- No frontend changes yet
- No other Rust modules modified

### ❌ Errors Faced
- Build error: raw string literal `r#"..."#` conflicted with inner `"#FF0000"` quotes in test
- Resolved: used `r##"..."##` delimiter instead

### 📝 Notes
- Config fields use `#[serde(default)]` so partial configs gracefully fill in defaults
- Default config is auto-created on first run if the file doesn't exist
- Path uses `dirs::config_dir()` which maps to XDG on Linux and AppData/Roaming on Windows

---

## [Task 4] — Clipboard Monitor — 2026-05-24

### ✅ What Changed
- Created `src-tauri/src/clipboard.rs` — background clipboard polling thread:
  - `ClipboardMonitor` struct with `private_mode: Arc<AtomicBool>` flag
  - `start_monitoring()` function polls `arboard::Clipboard` at configurable interval
  - Duplicate detection: compares with last captured content before saving
  - Calls `db.add_entry()` to persist new entries
  - Emits Tauri event `clipboard-changed` to frontend with the new entry
  - Respects private mode flag (pauses monitoring when locked, resets last_content)
  - Auto-retries clipboard initialization if it fails initially
- Updated `src-tauri/src/lib.rs`:
  - Refactored Database management to `Arc<Database>` for sharing between commands + clipboard thread
  - Updated all command signatures: `State<'_, Database>` → `State<'_, Arc<Database>>`
  - Added `use tauri::Manager` for AppHandle state access in setup
  - Added `ClipboardMonitor` to managed Tauri state
  - Added `set_private_mode` Tauri command to toggle clipboard monitoring
  - Poll interval reads from config behavior section via `setup` hook

### ✅ Tests
- All 13 existing tests pass (9 database + 4 config) — no test regressions
- Clipboard module has no standalone tests (requires system clipboard access)

### ⏭️ What Was Not Changed
- No frontend changes yet (frontend listens for `clipboard-changed` event in Task 6)
- No config, private_mode, hotkey, or autostart modules modified

### ❌ Errors Faced
- Missing `use tauri::Manager` import — AppHandle needs `Manager` trait in scope for `.state()` method
- Lifetime issue accessing Config state inside setup closure — resolved by binding MutexGuard to explicit local variable in inner block

### 📝 Notes
- Clipboard monitor runs in a detached `std::thread::spawn` — not Tokio, since it's a simple sleep-poll loop
- `arboard::Clipboard` is not thread-safe, so it's created inside the thread and never shared
- Private mode flag is `Arc<AtomicBool>` so the Tauri command and monitor thread can share it atomically

---

## [Task 5] — Frontend: Core UI — 2026-05-24

### ✅ What Changed
- Updated `src/index.html` — full semantic layout:
  - Header with search input + action buttons (date filter, private mode lock)
  - Date filter nav (All, Today, Yesterday, 7 Days, 30 Days)
  - Main scrollable card list with empty state placeholder
  - Footer with version info and settings button
  - Inline SVG icons for all buttons (no external icon deps)
  - Loads Inter font from Google Fonts CDN + system-ui fallback
  - Loads CSS files (main, cards, animations) and JS modules (app, api, search, theme, ui)
- Created `src/styles/main.css` — core styles:
  - CSS custom properties for all theme values (matching config defaults)
  - App layout: flex column with fixed header/filter/footer, scrollable card area
  - Search input with icon via CSS mask
  - Date filter tabs, icon buttons, empty state, custom scrollbar
- Created `src/styles/cards.css` — clipboard card styles:
  - Card with hover lift (translateY -1px), shadow, border highlight
  - Selected state (via keyboard nav), pinned indicator (accent left border)
  - Action buttons (copy, pin, delete) fade in on hover
  - Highlighted search matches in previews
  - No-results empty state styling
- Created `src/styles/animations.css` — micro-animations:
  - `fadeIn`, `slideIn`, `slideOut` for card enter/exit
  - `shake` for error states, `pulse` for loading
  - Smooth 200-300ms transitions on all interactive elements
- Created `src/js/theme.js` — theme loader:
  - `theme.load()` invokes `get_config` backend command
  - Applies config theme values as CSS custom properties on `:root`
- Created `src/js/api.js` — Tauri IPC wrapper:
  - Wraps all backend commands (`add_entry`, `get_entries`, `delete_entry`, `toggle_pin`, `get_config`, `save_config`, `set_private_mode`)
  - Uses `window.__TAURI__.core.invoke` (withGlobalTauri)
- Created `src/js/search.js` — search & filter logic:
  - Debounced input (300ms), date filter tab selection
  - Text highlighting helper for search results
- Created `src/js/ui.js` — DOM manipulation:
  - `renderCards()` — batch render entry list
  - `prependCard()` — animate new entry insertion
  - `removeCard()` — slide-out + delete
  - `updatePinState()` — toggle pinned class
  - `formatTimestamp()` — relative time display
- Created `src/js/app.js` — main app logic:
  - Initializes theme, loads entries on startup
  - Wires search/filter callbacks to backend
  - Click handlers: card body (copy), copy/pin/delete buttons
  - Keyboard navigation: ArrowUp/Down, Enter (copy), Delete (remove)
  - Listens for `clipboard-changed` event via `window.__TAURI__.event.listen`
- Updated `src-tauri/tauri.conf.json`:
  - Added `"withGlobalTauri": true` for `window.__TAURI__` access in webview
- Updated `src-tauri/capabilities/default.json`:
  - Already had necessary event permissions (unchanged)

### ✅ Tests
- All 13 Rust tests pass (unchanged)
- Frontend loads without errors in Tauri webview (verified via `npm run tauri dev`)

### ⏭️ What Was Not Changed
- No backend Rust modules modified (only tauri.conf.json)
- Lock screen CSS/JS (Task 8) not yet implemented — lock icon is placeholder-only

### ❌ Errors Faced
- `withGlobalTauri` placed inside `security` block was invalid — Tauri v2 schema rejected it
- Removed to top-level `app` key — build succeeded

### 📝 Notes
- frontend uses `window.__TAURI__` global API — no bundler, no npm frontend deps
- CSS variables mirror the config TOML schema for seamless theming
- JS modules are loaded via `<script type="module">` in dependency order (theme → api → search → ui → app)
