# Contributing to Ctrl+C

Thanks for your interest in improving **Ctrl+C**! This guide covers how to set up the project, the conventions we follow, and how to get your changes merged.

Ctrl+C is a cross-platform clipboard manager built with **Tauri v2** (Rust backend + vanilla JS frontend) and **SQLite**. It is licensed under the [MIT License](LICENSE).

## Code of Conduct

Be respectful and constructive. We want Ctrl+C to be a welcoming project for everyone. If you experience or witness unacceptable behavior, please report it to the maintainers. (A separate `CODE_OF_CONDUCT.md` can be added if the community grows.)

## Getting Started

### Prerequisites

- **Rust** (edition 2021) — https://rustup.rs
- **Node.js** (for the `@tauri-apps/cli` dev dependency)
- Platform-specific dependencies — see the [README "Building from Source"](README.md#building-from-source) section and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

### Local setup

```bash
git clone https://github.com/SlyHammad18/CtrlPlusC.git
cd Ctrl+C
npm install
```

### Run the dev build

```bash
npm run tauri dev
```

> **Note:** Use `npm run tauri dev`, **not** `cargo tauri dev` — the Tauri CLI is provided as an npm package in this repo.

## Development Workflow

1. **Fork** the repository and create your branch from `main`.
2. **Branch naming** uses a short type prefix:
   - `feat/short-description` — new features
   - `fix/short-description` — bug fixes
   - `chore/short-description` — maintenance, tooling, docs
   - `revert/short-description` — reverts
   - Use kebab-case for the description, e.g. `fix/scroll-performance`.
   - Stacked branches are allowed when one change depends on another (e.g. `feat/collapsible-groups-v2`).
3. **Keep changes focused** — one logical change per pull request. Small, reviewable PRs get merged faster.
4. **Stay in sync** — rebase onto the latest `main` before opening or merging your PR (`git fetch origin && git rebase origin/main`).

## Coding Standards

### Backend (Rust)

- Follow the module layout described in `AGENTS.md` / `Docs/DESIGN.md` (`lib.rs`, `clipboard.rs`, `database.rs`, `config.rs`, `hotkey.rs`, `private_mode.rs`, `autostart.rs`).
- Run `cargo fmt` and `cargo clippy` — **no formatting diffs and no Clippy warnings** before pushing.
- Persist data via SQLite (`rusqlite`, bundled). Schema/migration changes live in `database.rs`.
- Keep using the established crates (`arboard` for clipboard, `argon2` for private-mode hashing, etc.).

### Frontend (Vanilla JS/CSS)

- **No frameworks, no bundlers, no new runtime dependencies.** The frontend is intentionally plain HTML/CSS/JS to keep the bundle tiny.
- Theme everything through CSS custom properties (see `:root` in `src/styles/main.css`); do not hardcode colors.
- All Tauri IPC goes through the wrappers in `src/js/api.js` (`invoke` for commands, `listen` for events).
- Keep the modules in `src/js/` small and single-purpose (`app.js`, `ui.js`, `search.js`, `theme.js`).

### Commits & PR titles

- Write commits in the **imperative** mood, concise and descriptive (`add image lazy-loading`, `fix hover repaint on scroll`).
- Pull requests are **squash-merged** into `main`, and an auto-added `(#N)` reference is appended to the title. Because of this, **the PR title must be a clear, standalone summary** of the change.

## Testing & Verification

- **Rust:** `cargo test` runs the unit tests for each backend module. Add or update tests when you change behavior.
- **Frontend:** `npm run tauri dev` for manual verification. For UI changes, **include screenshots** in the PR description.
- Verify your change locally and ensure any CI checks pass **before** opening the PR.

## Documentation

- **User-facing changes** must update [`Docs/changelog.md`](Docs/changelog.md) using its existing format (✅ What Changed / ⏭️ What Was Not Changed / ❌ Errors Faced / 📝 Notes).
- If a feature or its usage changes, update [`README.md`](README.md) as well (features, shortcuts, configuration, build instructions).

## Opening a Pull Request

- Target the `main` branch.
- In the description, explain **what** changed and **why**, link any related issues, and attach screenshots for UI work.
- Note how you tested the change.
- Expect a review; address feedback, keep the branch up to date, and the PR will be squash-merged once approved.

## Reporting Bugs & Requesting Features

Please use [GitHub Issues](https://github.com/SlyHammad18/CtrlPlusC/issues). Include:

- Your OS and session type: **Windows**, **Linux (X11)**, or **Linux (Wayland)**.
- The app version (see Settings → About).
- Steps to reproduce, plus expected vs. actual behavior.
- Logs or screenshots where relevant.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
