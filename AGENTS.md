# Ctrl+C — Agent Instructions

## Project

**Ctrl+C** is a cross-platform clipboard manager (system tray app). Stack: **Tauri v2** (Rust backend, Vanilla HTML/CSS/JS frontend, SQLite).

## Status

Pre-initialization — only `Docs/DESIGN.md` and `Docs/changelog.md` exist.

## Workflow

- Perform tasks **one at a time, in order** (per `Docs/DESIGN.md` §9 task list). Do not skip or combine tasks.
- After every task, update `Docs/changelog.md` with the format in `Docs/DESIGN.md` §10.
- Test/verify before moving to the next task. If a task fails, document the error in changelog.md and fix before proceeding.

## Architecture

- **Backend:** Rust modules in `src-tauri/src/` — `main.rs` (entry), `lib.rs` (Tauri setup), `clipboard.rs`, `database.rs`, `config.rs`, `hotkey.rs`, `private_mode.rs`, `autostart.rs`
- **Frontend:** `src/index.html` + `src/styles/*.css` + `src/js/*.js` — no framework, vanilla JS only
- **Database:** SQLite via `rusqlite` with bundled feature; schema in DESIGN.md §6
- **Config:** TOML file at `~/.config/ctrl-c/config.toml` (Linux) or `%APPDATA%/ctrl-c/config.toml` (Windows)
- **IPC:** Tauri `invoke` for commands, `listen` for events (e.g., `clipboard-changed`)

## Key Design Decisions

- Frontend is intentionally framework-free — no React, Vue, or bundler.
- Clipboard polling at 500ms (configurable) via `arboard` crate.
- 100-entry FIFO history; pinned entries exempt from eviction.
- Private mode pauses clipboard monitoring entirely (not just hides UI).
- Wayland global shortcuts need `xdg-desktop-portal` or manual DE config.

## Dependencies

At initialization (Task 1), `Cargo.toml` will need dependencies listed in DESIGN.md §11. `package.json` will need only `@tauri-apps/cli` as devDep. No frontend dependencies.

## Verification

- Build/run dev: `cargo tauri dev`
- Test: unit tests per Rust module (no test runner specified yet — check `Cargo.toml` when initialized)
