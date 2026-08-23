<p align="center">
  <img src="Assets/logo.png" alt="Ctrl+C Logo" width="96" />
</p>

<h1 align="center">Ctrl+C — Clipboard Manager</h1>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/Tauri-FFC131?style=for-the-badge&logo=tauri&logoColor=black" alt="Tauri" />
  <img src="https://img.shields.io/badge/JavaScript-F7DF1E?style=for-the-badge&logo=javascript&logoColor=black" alt="JavaScript" />
  <img src="https://img.shields.io/badge/SQLite-003B57?style=for-the-badge&logo=sqlite&logoColor=white" alt="SQLite" />
  <img src="https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white" alt="Windows" />
  <img src="https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black" alt="Linux" />
</p>

<p align="center">
  <a href="https://github.com/SlyHammad18/CtrlPlusC/releases/tag/v0.2.0">
    <img src="https://img.shields.io/github/v/release/SlyHammad18/CtrlPlusC?style=for-the-badge&logo=github&label=Download" alt="Download" />
  </a>
</p>

**Ctrl+C** is a lightweight, privacy-first clipboard manager that lives in the system tray. It automatically captures clipboard history, supports search and filtering, pinning, private mode with password lock, and is fully themeable.

Built with **Tauri v2** (Rust backend + Vanilla JS frontend) for a tiny memory footprint (~15 MB).

---

## Features

- **Clipboard History** — Automatically captures text and images from the clipboard (unlimited by default; optional cap in Settings)
- **Search & Filter** — Real-time text search with highlighting, date filters (Today, Yesterday, 7d, 30d), and app source filtering
- **Pin Entries** — Keep important items pinned — they are excluded from auto-eviction
- **Private Mode** — Password-protect your clipboard with Argon2id hashing. When locked, clipboard monitoring pauses entirely
- **Custom Themes** — 6 built-in presets (Graphite, Void Purple, Synthwave, Midnight Ocean, Cyberpunk Terminal, Arctic Frost) or create your own with the color picker
- **Global Hotkey** — Toggle the window from any app (default: `Alt+V`, configurable)
- **System Tray** — Minimize to tray with Show/Hide, Lock Private Mode, and Quit menu
- **Autostart** — Launch on login (toggle from Settings)
- **Keyboard Navigation** — Arrow keys, Enter to copy, Delete to remove, keyboard shortcuts for all actions
- **Source App Tracking** — See which application each clipboard entry came from, filter by app

---

## Installation

### Windows

Download the [latest MSI installer](https://github.com/SlyHammad18/CtrlPlusC/releases/tag/v0.2.0) and run it.

```
Ctrl+C_0.2.0_x64_en-US.msi
```

The app requires **WebView2** (pre-installed on Windows 10+).

### Linux

Download the `.deb` or `.tar.gz` from the [latest release](https://github.com/SlyHammad18/CtrlPlusC/releases/tag/v0.2.0).

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
>
> **Auto-paste note:** Auto-paste simulates a paste keystroke after copying. On X11 it uses `xdotool`; on Wayland it tries `ydotool`, then native uinput (requires the `input` group), then `wtype`. In terminals it sends `Ctrl+Shift+V` automatically. If every method is unavailable, the entry is still copied to the clipboard and a toast explains what to install.

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
bg_primary = "#0B0D12"
bg_secondary = "#12151D"
bg_card = "#171B24"
bg_modal = "#141822"
text_primary = "#F1F4F9"
text_secondary = "#9AA4B2"
accent = "#4E8AFF"
accent_hover = "#3D72E8"
accent_subtle = "#182A4D"
danger = "#E5484D"
success = "#2FB58A"
border = "#1F2430"
border_card = "#262D3B"
border_radius = "10px"
font_family = "Geist, Inter, system-ui, sans-serif"
font_size = "14px"

[window]
width = 420
height = 600
opacity = 1.0

[behavior]
max_entries = 0
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
git clone https://github.com/SlyHammad18/CtrlPlusC.git
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
- **Unlimited history by default** — `max_entries = 0` keeps everything; set a cap in Settings to auto-evict oldest unpinned entries
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

[MIT](LICENSE)

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, branching conventions, coding standards, and pull-request guidelines.
