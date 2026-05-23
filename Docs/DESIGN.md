# Ctrl+C — Cross-Platform Clipboard Manager

## Design Document v1.0

> **Platforms:** Windows 10/11, Linux (Arch, Debian/Ubuntu, Fedora)
> **Goal:** Modern UI, low resource usage, privacy-first clipboard manager

---

## 1. Overview

**Ctrl+C** is a lightweight, cross-platform clipboard manager that lives in the system tray. It automatically captures clipboard history, supports search/filter, pinning, private mode with password lock, and is fully themeable via a config file. It targets both X11 and Wayland on Linux, and Windows natively.

---

## 2. Tech Stack

| Layer | Technology | Rationale |
|---|---|---|
| **Framework** | **Tauri v2** (Rust backend + WebView frontend) | Smallest footprint (~5-10MB RAM), uses native OS webview, no bundled Chromium |
| **Backend** | **Rust** | Memory-safe, high performance, direct OS API access |
| **Frontend** | **Vanilla HTML/CSS/JS** (no framework) | Zero bundle overhead, full design control |
| **Database** | **SQLite** via `rusqlite` | Single-file, zero-config, fast for 100 entries |
| **Encryption** | `argon2` + `aes-gcm` (Rust crates) | Password hashing + AES-256 encryption for private mode |
| **Config** | **TOML** file | Human-readable, easy to edit, Rust-native support via `toml` crate |

### Why Tauri over PyQt6/Electron?

- **Memory:** ~15MB vs PyQt6 (~50MB) vs Electron (~100MB+)
- **Binary Size:** ~3MB vs PyQt6 (~150MB with venv) vs Electron (~150MB+)
- **Cross-platform:** Native on Windows + Linux without Python runtime
- **Security:** Sandboxed by default, explicit permission model

---

## 3. Architecture

```
┌─────────────────────────────────────────────┐
│                  Frontend                    │
│         (HTML / CSS / Vanilla JS)            │
│  ┌──────────┐ ┌────────┐ ┌──────────────┐   │
│  │ Card List │ │ Search │ │ Private Lock │   │
│  └──────────┘ └────────┘ └──────────────┘   │
└──────────────────┬──────────────────────────┘
                   │ Tauri IPC (invoke/events)
┌──────────────────▼──────────────────────────┐
│               Rust Backend                   │
│  ┌────────────┐ ┌───────────┐ ┌──────────┐  │
│  │ Clipboard  │ │ Database  │ │ Config   │  │
│  │ Monitor    │ │ Manager   │ │ Manager  │  │
│  └────────────┘ └───────────┘ └──────────┘  │
│  ┌────────────┐ ┌───────────┐ ┌──────────┐  │
│  │ Hotkey     │ │ Private   │ │ Autostart│  │
│  │ Manager    │ │ Mode      │ │ Manager  │  │
│  └────────────┘ └───────────┘ └──────────┘  │
└──────────────────┬──────────────────────────┘
                   │
        ┌──────────▼──────────┐
        │   SQLite Database    │
        │  (~/.local/share/    │
        │   ctrl-c/history.db) │
        └─────────────────────┘
```

---

## 4. Feature Specifications

### 4.1 Clipboard History (100 Entries)

- Monitor system clipboard for text changes via polling (500ms interval)
- Store up to **100 entries** in SQLite (FIFO — oldest auto-deleted)
- Each entry stores: `id`, `content`, `preview` (first 100 chars), `timestamp`, `is_pinned`, `is_private`
- Duplicate detection: skip if content matches the most recent entry
- **Platform-specific monitoring:**
  - **Windows:** `arboard` crate (native Win32 clipboard API)
  - **X11:** `arboard` crate (X11 selections)
  - **Wayland:** `arboard` with `wayland-data-control` feature + `wl-clipboard` fallback

### 4.2 Core Actions

| Action | Description |
|---|---|
| **Copy** | Re-copy entry to system clipboard |
| **Pin** | Toggle pin status, pinned items are excluded from FIFO deletion |
| **Delete** | Remove single entry from history |

### 4.3 Search

- Real-time text search across all entry content
- Case-insensitive substring matching
- Debounced input (300ms) to avoid excessive queries
- Highlights matching text in results

### 4.4 Filter by Date

- Filter options: Today, Yesterday, Last 7 Days, Last 30 Days, All
- Uses SQLite `datetime()` for efficient filtering
- Combinable with text search

### 4.5 Private Mode (Password Lock)

- **Lock:** User sets a password → all clipboard monitoring pauses, UI is hidden behind a lock screen
- **Unlock:** User enters password → monitoring resumes, history visible again
- Password hashed with **Argon2id** and stored in config
- When locked: clipboard is NOT monitored (no entries saved)
- Lock state persists across app restarts
- Optional: encrypt stored entries with AES-256-GCM derived from password

### 4.6 Theming (Config File)

Default dark theme, fully customizable via TOML config at:
- **Linux:** `~/.config/ctrl-c/config.toml`
- **Windows:** `%APPDATA%\ctrl-c\config.toml`

```toml
[theme]
bg_primary = "#0F172A"
bg_secondary = "#1E293B"
bg_card = "#334155"
text_primary = "#F8FAFC"
text_secondary = "#94A3B8"
accent = "#3B82F6"
accent_hover = "#2563EB"
danger = "#EF4444"
success = "#22C55E"
border = "#475569"
border_radius = "12px"
font_family = "Inter, system-ui, sans-serif"
font_size = "14px"

[window]
width = 420
height = 600
opacity = 0.97

[behavior]
max_entries = 100
poll_interval_ms = 500
search_debounce_ms = 300
```

The frontend reads theme values from the backend at startup and applies them as CSS custom properties.

### 4.7 Autostart on Login

- **Linux:** Create `.desktop` file in `~/.config/autostart/`
- **Windows:** Registry entry in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
- Toggle via settings UI or config file: `autostart = true`

### 4.8 Global Hotkey

Default: `Ctrl+Shift+V` (configurable in config.toml)

| Platform | Implementation |
|---|---|
| **Windows** | `tauri-plugin-global-shortcut` (native RegisterHotKey) |
| **X11** | `tauri-plugin-global-shortcut` (XGrabKey) |
| **Wayland** | D-Bus `GlobalShortcuts` portal via `xdg-desktop-portal`. Fallback: user configures DE-native keybind to send `SIGUSR1` or launch toggle command |

```toml
[hotkey]
toggle_window = "Ctrl+Shift+V"
```

> **Wayland Note:** Global shortcuts are compositor-dependent. On GNOME/KDE/Sway, the app will attempt the `xdg-desktop-portal` GlobalShortcuts interface first. If unavailable, the user is prompted to set a custom keybind in their DE settings.

---

## 5. UI Design

### 5.1 Layout

```
┌──────────────────────────────┐
│  🔍 Search...    [📅] [🔒]  │  ← Header bar
├──────────────────────────────┤
│  Today · Yesterday · 7d · All│  ← Date filter tabs
├──────────────────────────────┤
│ ┌──────────────────────────┐ │
│ │ 📌 Pinned entry text...  │ │  ← Clipboard card
│ │ 2 min ago    [📋][📌][🗑]│ │
│ └──────────────────────────┘ │
│ ┌──────────────────────────┐ │
│ │ Regular entry text...    │ │
│ │ 15 min ago   [📋][📌][🗑]│ │
│ └──────────────────────────┘ │
│         ... more cards ...    │
├──────────────────────────────┤
│  Ctrl+C v1.0    [⚙ Settings]│  ← Footer
└──────────────────────────────┘
```

### 5.2 Design Principles

- **Dark-first:** Deep slate/navy background (#0F172A), subtle card elevation
- **Glassmorphism:** Slight backdrop blur + transparency on window
- **Micro-animations:** Card hover lift (translateY -2px), fade-in on load, smooth transitions (200ms ease)
- **Typography:** Inter font via Google Fonts CDN (bundled fallback for offline)
- **Responsive:** Fixed window size from config, scrollable card list
- **Keyboard navigation:** Arrow keys to navigate cards, Enter to copy, Delete to remove

### 5.3 Lock Screen (Private Mode)

- Blurred overlay covering all content
- Centered password input with lock icon
- Shake animation on wrong password
- Minimal — only password field and unlock button visible

---

## 6. Database Schema

```sql
CREATE TABLE IF NOT EXISTS entries (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    content     TEXT NOT NULL,
    preview     TEXT NOT NULL,
    timestamp   TEXT NOT NULL DEFAULT (datetime('now')),
    is_pinned   INTEGER NOT NULL DEFAULT 0,
    is_private  INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_timestamp ON entries(timestamp);
CREATE INDEX idx_pinned ON entries(is_pinned);
```

---

## 7. Project Structure

```
Ctrl+C/
├── Docs/
│   ├── DESIGN.md              # This file
│   └── changelog.md           # Updated after every task
├── src-tauri/
│   ├── Cargo.toml             # Rust dependencies
│   ├── tauri.conf.json        # Tauri configuration
│   ├── capabilities/          # Tauri v2 permissions
│   │   └── default.json
│   ├── icons/                 # App icons
│   └── src/
│       ├── main.rs            # Entry point
│       ├── lib.rs             # Tauri setup & plugin registration
│       ├── clipboard.rs       # Clipboard monitoring logic
│       ├── database.rs        # SQLite operations
│       ├── config.rs          # TOML config loading/saving
│       ├── hotkey.rs          # Global shortcut management
│       ├── private_mode.rs    # Password lock/unlock logic
│       └── autostart.rs       # Autostart registration
├── src/                       # Frontend
│   ├── index.html             # Main HTML
│   ├── styles/
│   │   ├── main.css           # Core styles + CSS variables
│   │   ├── cards.css          # Clipboard card styles
│   │   ├── lock.css           # Lock screen styles
│   │   └── animations.css     # Micro-animations
│   └── js/
│       ├── app.js             # Main app logic
│       ├── api.js             # Tauri IPC wrapper
│       ├── theme.js           # Theme loader from config
│       ├── search.js          # Search & filter logic
│       └── ui.js              # DOM manipulation helpers
├── .gitignore
├── package.json               # Frontend dependencies (minimal)
└── README.md
```

---

## 8. Platform-Specific Notes

### 8.1 Linux Distribution Packages

| Distro | Package Format | Dependencies |
|---|---|---|
| **Arch** | `.pkg.tar.zst` (AUR) | `webkit2gtk`, `libappindicator-gtk3` |
| **Debian/Ubuntu** | `.deb` | `libwebkit2gtk-4.1-0`, `libappindicator3-1` |
| **Fedora** | `.rpm` | `webkit2gtk4.1`, `libappindicator-gtk3` |

### 8.2 Windows

- Target: Windows 10+ (WebView2 is pre-installed)
- Installer: `.msi` via Tauri bundler
- Autostart: Registry-based

### 8.3 Wayland Compatibility Matrix

| Feature | GNOME | KDE | Sway/Hyprland |
|---|---|---|---|
| Clipboard Monitor | ✅ | ✅ | ✅ |
| System Tray | ✅ (via appindicator) | ✅ | ✅ (waybar) |
| Global Shortcut | ⚠️ Portal | ✅ Portal | ⚠️ Manual config |

---

## 9. Task Breakdown

> **Instructions for the Agent:**
> - Perform each task **one by one**, in order
> - After completing each task, update `Docs/changelog.md` with: what changed, what didn't, any errors faced
> - Do NOT skip tasks or combine multiple tasks
> - Test/verify each task before moving to the next
> - If a task fails, document the error in changelog.md and attempt to fix before proceeding

### Task 1: Project Initialization
- [ ] Initialize Tauri v2 project with Vanilla JS frontend
- [ ] Set up `Cargo.toml` with dependencies: `rusqlite`, `serde`, `toml`, `argon2`, `aes-gcm`, `arboard`
- [ ] Configure `tauri.conf.json` (app name, window size, permissions)
- [ ] Set up `capabilities/default.json` for clipboard, global-shortcut, tray permissions
- [ ] Create `.gitignore` for Rust + Node artifacts
- [ ] Verify project builds and runs (`cargo tauri dev`)
- [ ] Update `changelog.md`

### Task 2: Database Layer
- [ ] Implement `database.rs` with SQLite connection pool
- [ ] Create `entries` table schema with auto-migration on first run
- [ ] Implement CRUD operations: `add_entry()`, `get_entries()`, `delete_entry()`, `toggle_pin()`
- [ ] Implement FIFO cleanup (delete oldest unpinned when count > 100)
- [ ] Implement search query: `search_entries(query, date_filter)`
- [ ] Expose all DB functions as Tauri commands
- [ ] Unit test the database module
- [ ] Update `changelog.md`

### Task 3: Config Manager
- [ ] Implement `config.rs` — load/save TOML config file
- [ ] Define default config struct with all theme, window, behavior, hotkey fields
- [ ] Auto-create config file with defaults on first run
- [ ] Platform-aware config path (XDG on Linux, %APPDATA% on Windows)
- [ ] Expose `get_config` and `save_config` as Tauri commands
- [ ] Update `changelog.md`

### Task 4: Clipboard Monitor
- [ ] Implement `clipboard.rs` — background clipboard polling thread
- [ ] Use `arboard` crate for cross-platform clipboard access
- [ ] Poll every 500ms (configurable), detect text changes
- [ ] Skip duplicates (compare with last captured content)
- [ ] Emit Tauri event `clipboard-changed` to frontend on new entry
- [ ] Respect private mode flag (pause monitoring when locked)
- [ ] Update `changelog.md`

### Task 5: Frontend — Core UI
- [ ] Build `index.html` with semantic structure (header, card list, footer)
- [ ] Create `main.css` with CSS custom properties for theming
- [ ] Create `cards.css` with card component styles (hover, elevation, spacing)
- [ ] Create `animations.css` with micro-animations (fade-in, hover lift, slide)
- [ ] Implement `theme.js` — load theme from backend config, apply as CSS vars
- [ ] Implement dark theme as default with glassmorphism effects
- [ ] Load Inter font (bundled or CDN with fallback)
- [ ] Update `changelog.md`

### Task 6: Frontend — Card List & Actions
- [ ] Implement `app.js` — main application logic
- [ ] Implement `api.js` — Tauri invoke wrappers for all backend commands
- [ ] Render clipboard entries as cards with: preview, timestamp, action buttons
- [ ] Implement Copy button → re-copy to clipboard via backend
- [ ] Implement Pin button → toggle pin state, re-sort (pinned first)
- [ ] Implement Delete button → remove entry with confirmation
- [ ] Listen for `clipboard-changed` event → prepend new card with animation
- [ ] Keyboard navigation: Arrow keys, Enter to copy, Delete to remove
- [ ] Update `changelog.md`

### Task 7: Search & Date Filter
- [ ] Implement `search.js` — search input with 300ms debounce
- [ ] Wire search to backend `search_entries` command
- [ ] Highlight matching text in card previews
- [ ] Implement date filter tabs (Today, Yesterday, 7 Days, 30 Days, All)
- [ ] Combine search + date filter queries
- [ ] Show "No results" state with appropriate message
- [ ] Update `changelog.md`

### Task 8: Private Mode (Lock/Unlock)
- [ ] Implement `private_mode.rs` — password hash/verify with Argon2id
- [ ] Store hashed password in config file
- [ ] First-time setup: prompt user to create password
- [ ] Lock command: pause clipboard monitor, emit lock event
- [ ] Unlock command: verify password, resume monitor, emit unlock event
- [ ] Create `lock.css` — lock screen overlay styles
- [ ] Frontend lock screen: blurred overlay, password input, shake on error
- [ ] Persist lock state across restarts
- [ ] Update `changelog.md`

### Task 9: System Tray
- [ ] Configure tray icon in `lib.rs` using Tauri's built-in tray API
- [ ] Generate/add app icon (PNG + ICO formats)
- [ ] Tray menu items: Show/Hide, Private Mode Toggle, Quit
- [ ] Click tray icon → toggle window visibility
- [ ] Tray tooltip showing "Ctrl+C — Clipboard Manager"
- [ ] Update `changelog.md`

### Task 10: Global Hotkey
- [ ] Implement `hotkey.rs` — register configurable global shortcut
- [ ] Use `tauri-plugin-global-shortcut` for Windows + X11
- [ ] Read hotkey from config (default: `Ctrl+Shift+V`)
- [ ] Hotkey action: toggle window show/hide
- [ ] Wayland fallback: detect Wayland session, show setup instructions to user
- [ ] Update `changelog.md`

### Task 11: Autostart
- [ ] Implement `autostart.rs` — enable/disable autostart
- [ ] Linux: create/remove `.desktop` file in `~/.config/autostart/`
- [ ] Windows: add/remove registry key in `HKCU\...\Run`
- [ ] Toggle from settings UI and config file
- [ ] Expose as Tauri command
- [ ] Update `changelog.md`

### Task 12: Settings Panel
- [ ] Add settings view/modal in frontend
- [ ] Settings: autostart toggle, hotkey display, private mode setup, theme reset
- [ ] About section with version info
- [ ] Save settings changes to config via backend
- [ ] Update `changelog.md`

### Task 13: Polish & Platform Testing
- [ ] Test on X11 Linux environment
- [ ] Test on Wayland Linux environment
- [ ] Test on Windows (if available)
- [ ] Performance check: verify <20MB RAM usage
- [ ] Fix any platform-specific bugs
- [ ] Final UI polish pass (animations, spacing, colors)
- [ ] Update `changelog.md`

### Task 14: Packaging & Distribution
- [ ] Configure Tauri bundler for `.deb`, `.rpm`, `.AppImage`, `.msi`
- [ ] Add app metadata (description, license, author)
- [ ] Build release binaries for each target
- [ ] Write comprehensive `README.md` with install instructions per distro
- [ ] Final `changelog.md` update

---

## 10. Changelog Protocol

After **every task/prompt**, update `Docs/changelog.md` with this format:

```markdown
## [Task X] — Task Title — YYYY-MM-DD

### ✅ What Changed
- Bullet list of files created/modified
- What was implemented

### ⏭️ What Was Not Changed
- Anything skipped or deferred

### ❌ Errors Faced
- Any build errors, runtime issues, or bugs encountered
- How they were resolved (or if still open)

### 📝 Notes
- Any decisions made, assumptions, or things to revisit
```

---

## 11. Dependencies Summary

### Rust (Cargo.toml)
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-clipboard-manager = "2"
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
argon2 = "0.5"
aes-gcm = "0.10"
arboard = { version = "3", features = ["wayland-data-control"] }
chrono = "0.4"
dirs = "5"
```

### Frontend (package.json)
```json
{
  "dependencies": {},
  "devDependencies": {
    "@tauri-apps/cli": "^2"
  }
}
```
> Minimal — no frontend framework dependencies.

---

*Document created: 2026-05-24*
*Last updated: 2026-05-24*
