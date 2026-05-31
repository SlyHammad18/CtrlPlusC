# Ctrl+C — Cross-Platform Clipboard Manager

**Ctrl+C** is a lightweight, privacy-first clipboard manager that lives in the system tray. It automatically captures clipboard history, supports search and filtering, pinning, private mode with password lock, and is fully themeable.

Built with **Tauri v2** (Rust backend + Vanilla JS frontend) for a tiny memory footprint (~15 MB).

---

## Features

- **Clipboard History** — Automatically captures text and images from the clipboard (FIFO, up to 100 entries)
- **Search & Filter** — Real-time text search with highlighting, date filters (Today, Yesterday, 7d, 30d), and app source filtering
- **Pin Entries** — Keep important items pinned — they are excluded from auto-eviction
- **Private Mode** — Password-protect your clipboard with Argon2id hashing. When locked, clipboard monitoring pauses entirely
- **Custom Themes** — 6 built-in presets (Void Purple, Synthwave, Midnight Ocean, Cyberpunk Terminal, Arctic Frost, Obsidian) or create your own with the color picker
- **Global Hotkey** — Toggle the window from any app (default: `Alt+V`, configurable)
- **System Tray** — Minimize to tray with Show/Hide, Lock Private Mode, and Quit menu
- **Autostart** — Launch on login (toggle from Settings)
- **Keyboard Navigation** — Arrow keys, Enter to copy, Delete to remove, keyboard shortcuts for all actions
- **Source App Tracking** — See which application each clipboard entry came from, filter by app

---

## Installation

### Windows

Download the MSI installer from the [releases page](../../releases) and run it.

```
Ctrl+C_0.1.0_x64_en-US.msi
```

The app requires **WebView2** (pre-installed on Windows 10+).

### Linux

Packages available for Arch, Debian/Ubuntu, and Fedora. See the [releases page](../../releases).

**Dependencies:**
- **Arch:** `webkit2gtk`, `libappindicator-gtk3`
- **Debian/Ubuntu:** `libwebkit2gtk-4.1-0`, `libappindicator3-1`
- **Fedora:** `webkit2gtk4.1`, `libappindicator-gtk3`

---

## Usage

### First Launch

1. Launch Ctrl+C from the application menu or terminal.
2. The app starts minimized to the system tray.
3. Click the tray icon or press the global hotkey (`Alt+V`) to open.
4. Copy something — it will appear in the history.

### Global Hotkey

Default: **Alt+V** — toggle window show/hide.

To change the hotkey, open Settings → click the hotkey display → press your desired key combination.

> **Wayland note:** Global shortcuts depend on your desktop environment. On GNOME/KDE, the app attempts the `xdg-desktop-portal` GlobalShortcuts interface. If unavailable, you'll need to configure a custom keybind in your DE settings to run `ctrl-c toggle`.

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Alt+V` | Toggle window (configurable) |
| `↑`/`↓` | Navigate cards |
| `←`/`→` | Focus action buttons |
| `Enter` | Copy selected entry |
| `Delete` | Delete selected entry |
| `P` | Toggle pin |
| `E` | Edit entry |
| `Ctrl+L` | Lock private mode |
| `Ctrl+R` | Toggle recording |
| `Ctrl+I` | Toggle settings |
| `Ctrl+D` | Clear all history |
| `Ctrl+F` | Toggle app filter |
| `Ctrl+/` | Focus search |
| `Escape` | Close overlays or hide window |

### Private Mode

1. Click the lock icon in the header (or press `Ctrl+L`).
2. Set a password (minimum 4 characters).
3. The app locks — clipboard monitoring pauses, entries are hidden behind a lock screen.
4. To unlock, enter your password.
5. Lock state persists across restarts.

### Custom Themes

1. Open Settings → **Customize**.
2. Pick a preset or use the color pickers to create your own.
3. Click **Save** to persist.

Themes are stored in the config file and apply immediately.

---

## Configuration

The config file is located at:

- **Windows:** `%APPDATA%\ctrl-c\config.toml`
- **Linux:** `~/.config/ctrl-c/config.toml`

```toml
[theme]
bg_primary = "#0A0A0A"
bg_secondary = "#141414"
bg_card = "#1E1E1E"
bg_modal = "#181818"
text_primary = "#E8E8E8"
text_secondary = "#707070"
accent = "#AAAAAA"
accent_hover = "#CCCCCC"
accent_subtle = "#1A1A1A"
danger = "#E05555"
success = "#55AA77"
border = "#242424"
border_card = "#2C2C2C"
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

[hotkey]
toggle_window = "Alt+V"

autostart = false
```

---

## Building from Source

### Prerequisites

- **Rust** (edition 2021)
- **Node.js** (for `@tauri-apps/cli`)
- Platform-specific dependencies (see [Tauri docs](https://v2.tauri.app/start/prerequisites/))

### Setup

```bash
git clone https://github.com/your-org/ctrl-c.git
cd ctrl-c

npm install
```

### Development

```bash
npm run tauri dev
```

### Production Build

```bash
# All targets
npm run tauri build

# MSI only (Windows)
npm run tauri build -- --bundles msi
```

The built installer is in `src-tauri/target/release/bundle/`.

---

## Architecture

```
Frontend (Vanilla HTML/CSS/JS)
  ├── index.html          — Main HTML shell
  ├── styles/             — CSS with custom properties for theming
  │   ├── main.css        ─ Core layout
  │   ├── cards.css       ─ Clipboard card components
  │   ├── lock.css        ─ Private mode lock screen
  │   ├── settings.css    ─ Settings panel + overlays
  │   └── animations.css  ─ Micro-animations
  └── js/                 — Vanilla JS modules (no framework)
      ├── app.js          ─ Main app logic & event wiring
      ├── api.js          ─ Tauri IPC invoke wrappers
      ├── theme.js        ─ Theme loading from config
      ├── search.js       ─ Search & filter state
      └── ui.js           ─ DOM manipulation helpers

        │ Tauri IPC (invoke / events)
        ▼

Backend (Rust)
  ├── lib.rs              — Tauri setup, commands, tray, clipboard polling
  ├── clipboard.rs        — Clipboard monitor state & dedup tracking
  ├── database.rs         — SQLite CRUD, search, FIFO cleanup
  ├── config.rs           — TOML config load/save
  ├── hotkey.rs           — Global shortcut parsing
  ├── private_mode.rs     — Argon2id password hash/verify
  ├── autostart.rs        — Registry (Win) / .desktop (Linux) autostart
  └── main.rs             — Entry point

Database: SQLite (history.db)
  └── entries table: id, content, content_type, preview, image_data,
      width, height, timestamp, is_pinned, is_private, source_app, name
```

### Key Design Decisions

- **No frontend framework** — Vanilla JS keeps the bundle at zero dependencies and minimal memory usage
- **Clipboard polling at 500ms** — Polling is necessary on Windows (clipboard APIs require a message pump); frontend-driven via `setInterval` calling a Tauri command
- **100-entry FIFO history** — Oldest unpinned entries are auto-deleted when the limit is exceeded
- **Images stored as raw RGBA** — Avoids re-encoding on copy/paste; on-demand PNG thumbnail generation for display
- **Argon2id for private mode** — Strong password hashing, no plaintext passwords stored

---

## Tech Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| **Framework** | Tauri v2 | ~15 MB RAM, no bundled Chromium |
| **Backend** | Rust | Memory-safe, direct OS API access |
| **Frontend** | Vanilla JS | Zero bundle overhead |
| **Database** | SQLite (rusqlite) | Single-file, zero-config |
| **Config** | TOML | Human-readable, Rust-native |
| **Clipboard** | arboard | Cross-platform clipboard access |
| **Encryption** | argon2 | Password hashing |

---

## Platform Support

| Feature | Windows | Linux (X11) | Linux (Wayland) |
|---------|---------|-------------|-----------------|
| Clipboard Monitor | ✅ | ✅ | ✅ |
| System Tray | ✅ | ✅ | ✅ |
| Global Hotkey | ✅ | ✅ | ⚠️ (via portal) |
| Autostart | ✅ | ✅ | ✅ |

---

## License

MIT
