# Ctrl+C — Changelog

> This file is updated after every task/prompt. It tracks what changed, what didn't, and any errors encountered.

---

## [Release] — v0.2.0 Published (.deb + .tar.gz) — 2026-08-21

### ✅ What Changed
- Version bumped `0.1.0 → 0.2.0` in `tauri.conf.json` + `Cargo.toml` (PR #2); README download links updated to v0.2.0; auto-paste note rewritten for the new Wayland backend chain.
- Built release via `npm run tauri build`: `Ctrl+C_0.2.0_amd64.deb` (5.3 MB) from the deb bundle target.
- Assembled `Ctrl+C_0.2.0_amd64.tar.gz` (5.1 MB) replicating the v0.1.0 layout: `ctrl-c-v0.2.0/{ctrl-c, ctrl-c.png (512×512), Ctrl+C.desktop, install.sh, README.txt}` — install.sh echo updated to v0.2.0; README.txt documents runtime deps, optional auto-paste deps (xdotool/ydotool/wtype/uinput), and the Window Calls extension recommendation.
- Tagged `v0.2.0` on main (`0f42274`) and published [GitHub release](https://github.com/SlyHammad18/CtrlPlusC/releases/tag/v0.2.0) with both assets.

### ⏭️ What Was Not Changed
- Windows MSI not included (cannot cross-build on Linux without a cross-compilation setup).
- deb `Depends` left as-is (xdotool retained for X11 users; Wayland paste tools are optional runtime fallbacks).

### ❌ Errors Faced
- None — build clean in 2m50s, deb metadata verified (`Version: 0.2.0`, correct Depends).

### 📝 Notes
- Release notes drafted from the two Wayland-focus changelog entries above.

---

## [Cleanup] — Debug Overlay & Trace Logging Removed; Extension Mode Confirmed Working — 2026-08-21

### ✅ What Changed
- **Extension mode verified on this machine:** after a session restart, gnome-shell loaded `window-calls@domandoman.xyz`; the probe now resolves to `Extension` mode and focus save/restore/paste works end-to-end (user-confirmed).
- **Removed all paste-debug instrumentation** (was temporary diagnostics for the focus investigation):
  - `lib.rs`: deleted `debug_log` / `debug_log_window` helpers, all call sites in `show_window` / `hide_and_paste` / `copy_and_paste`, and the `log_active_window` / `log_mutter_focus` diagnostic functions. Removed setup eprintlns.
  - `paste.rs`: dropped the `log: &dyn Fn(&str)` callback from `simulate_paste` and all backends (`try_ydotool`, `try_ydotool_type`, `simulate_paste_uinput`, `ensure_virtual_keyboard`, `try_wtype`, `try_xdotool`). Error strings returned to the caller unchanged.
  - `wayland_focus.rs`: `save_current_focus()` / `restore_focus()` no longer take a `log` param; removed the now-dead `title` field from `WaylandTarget` (only used by logging).
  - Frontend: deleted the `#paste-debug` overlay div (`index.html`), its JS wiring incl. the `paste-debug` event listener (`app.js`), and all `.paste-debug*` styles (`cards.css`). Kept the `#focus-mode-hint` banner and the `paste-error` toast.
- **Fixed stale comment** in `lib.rs` (old iconify-era strategy note) — now documents the actual strategy: `gtk_widget_hide` unmap + WM-state verify + wayland_focus hybrid restore.

### ⏭️ What Was Not Changed
- Functional logic untouched: WM-state hide poll (`force_wm_hidden`), focus restore + verify loop, keystroke selection (`CtrlV`/`CtrlShiftV`/`ShiftInsert`), fallback chains.
- `#focus-mode-hint` banner kept — still useful for NoExtension-mode users.

### ❌ Errors Faced
- `cargo check` flagged `WaylandTarget.title` as dead code after removing the logging that read it — removed the field.
- `cargo test --lib`: 35/36 pass; the 1 failure remains the pre-existing environmental `hotkey::tests::test_wayland_not_set`. All JS passes `node --check`.

### 📝 Notes
- Build: `cargo check` clean, no warnings.

---

## [Feature] — Wayland Focus Fix v2: Hybrid Adaptive (window-calls Extension Route) — 2026-08-21

### ✅ What Changed
- **New module `src-tauri/src/wayland_focus.rs`** — the GNOME Shell `window-calls` extension route (Phase 2), plus the `accept_focus=FALSE` mouse-only fallback (Option C "hybrid adaptive"), plus Phase 3 (terminal paste keystroke).
  - `FocusMode` probe: runs `dbus-send --session --print-reply=literal --dest=org.gnome.Shell /org/gnome/Shell/Extensions/Windows org.gnome.Shell.Extensions.Windows.List`. Success → `Extension`; `UnknownMethod`/error → `NoExtension`. Cached in a `OnceLock` at startup.
  - `WaylandTarget { id, title, wm_class, wm_class_instance }` captured from `List()` (the extension's JSON includes a per-window `focus` field — no `Details` scan needed).
  - `save_current_focus()` — called in `show_window` before the picker takes focus.
  - `restore_focus()` — `Activate(<winid>)` (raise + focus the target, same as clicking it in the shell).
  - `verify_focus(id)` — re-runs `List()` and checks the saved `id` has `focus: true`; polled up to ~1s after restore.
  - Tolerant `parse_json` — `dbus-send --print-reply=literal` prints the raw JSON string (leading whitespace tolerated); 4 unit tests.
- **`lib.rs` integration:**
  - `show_window()` is now mode-aware: Extension → capture target + accept-focus ON (keyboard nav preserved); NoExtension → accept-focus OFF permanently (mouse-only, picker never takes focus → paste goes to whatever has focus, correct by construction); X11 → unchanged xdotool path.
  - `hide_and_paste()` Wayland branch rewritten: unmap via `gtk_widget_hide` (iconify proven a no-op in Phase 0a — dead code `force_minimize` removed) → Extension mode: `Activate` + verify-focus poll + log `verified=true/false`; NoExtension mode: 100ms settle, no restore needed. X11 path unchanged.
  - `setup()`: initializes the probe and pins `accept_focus(false)` for NoExtension Wayland.
  - New `get_focus_mode` Tauri command → `"x11" | "extension" | "no-focus"`.
- **`paste.rs` — Phase 3 (terminal-aware keystroke):**
  - New `PasteKey` enum: `CtrlV`, `CtrlShiftV`, `ShiftInsert`, each with ydotool keycodes / uinput sequence / wtype args / xdotool keysym.
  - `paste_key_for_target()` in `lib.rs`: Extension mode + terminal `wm_class` → `CtrlShiftV`; Extension + GUI → `CtrlV`; unknown target (NoExtension/X11) → `ShiftInsert` (universal).
  - `looks_like_terminal()` — matches gnome-terminal, kitty, alacritty, wezterm, foot, konsole, xterm, urxvt, tilix, terminator, ghostty, ptyxis, kgx, st-, etc.
  - `simulate_paste(text, key, log)` — all backends (ydotool, uinput, wtype, xdotool) now take the key.
- **Frontend:** `api.getFocusMode()`; `#focus-mode-hint` banner (amber) shown when mode is `no-focus`, telling the user to install "Window Calls" (e.g.o #4724) and re-login to restore keyboard navigation.
- **Extension installed on this machine:** `window-calls@domandoman.xyz` v21 (supports Shell 45–50; shell here is 48.7) downloaded from e.g.o and placed in `~/.local/share/gnome-shell/extensions/`. It is NOT yet loaded by the running gnome-shell (needs a session restart to appear in the extension manager), so this session is currently in `NoExtension` mode; it will auto-upgrade to `Extension` mode after re-login.

### ⏭️ What Was Not Changed
- X11 path (`restore_focus` via xdotool), Windows paste, `log_mutter_focus`/`log_active_window` diagnostics.
- `force_is_visible_wv` kept for tray/hotkey/socket toggles (show()/hide() flip the GTK visible flag — correct there).
- No new frontend dependencies (framework-free stack unchanged).

### ❌ Errors Faced
- `dbus-send` rejects `--dest NAME` (space form) and `--object-path` — it requires `--dest=NAME` (equals) and takes the object path as a positional arg. Caught by a manual invocation test before writing the module.
- `gnome-extensions install` succeeded but the running shell's extension manager did not pick up the new extension without a session restart (`UnknownMethod: Object does not exist at path` on `List`, `gnome-extensions list` doesn't show it). Documented; NoExtension mode handles this gracefully. No shell restart was attempted (would kill the user's session).
- `cargo test --lib`: 35/36 pass. The 1 failure is the pre-existing `hotkey::tests::test_wayland_not_set` (asserts `!is_wayland()` but this machine runs Wayland). All JS passes `node --check`.

### 📝 Notes
- Build: `cargo check` clean, no warnings.
- Verification matrix: (1) THIS session (NoExtension): picker should not take focus — select via mouse, paste should land in the target because the target never lost focus. Expect `no-extension mode -- no focus restore needed` in the log. (2) After re-login (Extension): `wayland_focus: captured target ...`, then `Activate(...)`, then `focus restored ... verified=true`, and paste lands in the target with the correct keystroke (`Ctrl+Shift+V` in terminals, `Ctrl+V` in GUI apps).
- `window-calls` D-Bus surface confirmed from the installed `extension.js`: `List() -> s` (JSON with per-window `focus`), `GetTitle(u) -> s`, `Activate(u) -> ()`; object path `/org/gnome/Shell/Extensions/Windows`, interface `org.gnome.Shell.Extensions.Windows`.

---

### ✅ What Changed
- **Root cause confirmed (0a):** The hide-verification poll used `gtk_widget_get_visible()`, which is a GTK-internal widget flag that `gtk_window_iconify()` never flips. Every Wayland paste therefore logged `STILL VISIBLE after 400ms` and fell through to `gtk_widget_hide()` — the iconify path was never actually verified.
- **New real WM-state check (`lib.rs`):**
  - `gdk_window_state_raw()` — reads `gdk_window_get_state()` from the cached raw `GdkWindow` (via `gtk_widget_get_window`).
  - `force_wm_hidden()` — window is hidden iff `GDK_WINDOW_STATE_WITHDRAWN | GDK_WINDOW_STATE_ICONIFIED` is set (WITHDRAWN=1, ICONIFIED=2, GDK3).
  - The `hide_and_paste` poll now verifies iconify against the compositor state and logs the raw bitmask + the `GDK_WINDOW_STATE_FOCUSED` (128) bit each 20ms iteration, so logs prove whether iconify actually unmaps and when Mutter moves focus away. On timeout it falls back to `gtk_widget_hide` and verifies again.
  - Final pre-paste check switched from `is_visible` to `!force_wm_hidden`.
- **Diagnostics made honest (0c):**
  - `log_mutter_focus` now detects `AccessDenied`/`UnknownMethod`/empty Eval results and logs that `org.gnome.Shell.Eval` is restricted on GNOME 41+ (Wayland focus info requires a Shell extension). No more misleading `mutter_focus: (false, '')`-style output.
  - `log_active_window` output annotated `(xwayland-only; does not reflect Wayland-native focus)` — xdotool only sees XWayland placeholders on Wayland.
- **Dynamic accept-focus (Phase 1, primary fix):** new `set_accept_focus(accept)` calls `gtk_window_set_accept_focus` + `gtk_window_set_focus_on_map` on the cached window.
  - `show_window()` enables accept-focus before showing, so the picker keeps keyboard focus for search / ArrowUp/Down / Enter / hotkey-recording while open.
  - `hide_and_paste()` disables accept-focus the instant a selection is made (before iconify), declaring the window non-focus-taking during dismissal so it doesn't fight Mutter's focus restoration.
  - Rationale: confirmed GNOME honors `accept_focus = FALSE` (sway ignores it — swaywm/sway#6368). This is the plan's keyboard-nav-preserving variant, chosen because this app has a real keyboard layer (search box, arrow nav, Enter-to-paste, hotkey recording) that plain always-off `accept_focus = FALSE` would break.
- **Removed dead code:** `force_is_visible` (Window variant, orphaned by the WM-state poll) and `force_minimize_wv` (never called).

### ⏭️ What Was Not Changed
- ydotool socket path (0b): already correct in `paste.rs` — uses `$YDOTOOL_SOCKET` → `$XDG_RUNTIME_DIR/.ydotool_socket` → `/tmp/.ydotool_socket`. No UID hardcode to fix.
- `force_is_visible_wv` kept — tray/hotkey/socket toggles use show()/hide() which DO flip the GTK visible flag, so it's correct there (only iconify doesn't).
- Paste fallback chain, X11 path (`restore_focus`), Windows paste — unchanged.
- Phase 2 (GNOME Shell extension route: `focused-window-dbus` + `activate-window-by-title`) NOT implemented — documented fallback if dynamic accept-focus + real WM-state verification prove insufficient against the Mutter MRU non-determinism bug.
- Phase 3 (terminal `Ctrl+Shift+V`, target window id/class logging) deferred — requires reliable `wm_class` which is blocked without a Shell extension on Wayland.

### ❌ Errors Faced
- `cargo check` initially flagged `force_is_visible` (orphaned by this change) and pre-existing `force_minimize_wv` as dead code — removed both, build is warning-free.
- `cargo test --lib`: 31/32 pass. The 1 failure is the pre-existing `hotkey::tests::test_wayland_not_set` (asserts `!is_wayland()` but this machine runs Wayland).
- All JS files pass `node --check`.

### 📝 Notes
- Build: `cargo check` clean, no warnings.
- Manual verification still pending: `npm run tauri dev` — open picker over a terminal, select an entry, confirm the log shows real `wm_state=0x... hidden=true` (ICONIFIED/WITHDRAWN) instead of the old `STILL VISIBLE` fallback, and that the paste lands in the terminal.
- Honest caveat (from research): setting `accept_focus = FALSE` at hide-time does not rewrite Mutter's MRU stack, so if Mutter still restores focus to the wrong window on current versions, the next step is the Phase 2 extension route.

---

## [Fix] — GTK Direct Hide: Bypass Broken Tauri hide() on Wayland — 2026-08-20

### ✅ What Changed
- **Root cause confirmed:** Debug trace from previous session proved Tauri v2.11.2's `WebviewWindow::hide()` is **broken on Wayland** — it returns `Ok()` without actually unmapping the GTK window. Polling fallbacks (minimize, off-screen move) also failed because they go through the same broken Tauri layer.
- **New approach: GTK direct hide.** Instead of calling Tauri's `window.hide()`, we now find the GTK toplevel window by title (`find_our_gtk_window()` — iterates `gtk::Window::list_toplevels()`) and call `gtk_win.hide()` / `gtk_win.is_visible()` / `gtk_win.iconify()` directly via `gtk::prelude::*`.
- **Cross-platform wrappers** (`force_hide`, `force_hide_wv`, `force_is_visible`, `force_is_visible_wv`, `force_minimize`, `force_minimize_wv`) — on Linux these use GTK directly; on other platforms they fall back to Tauri's API. Separate `_wv` variants for `WebviewWindow` (tray/hotkey handlers) and non-`_wv` for `Window` (command handlers).
- **All 6 `window.hide()` call sites** now use `force_hide` or `force_hide_wv` instead of raw Tauri `hide()`.
- **Removed `raw-window-handle` dependency** — raw-window-handle 0.6 dropped GTK support; using `gtk` crate directly is cleaner.
- **Added `gtk = "0.18"`** to Linux dependencies in `Cargo.toml`.

### ⏭️ What Was Not Changed
- `hide_and_paste` flow unchanged — it still calls `force_hide` → poll visibility → `simulate_paste`.
- `ignore_blur` handling unchanged.
- Paste simulation (ydotool/uinput/wtype) unchanged.

### ❌ Errors Faced
- `raw-window-handle` 0.6.2 removed the `GtkWindow`/`Gtk` variant — tried enabling a `gtk` feature that doesn't exist, then switched to using `gtk::Window::list_toplevels()` directly.
- GTK methods (`hide`, `is_visible`, `iconify`) require `use gtk::prelude::*` in each function scope.
- Build passes, 31/32 tests pass (1 pre-existing: `test_wayland_not_set` fails because `WAYLAND_DISPLAY` is set).

### 📝 Notes
- This is the definitive fix for the Wayland hide problem. Previous approaches (Tauri hide + verification + fallbacks) all failed because they still went through Tauri's broken API. Direct GTK calls bypass the issue entirely.
- Run `npm run tauri dev` and check the debug trace. The `pre-paste is_visible=` line should now show `false` (window actually hidden) instead of `true` as before.

---

## [Fix] — Window Hide Verification + ignore_blur During Paste — 2026-08-20

### ✅ What Changed
- **Root cause:** `window.hide()` was failing silently on Wayland/Tauri v2 — all calls used `let _ = window.hide()` which discarded the error. The Ctrl+C window stayed visible and focused, so ydotool keystrokes went into our own window instead of the target app. Additionally, the blur handler (`on_window_event Focused(false)`) could race with `hide_and_paste` because `ignore_blur` was never set during paste.
- **`hide_and_paste` rewritten with verification and fallbacks:**
  1. `window.hide()` result is now logged (Ok or error).
  2. Polls `window.is_visible()` every 20ms for up to 500ms to confirm the window actually hid.
  3. If still visible, tries `window.minimize()` as fallback.
  4. If still visible, moves the window off-screen as last resort.
  5. Final `is_visible()` check before calling `simulate_paste` — logs WARNING if window is still visible.
- **`ignore_blur` set during paste flow:** `copyById()` in `app.js` now calls `setIgnoreBlur(true)` before `copyAndPaste`/`copyImageAndPaste` and resets it in a `finally` block. This prevents the blur handler from racing with `hide_and_paste`.
- **Shift+Insert key combo + `ydotool type` fallback** (from previous entry) remain in place.

### ⏭️ What Was Not Changed
- Blur handler logic unchanged — still respects `ignore_blur` flag.
- Other `window.hide()` calls (tray icon, hotkey toggle) unchanged — they don't need the same verification.

### ❌ Errors Faced
- `window.set_visible(false)` does not exist in Tauri v2 — replaced with `minimize()` + off-screen move as fallbacks.
- Build passes, 31/32 tests pass.

### 📝 Notes
- Run `npm run tauri dev`, try paste, and check the debug trace for `hide_and_paste: hidden after Xms` (confirming hide worked) or `STILL VISIBLE after 500ms` (triggering fallbacks). The `pre-paste is_visible=` line tells you definitively whether the window was hidden before keystrokes were injected.

## [Fix] — Window Focus Restoration + ydotool Priority — 2026-08-20

### ✅ What Changed
- **Root cause:** On GNOME Wayland, after hiding the Ctrl+C window, focus was not restored to the previously-active window. Keystrokes from `simulate_paste()` went to the desktop (nowhere). Additionally, `xdotool windowactivate` was interfering with Mutter's natural focus restoration by sending spurious `_NET_ACTIVE_WINDOW` X11 messages through XWayland.
- **`lib.rs` — focus management:** Added `save_focus()` / `restore_focus()` functions (Linux only):
  - `save_focus()` runs `xdotool getactivewindow` and stores the window ID in a static `OnceLock<Mutex<Option<String>>>`. Called in `show_window()` before the Tauri window appears.
  - `restore_focus()` runs `xdotool windowactivate --sync <id>` to return focus to the saved window. **X11 only** — on Wayland this is skipped because xdotool cannot focus Wayland windows and the calls interfere with Mutter.
- **`hide_and_paste()` updated:** Platform-aware flow:
  - **Wayland:** `hide → 200ms delay (Mutter restores focus automatically on window unmap) → paste`
  - **X11:** `hide → xdotool windowactivate → 150ms delay → paste`
- **`paste.rs` — ydotool socket path fix:** `try_ydotool()` now checks `$YDOTOOL_SOCKET`, then `$XDG_RUNTIME_DIR/.ydotool_socket`, then `/tmp/.ydotool_socket` (was only checking the last one). On this system the socket is at `/run/user/1000/.ydotool_socket`.
- **`paste.rs` — Wayland priority swapped:** ydotool is now tried first on Wayland (before uinput), because ydotoold runs as root and creates a trusted virtual keyboard that Mutter accepts. uinput remains as fallback for wlroots compositors.

### ⏭️ What Was Not Changed
- `copy_and_paste` and `copy_image_and_paste` commands unchanged.
- Windows paste path unchanged.
- Image paste uses the same fallback chain as text paste.
- `save_focus()` / `restore_focus()` are no-ops on non-Linux (wrapped in `#[cfg(target_os = "linux")]`).

### ❌ Errors Faced
- `xdotool windowactivate --sync <id>` on Wayland produces `XGetWindowProperty[_NET_WM_DESKTOP] failed (code=1)` and sends spurious `_NET_ACTIVE_WINDOW` messages through XWayland that interfere with Mutter's focus tracking. Fixed by only running `restore_focus()` on X11.

### 📝 Notes
- 31/32 tests pass (1 pre-existing Wayland-env failure: `hotkey::tests::test_wayland_not_set`).
- On Wayland, external apps cannot focus other windows by design. Mutter handles focus restoration automatically when a window is unmapped (hidden). We just need to wait for the compositor to process the hide event.
- If Mutter's automatic focus restoration still doesn't work, the next step would be investigating `libei` via the XDG RemoteDesktop portal (used by `wdotool`), which is the proper Wayland-native input injection mechanism.

---

### ✅ What Changed
- **Root cause (final):** On GNOME Wayland, all three external paste tools fail:
  - `ydotool` — reports success (exit 0) but Mutter drops the simulated keystrokes (OpenWhispr #956, June 2026).
  - `wtype` — guaranteed to fail because GNOME does not implement the `virtual-keyboard-unstable-v1` protocol.
  - `xdotool` — returns exit 0 on Wayland with XWayland but silently fails for native Wayland windows.
- **New `src-tauri/src/paste.rs` module** — all paste simulation logic extracted from `lib.rs` into a dedicated module with a clean fallback chain:
  1. **Native uinput** (primary) — creates a virtual keyboard via `/dev/uinput` at the kernel level, bypassing the compositor entirely. Works on **all** Wayland compositors (GNOME, KDE, Hyprland, Sway) and X11. Uses `mouse-keyboard-input` crate (v0.9.1). Device is lazily initialized via `OnceLock<Mutex<VirtualDevice>>` — 200ms one-time cost at first paste, zero overhead after.
  2. `ydotool` (fallback) — with proper daemon socket verification (`/tmp/.ydotool_socket`) before attempting.
  3. `wtype` (fallback) — for wlroots compositors (Hyprland, Sway).
  4. `xdotool` (X11 fallback) — reliable on X11 sessions.
- **`lib.rs` simplified:** `simulate_paste()` is now a thin wrapper calling `paste::simulate_paste()`. The inline Windows `keybd_event` and Linux `xdotool`/`ydotool`/`wtype` code removed.
- **`Cargo.toml`:** Added `mouse-keyboard-input = "0.9.1"` under `[target.'cfg(target_os = "linux")'.dependencies]`.
- **`tauri.conf.json`:** Removed `wtype` from deb dependencies (no longer needed for the primary paste path). `xdotool` kept (used by `get_foreground_app()` on X11).
- **Frontend toast** updated: message prefix changed from "Copied, but auto-paste needs:" to "Copied, but paste failed:" (since backend now returns full setup instructions, not tool names). Toast duration raised to 8s.
- **User requirement:** must be in the `input` group for uinput access: `sudo usermod -aG input $USER && relogin`.

### ⏭️ What Was Not Changed
- `hide_and_paste()` at the time of this entry (later updated in focus-restoration fix).
- Windows paste path unchanged (uses `windows-sys` `keybd_event` directly).
- `copy_and_paste` and `copy_image_and_paste` commands unchanged — they call `hide_and_paste()` which calls `simulate_paste()`.
- Image paste uses the same fallback chain as text paste.

### ❌ Errors Faced
- `OnceLock::get_or_try_init` is unstable (`once_cell_try` feature gate). Workaround: manual init with `ensure_virtual_keyboard()` that checks `get().is_some()`, creates device, and uses `set()`.
- `mouse-keyboard-input`'s `VirtualDevice` methods require `&mut self`, so the device is wrapped in `Mutex<VirtualDevice>` inside the `OnceLock` for interior mutability.
- Build requires `libudev-dev` on the build machine (for `mouse-keyboard-input` crate's sys dependency).

### 📝 Notes
- 31/32 tests pass (1 pre-existing Wayland-env failure: `hotkey::tests::test_wayland_not_set`).
- The uinput approach is the same kernel-level injection used by CopyClip, GhostClip, and OpenWhispr v1.4.9+ — the most reliable paste method on modern Linux.
- `wtype` removed from deb hard deps to avoid pulling in packages that don't work on GNOME. Users on wlroots compositors can install it manually as a fallback.

---

## [Fix] — Auto-Paste Timing: Wait for Focus Loss Before Injecting — 2026-08-18

### ✅ What Changed
- **Root cause:** after `window.hide()`, the paste was injected before the compositor moved keyboard focus away from the Ctrl+C window, so Ctrl+V landed in our own (hidden-but-focused) window — nothing pasted into the target.
- The previous `is_visible()` poll was useless on Wayland: GTK flips `is_visible()` to `false` synchronously on `hide()`, before the compositor commits the hide.
- **`hide_and_paste()` now polls `window.is_focused()`** (Linux → `gtk_window_is_active()`, driven by real compositor focus events) until the window no longer holds keyboard focus — i.e. GNOME has restored focus to the previously active window. Bounded at 50 × 20ms (1s) so it can never hang; then a 50ms settle before `simulate_paste()`.
- ydotool key injection now uses `-d 30` (30ms between key events) for reliable combo recognition.

### ⏭️ What Was Not Changed
- Fallback chain (ydotool → wtype → toast) and X11 (`xdotool`) path unchanged. Image paste uses the same `hide_and_paste()`.

### ✅ Verification
- `ydotool key 30:1 30:0` (inject letter `a`) confirmed injection reaches the focused window.
- `ydotool key -d 30 29:1 47:1 47:0 29:0` confirmed clipboard + injection work outside the app.
- Debug + release binaries rebuilt; no `.deb` produced (per request).

---

## [Fix] — Auto-Paste via ydotool on GNOME Wayland — 2026-08-18

### ✅ What Changed
- **Root cause (final):** GNOME's Mutter deliberately does not implement the `virtual-keyboard-unstable-v1` protocol (security-motivated design decision). `wtype` therefore can never work on GNOME/KWin — only on wlroots compositors (Sway, Hyprland, ...). This is compositor-imposed, not a config/syntax issue.
- **`simulate_paste()` (Linux/Wayland) now uses a fallback chain:**
  1. `ydotool key 29:1 47:1 47:0 29:0` — injects via `/dev/uinput` at the kernel level, upstream of any compositor. Works on GNOME, KDE, Sway, everything. Requires the `ydotoold` daemon.
  2. `wtype -M ctrl v -m ctrl` — virtual-keyboard protocol, for wlroots-based compositors.
  3. If both fail, emits a toast telling the user to start the service: `systemctl --user enable --now ydotool`.
- X11 path unchanged (`xdotool key --clearmodifiers ctrl+v`).

### ⏭️ What Was Not Changed
- Deb `depends` keeps `wtype` + `xdotool` (both in trixie main). `ydotool` deliberately NOT added as a hard dependency — it only exists in `trixie-backports`, and a hard dep would break `apt install` for users without backports enabled.

### ❌ Errors Faced
- `ydotoold.service` does not exist; the Debian package names the user unit `ydotool.service` (ExecStart still runs `ydotoold`).
- User service fails with `failed to open uinput device: Permission denied` because the user is not in the `input` group (package udev rule `80-uinput.rules` grants `/dev/uinput` to group `input`, mode 0660). Fix: `sudo usermod -aG input $USER` + re-login (group membership applies at login; the systemd user manager inherits it).

### 📝 Notes
- Debug + release binaries rebuilt (`cargo build` / `cargo build --release`); no `.deb` bundle produced (per request).
- All running app instances were killed (`pkill -x ctrl-c`).

---

## [Fix] — wtype Syntax Correction for Auto-Paste — 2026-08-16

### ✅ What Changed
- **Root cause found:** `simulate_paste()` invoked `wtype -k ctrl+v`, but `-k` accepts a single key only. wtype fails with `Unknown key 'ctrl+v'`, so auto-paste never fired even with wtype installed.
- **Fixed to:** `wtype -M ctrl v -m ctrl` (press ctrl, type v, release ctrl) in `src-tauri/src/lib.rs`.
- Rebuilt release `.deb`.

### ⏭️ What Was Not Changed
- Nothing else; 180ms paste delay and single-instance guard from the previous entry remain.

### ❌ Errors Faced
- `wtype --help` / `wtype -h` error with "Missing argument" (option requires an arg); used `man wtype` to confirm flag semantics instead.

### 📝 Notes
- Verified against `man wtype` (`-M MOD` press, `-m MOD` release, `-k KEY` single key; modifiers auto-release at exit).

---

## [Fix] — Auto-Paste on Wayland + Deb Dependencies — 2026-08-16

### ✅ What Changed
- **Auto-paste root cause found:** two `ctrl-c` instances were running; the first (`/usr/bin/ctrl-c (deleted)`) was a stale process from before the `.deb` reinstall, still executing the old pre-`wtype` binary in memory. Its inode was deleted on reinstall but it kept running, so clicks hit the old X11-only paste path that fails silently. Killed all instances; only a fresh, new-code instance runs now.
- **Paste timing hardened:** the hide→Ctrl+V delay was raised from 50ms to 180ms (`copy_and_paste`, `copy_image_and_paste`) so GNOME/Wayland has time to return keyboard focus to the previously active window before `wtype -k ctrl+v` is sent.
- **Single-instance guard (Linux):** new `is_already_running()` connects to the toggle Unix socket at startup; if it connects, the second instance exits with "Ctrl+C is already running" instead of silently running alongside the first (prevents the reinstall/autostart stale-duplicate scenario). Guard runs before `start_socket_listener()`.
- **Deb now installs all runtime dependencies:** added `wtype` and `xdotool` to `bundle.linux.deb.depends` in `tauri.conf.json`. `apt install ./Ctrl+C_0.1.0_amd64.deb` now auto-installs the paste-simulation tools (no manual step).
- README already documents the auto-paste tool requirement; `.deb` dependency change supersedes the manual install note.

### ⏭️ What Was Not Changed
- Paste remains a single, well-timed attempt (no retry loop) to avoid double-pasting.
- `get_foreground_app()` on Linux still uses `xdotool` — app-name detection on Wayland remains best-effort (returns empty without it).

### ❌ Errors Faced
- `pkill -f 'ctrl-c'` matched the invoking shell's own command line and killed it (timeout); resolved by using `pkill -x ctrl-c` (exact process name).
- `sudo` install of the `.deb` cannot run non-interactively here (password required) — user installs it.

### 📝 Notes
- 31/32 lib tests pass (1 pre-existing Wayland-env failure `hotkey::tests::test_wayland_not_set`). Release `.deb` rebuilt with new depends.

---

## [Feature] — Unlimited History, Safe Clear-All, Wayland Auto-Paste — 2026-08-15

### ✅ What Changed
- **Unlimited history by default:** `behavior.max_entries` default is now `0` (keep everything). `Database` now stores `max_entries` (`AtomicI64`) and:
  - `add_entry_ext` → `cleanup()` skipped when `max_entries <= 0`
  - `get_entries` → `LIMIT` omitted when unlimited (was hardcoded `LIMIT 100`)
  - New `Database::set_max_entries()`, synced at startup and whenever Settings saves config
  - Settings panel gained a **History limit** field (`setting-max-entries`): `0` = unlimited, any number = cap. Wired in `app.js`.
  - Tests: `test_unlimited_max_entries`, `test_capped_max_entries`.
- **Clear-all now asks about pinned entries:** new `showClearAllDialog()` in `ui.js` (Cancel / "Clear all, keep pinned" / "Delete everything"). Backend `clear_all` command takes `keep_pinned`; `Database::clear_all(keep_pinned)` deletes only unpinned rows when requested. UI reloads afterwards so remaining pinned cards show. New confirm styles (`.confirm-keep`, `.confirm-hint`, `.confirm-actions-col`). Test: `test_clear_all_keeps_pinned`.
- **Auto-paste fixed for Linux/Wayland:** `simulate_paste()` now returns `Result` and:
  - **Wayland:** runs `wtype -k ctrl+v` (was X11-only `xdotool`, which failed silently on Wayland)
  - **X11:** runs `xdotool key --clearmodifiers ctrl+v`
  - On failure, `copy_and_paste` / `copy_image_and_paste` emit a `paste-error` event; frontend shows a toast ("Copied, but auto-paste needs: install wtype/xdotool") since the clipboard write already succeeded.
- README updated (unlimited history, `max_entries = 0` example, auto-paste/Wayland tool note).

### ⏭️ What Was Not Changed
- `get_foreground_app()` on Linux still uses `xdotool` (returns empty app name on Wayland without it) — separate concern, left as-is.
- Pre-existing `hotkey::tests::test_wayland_not_set` still expected-fails on this Wayland machine.

### ❌ Errors Faced
- `sudo apt-get install -y wtype` requires an interactive password; blocked in this environment. User must run it: `sudo apt install -y wtype`.
- Cargo test run initially rejected two `TESTNAME` filters in one invocation (CLI limitation) — reran with a single filter.

### 📝 Notes
- 31/32 lib tests pass (1 pre-existing Wayland-env failure). JS passes `node --check` on all files. Debug + release `.deb` rebuilt.

---

## [UI Overhaul] — Graphite Design System — 2026-08-15

### ✅ What Changed
- **Design system "Graphite":** new default palette (deep off-black `#0B0D12` base, elevated `#171B24` cards, single electric-blue accent `#4E8AFF`). All 13 theme tokens recalibrated in `src-tauri/src/config.rs`, `src/js/app.js` presets, and `README.md`.
- **Typography:** switched UI font default to **Geist** (fallback Inter) and added **JetBrains Mono** for all metadata (timestamps, app names, group labels, hotkeys, kbd hints, version, entry names). Fonts loaded via Google Fonts in `src/index.html`.
- **Window chrome:** enabled native drop shadow (`shadow: true`) in `tauri.conf.json`; `.app` now uses `border-radius: var(--border-radius)` with the transparent window for rounded corners.
- **Layout rebuild:** slim mono titlebar with inline close button; command-palette-style search bar with `Ctrl /` kbd hint + clear button; labeled **REC / PAUSED status pill** (replaced orphan green dot); footer now shows entry count on the left and "Clear all" + settings on the right.
- **Card redesign:** proper elevation (tinted shadows), more padding, pinned cards get a subtle accent tinted left bar + filled star (replaced clashing white stripe), action buttons always faintly visible with semantic hover colors, group dividers now include a hairline + count badge.
- **Signature micro-interaction: copy-flash** — clicking a card to copy fires an accent ring pulse and swaps the copy icon to a checkmark for 700ms (`ui.js flashCopied`, wired in `app.js copyById`).
- **Accessibility & motion:** global `:focus-visible` accent rings; all animations gated behind `prefers-reduced-motion`; `slideOut` rewritten to animate only transform/opacity (no `max-height`); skeleton loading state added; contrast raised for secondary text/placeholders.
- **Overlays unified:** settings/theme/edit/lock panels share one radius (`--border-radius`), scrim + backdrop blur, consistent headers/footers; theme editor buttons now use darker `accent_hover` so white labels pass 4.5:1.
- **Copy audit:** removed em-dash in empty-state copy; placeholder changed to "Search history".
- **Design tokens:** added non-themed CSS vars (`--font-mono`, `--radius-sm`, `--elev-1/2/3`, `--scrim`, `--z-*` scale, `--accent-rgb` derived in `theme.js`).
- **Icons:** standardized inline SVGs to 24px viewBox, stroke-width 1.8, round caps.
- **Themes refreshed:** all 6 presets recalibrated (no pure black, elevated surfaces, AA secondary text, darker hover accents); `Obsidian` default renamed **Graphite** (`btn-theme-reset` updated).

### ⏭️ What Was Not Changed
- No backend logic changes beyond config defaults (clipboard, database, hotkey, private mode untouched).
- Icon library not added — zero-dependency frontend architecture preserved; existing inline SVG set standardized instead.
- `src-tauri` `window.opacity` config is not applied to the actual window at runtime (pre-existing; left as-is).

### ❌ Errors Faced
- None during implementation.
- Pre-existing test `hotkey::tests::test_wayland_not_set` fails on this machine because the session is Wayland (asserts `!is_wayland()`); unrelated to this task. Config tests pass.

### 📝 Notes
- `cargo check` passes; `cargo test --lib config::tests` passes (4/4); all JS files pass `node --check`.
- Verification pending: dev build + screenshot review via vision model.

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

---

## [Task 6] — Frontend: Card List & Actions — 2026-05-24

### ✅ What Changed
- Updated `src-tauri/src/lib.rs`:
  - Added `copy_to_clipboard` Tauri command using `arboard::Clipboard::set_text()` — writes text to system clipboard via backend (registered in handler list)
- Updated `src/js/api.js`:
  - Added `copyToClipboard(text)` wrapper for the new backend command
- Updated `src/js/ui.js`:
  - Added `showToast(message)` — temporary bottom-center notification (2s auto-dismiss, fade animation)
  - Added `showConfirm(message)` — modal overlay with Cancel/Delete buttons, promise-based API, click-outside-to-cancel
- Updated `src/js/app.js`:
  - Copy action now calls `api.copyToClipboard()` (backend via arboard) instead of `navigator.clipboard.writeText()`
  - Card body click and Enter key both use shared `copyById()` helper
  - Delete button now shows confirmation dialog via `ui.showConfirm()` before removing
  - Keyboard Delete key also shows confirmation dialog
  - Toast notification on successful copy/deletion
  - `keydown` listener changed to async to support `await` in delete handler
- Updated `src/styles/cards.css`:
  - Added `.toast` and `.toast-visible` styles for notification popup
  - Added `.confirm-overlay`, `.confirm-box`, `.confirm-actions`, `.confirm-btn` styles for the modal confirmation dialog

### ✅ Tests
- All 13 Rust tests pass (unchanged)
- Build compiles clean, app launches without errors

### ⏭️ What Was Not Changed
- Pin action already worked: backend returns pinned-first sort, `loadEntries()` re-fetches after toggle — no changes needed
- `clipboard-changed` listener already worked — no changes needed
- Keyboard navigation already worked — only Delete key needed confirmation added

### ❌ Errors Faced (Fixed)
- **ES Module scoping bug**: `<script type="module">` scopes each file's top-level `const` declarations. `theme`, `api`, `search`, `ui` were invisible to `app.js` → `ReferenceError: theme is not defined`
  - **Fix**: All shared objects assigned to `window.*`; all cross-module references prefixed with `window.`
- **Clipboard never showed entries on Windows**: Background thread polling with `arboard` fails on Windows because `OpenClipboard` requires a thread with a Windows message pump. The `arboard` hidden window couldn't access clipboard data.
  - **Fix**: Replaced background thread architecture with frontend-side polling → `check_clipboard` Tauri command runs on main thread where clipboard APIs work

### 📝 Notes
- Copy action uses `arboard` via Tauri command on main thread (not `navigator.clipboard`)
- The `keydown` listener was made async — safe for keyboard events
- Clipboard architecture changed from push (background thread → events) to pull (frontend polls command every 500ms)
- `tauri-plugin-clipboard-manager` registered in lib.rs for future use

---

## [Task 6b] — Clipboard Architecture Fix & Debug — 2026-05-24

### ✅ What Changed
- **Rewrote `src-tauri/src/clipboard.rs`**:
  - Removed background thread + `capture()` function + `start_monitoring()`
  - Simplified to just `ClipboardMonitor` struct with `private_mode` (AtomicBool) and `last_content` (Mutex<Option<String>>) for shared state
- **Rewrote `src-tauri/src/lib.rs`**:
  - Added `check_clipboard` Tauri command: runs on main thread, creates `arboard::Clipboard`, reads text, compares with `monitor.last_content`, calls `db.add_entry()` if changed, returns `Option<Entry>`
  - Updated `set_private_mode` to also reset `last_content` when locking
  - Registered `tauri_plugin_clipboard_manager::init()` plugin
  - Removed `setup` closure (no more background thread to start)
  - Removed unused `Manager` import
- **Updated `src/js/api.js`**:
  - Added `__TAURI_INTERNALS__` fallback path for `invoke`
  - Added `checkClipboard()` wrapper
  - All module values assigned to `window.api`
- **Updated `src/js/theme.js`**:
  - Uses `window.api.getConfig()` (not bare `api`)
  - Assigned to `window.theme`
- **Updated `src/js/search.js`**:
  - Assigned to `window.search`
- **Updated `src/js/ui.js`**:
  - Uses `window.search.highlight()` and `window.search.getQuery()` everywhere
  - Added `showError(message)` — renders red error in empty state with Retry button
  - Added `setLoadEntries(fn)` — stores refresh callback for Retry button
  - Assigned to `window.ui`
- **Rewrote `src/js/app.js`**:
  - All references use `window.` prefix (`window.theme`, `window.api`, etc.)
  - Removed `clipboard-changed` event listener (no longer emitted)
  - Added `setInterval` polling `check_clipboard` every 500ms
  - Added 5-second full refresh interval as backup
  - Added Refresh button handler for `btn-refresh`
- **Updated `src/index.html`**:
  - Added "Refresh" button to empty state

### ✅ Tests
- All 13 Rust tests pass (unchanged)

### ⏭️ What Was Not Changed
- Database, config modules unchanged
- All existing Tauri commands preserved (add_entry, get_entries, delete_entry, toggle_pin, get_config, save_config, copy_to_clipboard)

### ❌ Errors Faced (Fixed in this task)
- `window.__TAURI__.core.invoke` undefined on some Tauri v2 configurations → fallback to `window.__TAURI_INTERNALS__.invoke` with correct args format

### 📝 Notes
- Windows clipboard API (`OpenClipboard`/`GetClipboardData`) requires the calling thread to have a Windows message queue. Creating `arboard::Clipboard` inside a `thread::spawn` creates a hidden HWND but the thread's sleep-loop never pumps messages, so clipboard reads fail silently.
- The fix decouples clipboard access from the polling thread: `check_clipboard` runs as a Tauri command on the main event-loop thread which has a proper message pump.
- Future optimization: cache the `arboard::Clipboard` instance in Tauri managed state instead of creating a new one per poll (but `Clipboard` is not `Send`, so it can't be shared across threads easily).

---

## [Task 7] — Search & Date Filter — 2026-05-24

### ✅ What Changed
- Modified `src/js/ui.js` — `renderCards()` now shows contextual empty state:
  - Active search query → "No results for '[query]'" with search icon
  - Active date filter (non-"All") → "No entries for this period" with calendar icon
  - No query/filter → default "No clipboard entries yet" (includes Refresh button)
  - Refresh button re-bound after innerHTML replacement (DOM listeners lost on innerHTML set)

### ✅ Tests
- All 13 Rust tests pass (unchanged)

### ⏭️ What Was Not Changed
- No backend Rust changes
- No HTML/CSS changes
- search.js, api.js, app.js unchanged (search/filter/highlight were already wired in Task 5)

### ❌ Errors Faced
- None

### 📝 Notes
- Most of Task 7 was already implemented in Tasks 2 and 5 (search debounce, backend wiring, highlighting, date filter tabs, combined queries). The only missing piece was the contextual "no results" state.

---

## [Task 8] — Private Mode (Lock/Unlock) — 2026-05-24

### ✅ What Changed
- **Created `src-tauri/src/private_mode.rs`** — Argon2id password hashing module:
  - `hash_password(password)` — generates salted Argon2id hash
  - `verify_password(password, hash)` — constant-time verification
  - 4 unit tests (hash/verify roundtrip, wrong password, empty hash, salt uniqueness)
- **Updated `src-tauri/src/config.rs`**:
  - Added `private_mode_locked: bool` field to `Config` struct (persists lock state across restarts)
- **Updated `src-tauri/src/lib.rs`**:
  - Added `mod private_mode` declaration
  - Added `PrivateModeStatus` struct (locked + has_password flags)
  - Added 4 Tauri commands:
    - `get_private_mode_status` — returns lock state and whether password is set
    - `set_private_mode_password` — hashes password, saves to config, locks immediately
    - `lock_private_mode` — sets locked flag in config + monitor, saves to disk
    - `unlock_private_mode` — verifies password, clears locked flag, resumes monitoring
  - Removed the old `set_private_mode` command (replaced by `lock_private_mode`/`unlock_private_mode`)
  - `run()` now checks `private_mode_locked` at startup and initializes monitor accordingly
- **Created `src/styles/lock.css`** — lock screen overlay styles (blur, centered layout, input, shake animation)
- **Updated `src/index.html`**:
  - Added `<link>` for `lock.css`
  - Added lock screen overlay HTML (lock icon, title, password input, unlock button, error message)
- **Updated `src/js/api.js`**:
  - Replaced `setPrivateMode` with `getPrivateModeStatus`, `setPrivateModePassword`, `lockPrivateMode`, `unlockPrivateMode`
- **Updated `src/js/ui.js`**:
  - Added `showLockScreen()` — shows overlay with unlock UI
  - Added `hideLockScreen()` — hides overlay
  - Added `showPasswordSetup()` — shows overlay with password creation UI
  - Added `lockShake()` — triggers shake animation on wrong password
  - Added `setLockError(msg)` — displays error text on lock screen
- **Updated `src/js/app.js`**:
  - Added `handleLockAction()` — checks status, either shows password setup or locks immediately
  - Added `handleUnlockOrSetPassword()` — either creates password or verifies unlock
  - Lock button click → lock or password setup
  - Lock screen button click → unlock or set password
  - Enter key on lock input → same as button click
  - On startup: checks if locked → shows lock screen immediately

### ✅ Tests
- All 17 Rust tests pass (13 existing + 4 new private_mode)

### ⏭️ What Was Not Changed
- database.rs, clipboard.rs unchanged
- No hotkey.rs, autostart.rs created yet

### ❌ Errors Faced
- None

### 📝 Notes
- First-time flow: user clicks lock → prompted to create password → password saved → immediately locked
- Subsequent flow: user clicks lock → immediately locked (no password prompt)
- Lock screen shows on app restart if was locked before
- Clipboard monitoring paused when locked (via existing `ClipboardMonitor.private_mode` flag)

---

## [Task 9] — System Tray — 2026-05-24

### ✅ What Changed
- **Updated `src-tauri/src/lib.rs`**:
  - Added `tauri::Emitter` import (needed for `emit()` on AppHandle)
  - Added `tauri::Manager` import (needed for `get_webview_window`, `state()` on AppHandle)
  - Added `.setup()` closure to `Tauri::Builder` that creates the system tray:
    - **Tray icon** — uses `app.default_window_icon()` (existing icons)
    - **Tooltip** — "Ctrl+C — Clipboard Manager"
    - **Menu items**: "Show/Hide" (toggle window), "Lock Private Mode" (locks from tray), "Quit" (exit app)
    - **Left-click on tray icon** — toggles window visibility (show/hide)
    - **Menu event handlers**:
      - `show_hide` — show or hide the main window
      - `lock_private` — locks private mode (pauses clipboard, saves config), emits `private-mode-locked` event to frontend
      - `quit` — exits the application
- **Updated `src/js/app.js`**:
  - Added `__TAURI__.event.listen('private-mode-locked', ...)` to show lock screen when locked from tray

### ✅ Tests
- All 17 Rust tests pass (unchanged)

### ⏭️ What Was Not Changed
- No new Rust modules created
- No hotkey.rs or autostart.rs yet

### ❌ Errors Faced
- Missing `use tauri::Emitter` import caused compile error (`emit` method from `Emitter` trait not in scope) — resolved by adding the import

### 📝 Notes
- Tray uses Tauri v2's built-in `tray-icon` feature (already in Cargo.toml)
- Icons already existed from Task 1 initialization
- Private mode from tray only locks; unlocking still requires frontend password entry
- Left-click on tray icon toggles window; right-click opens context menu

---

## [Task 10] — Global Hotkey — 2026-05-24

### ✅ What Changed
- **Created `src-tauri/src/hotkey.rs`** — global hotkey parsing module:
  - `parse_hotkey(hotkey_str)` — parses config hotkey string (e.g. "Ctrl+Shift+V") into `tauri_plugin_global_shortcut::Shortcut`
  - Supports modifiers: Ctrl, Alt, Shift, Super/Win/Cmd
  - Supports letter keys, digit keys, F1-F12, Space, Enter, Escape, Tab, navigation keys
  - `is_wayland()` — checks `WAYLAND_DISPLAY` env var for Wayland detection
  - 6 unit tests (parse common shortcuts, invalid key, missing key, Wayland detection)
- **Updated `src-tauri/src/lib.rs`**:
  - Added `mod hotkey;` declaration
  - Added import: `use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};`
  - Registered `tauri_plugin_global_shortcut::Builder` plugin with a handler that toggles window visibility on `ShortcutState::Pressed`
  - In `.setup()`: reads hotkey from config (`config.hotkey.toggle_window`), parses it, and registers via `app.global_shortcut().register()`
  - Wayland fallback: if `is_wayland()` returns true, logs a message about `xdg-desktop-portal` or manual DE keybind setup instead of registering

### ✅ Tests
- All 23 Rust tests pass (13 database + 4 config + 4 private_mode + 6 hotkey)

### ⏭️ What Was Not Changed
- No frontend changes (hotkey toggles window at OS level, no JS needed)
- No database or clipboard modules changed

### ❌ Errors Faced
- None

### 📝 Notes
- Default hotkey: `Ctrl+Shift+V` (from config)
- Hotkey is configurable via `config.toml` → `[hotkey] toggle_window`
- The handler toggles the main window's visibility (same as tray click)
- Wayland detection is passive — just logs a message, doesn't block startup

---

## [Task 11] — Autostart — 2026-05-24

### ✅ What Changed
- **Created `src-tauri/src/autostart.rs`** — platform-specific autostart module:
  - `enable_autostart()` — registers app to start on login
    - **Windows:** `reg add HKCU\...\Run` with current exe path
    - **Linux:** creates `~/.config/autostart/ctrl-c.desktop` file
  - `disable_autostart()` — removes autostart registration
    - **Windows:** `reg delete HKCU\...\Run` value
    - **Linux:** removes `.desktop` file
  - `is_autostart_enabled()` — checks if autostart is currently registered
    - **Windows:** `reg query` exit code
    - **Linux:** file existence check
  - Unsupported platforms return `Err("Autostart not supported on this platform")`
- **Updated `src-tauri/src/lib.rs`**:
  - Added `mod autostart;` declaration
  - Added 3 Tauri commands: `enable_autostart`, `disable_autostart`, `is_autostart_enabled`
  - In `run()`: checks `config.autostart` flag at startup and calls `autostart::enable_autostart()` if true (ensures registration on every launch)
- **Updated `src/js/api.js`**:
  - Added `enableAutostart()`, `disableAutostart()`, `isAutostartEnabled()` wrappers

### ✅ Tests
- All 23 Rust tests pass, zero warnings

### ⏭️ What Was Not Changed
- Settings UI not yet implemented (Task 12) — autostart toggle in settings will come next
- No frontend CSS/HTML changes

### ❌ Errors Faced
- None

### 📝 Notes
- Uses `std::process::Command` for Windows registry (no new crate dependencies)
- Config `autostart` field controls desired state; actual OS registration is applied at startup
- Settings UI toggle will call `save_config(autostart=true/false)` + `enable_autostart()/disable_autostart()`

---

## [Task 12] — Settings Panel — 2026-05-24

### ✅ What Changed
- **Created `src/styles/settings.css`** — settings modal styles:
  - Overlay with centered panel, header with close button
  - Rows with label/description + control (toggle, value badge, action button)
  - Custom toggle switch (accent-colored slider with circle knob)
  - About section at the bottom
- **Updated `src/index.html`**:
  - Added `<link>` for `settings.css`
  - Added settings modal overlay with:
    - **Autostart toggle** — toggle switch for start-on-login
    - **Global hotkey** — read-only display of current hotkey binding
    - **Private mode** — "Set Password" / "Change Password" button
    - **Reset theme** — button to restore default colors
    - **About section** — app name and version
- **Updated `src/js/ui.js`**:
  - Added `showSettings()` — shows settings modal overlay
  - Added `hideSettings()` — hides settings modal overlay
- **Updated `src/js/app.js`**:
  - Settings button click → loads current config, autostart status, private mode status → populates UI → shows modal
  - Autostart toggle `change` → calls `enableAutostart()`/`disableAutostart()` + saves to config
  - "Set Password" / "Change Password" → hides settings, opens password setup flow
  - "Reset theme" → clears theme config to defaults, reloads theme, shows toast
  - Close button and backdrop click → hides settings modal

### ✅ Tests
- All 23 Rust tests pass (unchanged)

### ⏭️ What Was Not Changed
- No backend Rust changes
- No database/clipboard/hotkey/autostart module changes

### ❌ Errors Faced
- None

### 📝 Notes
- Theme reset works by saving an empty theme object (`{}`) — `serde(default)` fills in defaults on next load
- Password setup redirects to the existing lock screen flow (reuses `showPasswordSetup`)
- Autostart toggle calls both OS registration + config save

---

## [Unplanned] — Titlebar, Start Hidden, Copy & Paste — 2026-05-24

### ✅ What Changed
- **Updated `src-tauri/tauri.conf.json`**:
  - Added `"decorations": false` — removes OS titlebar for custom titlebar
  - Added `"visible": false` — window starts hidden (app lives in tray)
- **Created custom titlebar in `src/index.html`**:
  - Drag region via `data-tauri-drag-region` attribute
  - Minimize button (SVG icon) → minimizes to taskbar
  - Close button (X icon, red hover) → hides window to tray
  - Removed `btn-date-filter` (duplicate with filter nav tabs below)
- **Updated `src/styles/main.css`**:
  - Added `.titlebar`, `.titlebar-title`, `.titlebar-actions`, `.titlebar-btn`, `.titlebar-btn-close` styles
- **Updated `src-tauri/Cargo.toml`**:
  - Added `windows-sys` dependency (Windows-only, for keyboard simulation)
- **Updated `src-tauri/src/lib.rs`**:
  - Added `copy_and_paste` Tauri command:
    - Copies text to system clipboard via `arboard`
    - Calls `simulate_paste()` which presses Ctrl+V:
      - **Windows:** `keybd_event` via `windows-sys` (VK_CONTROL + VK_V, key down/up with 15ms delay)
      - **Linux:** spawns `xdotool key ctrl+v`
      - **Other:** no-op
  - Added `simulate_paste()` platform-specific helper functions
  - Registered `copy_and_paste` command in invoke_handler
- **Updated `src/js/api.js`**:
  - Added `copyAndPaste(text)` wrapper
- **Updated `src/js/app.js`**:
  - `copyById()` now calls `copyAndPaste` instead of `copyToClipboard`
  - Close button (`btn-close`) → `appWindow.hide()`
  - Minimize button (`btn-minimize`) → `appWindow.minimize()`

### ✅ Tests
- All 23 Rust tests pass (unchanged)

### ⏭️ What Was Not Changed
- Tasks 13 and 14 postponed

### ❌ Errors Faced
- None

### 📝 Notes
- Window starts hidden; user clicks tray icon to show it
- Close button hides to tray (not closes); use tray menu "Quit" to exit
- Copy + paste does NOT add duplicate entries — duplicate detection in `check_clipboard` compares with `last_content` and skips matches
- `windows-sys` version 0.61 matches Tauri's dependency tree (no version conflict)

---

## [Unplanned] — Window Polish — 2026-05-24

### ✅ What Changed
- **Removed minimize button** from titlebar (`src/index.html` + `src/js/app.js`)
- **Updated `src-tauri/tauri.conf.json`**:
  - `"resizable": false` — window is fixed size
  - `"transparent": true` — no white background behind the app
  - `"shadow": false` — no window drop shadow
- **Updated `src/styles/main.css`**:
  - Removed `border-radius` from `.app` — eliminates border artifacts around the edge

### ✅ Tests
- All 23 Rust tests pass (unchanged)

### ❌ Errors Faced
- `windows-transparent` feature not available in Tauri 2.11.2 — removed it; `transparent: true` works without feature flag

### 📝 Notes
- Transparent window works via `transparent: true` in window config (no extra feature needed in Tauri v2)
- Removing `border-radius` ensures the app background fills the entire window edge-to-edge

---

## [Unplanned] — Theme Overhaul — 2026-05-24

### ✅ What Changed
- **Updated all 6 theme presets in `src/js/app.js`** — new color values for Void Purple, Synthwave, Midnight Ocean, Cyberpunk Terminal, Arctic Frost, Obsidian with darker/saturated backgrounds, improved contrast ratios, and 3 new fields per preset
- **Added 3 new CSS variables**: `--bg-modal`, `--border-card`, `--accent-subtle`
- **Updated `src/styles/main.css`**: new `:root` defaults, search input uses `--bg-card` + focus state border uses `--accent-subtle`
- **Updated `src/styles/cards.css`**: cards use `--border-card` (visible subtle border instead of transparent), hover uses `--accent-subtle`, search highlight uses `--accent-subtle`, confirm dialog uses `--bg-modal`
- **Updated `src/styles/settings.css`**: settings panel + theme panel use `--bg-modal` for overlay panels
- **Updated `src/styles/lock.css`**: lock overlay uses `--bg-modal`
- **Updated `src/js/theme.js`**: applies `--bg-modal`, `--border-card`, `--accent-subtle` from theme config
- **Updated `src-tauri/src/config.rs`**: added `bg_modal`, `border_card`, `accent_subtle` fields to `ThemeConfig` struct + serde defaults + Default impl + test assertions; updated Void Purple defaults with new values
- **Updated `src/js/app.js`**: `themeFields` now includes `bg_modal`, `accent_subtle`, `border_card` in the editor

### ✅ Tests
- All 24 Rust tests pass (unchanged count, updated assertions)

### ⏭️ What Was Not Changed
- No database, clipboard, private_mode, hotkey, autostart module changes
- No new HTML elements

### ❌ Errors Faced
- None

### 📝 Notes
- Void Purple is now `#0A0612` (was `#080611`), with distinct `--bg-modal: #160F28` for overlay panels
- `--border-card` gives every card a subtle visible border by default (was transparent)
- `--accent-subtle` used for hover borders, search highlights, and search focus states
- Arctic Frost retains its light theme identity with appropriate light-mode equivalents of new vars

---

## [Unplanned] — Default Theme Obsidian + Stop Recording Button — 2026-05-24

### ✅ What Changed
- **Default theme changed to Obsidian** (`src/styles/main.css` `:root`, `src-tauri/src/config.rs` serde defaults, test assertions)
- **Theme reset button** now resolves to Obsidian by name (`presets.find(p => p.name === 'Obsidian')`)
- **Added Stop Recording button** in header:
  - `src-tauri/src/clipboard.rs` — added `paused: Arc<AtomicBool>` to `ClipboardMonitor`
  - `src-tauri/src/lib.rs` — added `set_monitoring` Tauri command; `check_clipboard` now checks `paused` flag alongside `private_mode`
  - `src/js/api.js` — added `setMonitoring(active)` wrapper
  - `src/index.html` — stop/play toggle button with record dot indicator, dual SVG icons
  - `src/styles/main.css` — `.record-btn`, `.record-dot` styles with pulse animation when paused
  - `src/js/app.js` — toggle handler calls `setMonitoring`, swaps icons, toggles red dot pulse

### ✅ Tests
- All 24 Rust tests pass (unchanged)

### ⏭️ What Was Not Changed
- No database, private_mode, hotkey, autostart changes
- Existing private mode functionality unaffected

### ❌ Errors Faced
- None

### 📝 Notes
- Stop button is independent from private mode — just pauses clipboard polling, doesn't require password
- Green dot when recording, pulsing red dot when paused
- Clicking stop/resume is instant (no debounce needed)
- Paused state is in-memory only (not persisted across restarts)

---

## [Unplanned] — Image Clipboard Support (v3, raw RGBA storage) — 2026-05-25

### 🔴 Issue (v2 crash fix)
- Previous v2 approach stored PNG in DB and used `image` crate for encoding/decoding
- `copy_image_and_paste` crashed with `STATUS_ACCESS_VIOLATION (0xc0000005)` when calling `arboard::set_image()` after PNG→RGBA decode
- Root cause: `image` crate's internal FFI calls (libpng/zlib) caused a C-level memory crash during decode/encode

### ✅ What Changed (v3 Fix)
- **`Cargo.toml`** — Added `base64` (0.22) and `image` (0.25) crates
- **`src-tauri/src/database.rs`** — Major schema rework:
  - **Stores raw RGBA bytes** instead of PNG (`image_data` BLOB)
  - Added `width` and `height` columns (INTEGER)
  - `add_image_entry()` now takes `(raw_rgba, width, height, is_private)`
  - `get_entry_image_data()` returns `Option<(Vec<u8>, u32, u32)>` — raw RGBA + dimensions
  - Preview shows dimensions: `"Image 1920x1080"`
  - Migration adds `width`/`height` columns with `ALTER TABLE`
  - 27 tests all passing
  - New columns: `content_type TEXT NOT NULL DEFAULT 'text'`, `image_data BLOB`
  - New `Entry` field: `content_type` ('text' or 'image')
  - `add_image_entry()` — stores PNG bytes in DB with content_type='image'
  - `get_entry_image_data()` — retrieves raw PNG bytes for a given entry ID
  - Search queries now filter by `content_type = 'text'` (images excluded from text search)
  - Migration: `ALTER TABLE` with error handling for existing databases
  - 4 new tests (add image, image preview, search excludes images, nonexistent image)
- **`src-tauri/src/clipboard.rs`** — Added `last_image_hash: Arc<Mutex<Option<u64>>>` to `ClipboardMonitor` for image dedup; added `hash_bytes()` helper using `DefaultHasher`
- **`src-tauri/src/lib.rs`** — Image handling rewritten to avoid `image` crate crashes:
  - **`check_clipboard`** (sync, no async needed) — detects images via `clip.get_image()`, validates buffer size, hashes raw RGBA for dedup, stores raw RGBA + dimensions directly in DB. **No `image` crate calls in this path** — zero CPU overhead for polling.
  - **`get_entry_image`** — reads raw RGBA from DB, converts to PNG thumbnail (300×200 max) using `image` crate, returns base64 data URI. `image` crate used only here (on-demand, once per card render).
  - **`copy_image_and_paste`** — reads raw RGBA from DB, validates buffer size matches `w * h * 4`, passes directly to `arboard::set_image()`. **No `image` crate calls** — eliminates the crash risk entirely.
  - `clear_all`, `lock_private_mode`, `set_private_mode_password` all reset `last_image_hash` state
- **`src/js/api.js`** — Added `getEntryImage(id)` and `copyImageAndPaste(id)` wrappers
- **`src/js/ui.js`** — `createCard()` renders image cards with async thumbnail loading, label shows dimensions from preview
- **`src/styles/cards.css`** — Added image card styles: `.clip-image-wrap`, `.clip-image-thumb`, `.clip-image-label`
- **`src/js/app.js`** — `copyById()` detects image entries and calls `copyImageAndPaste` instead of text copy

### ⏭️ What Was Not Changed
- Config, hotkey, autostart, private_mode modules unchanged
- Theme editor, settings panel, lock screen unchanged

### ❌ Errors Faced
- `STATUS_ACCESS_VIOLATION (0xc0000005)` in `copy_image_and_paste` — caused by `image` crate's FFI (libpng/zlib) when decoding PNG→RGBA for `arboard::set_image()`. Fixed by storing raw RGBA directly and skipping the PNG decode in the paste path.
- Type mismatch: `Some("__image__")` vs `Option<&String>` — fixed with `as_deref()`
- `query_row` errors on no rows — fixed with manual `stmt.query() + rows.next()`

### 📝 Notes
- Images stored as **raw RGBA bytes** in SQLite BLOB (not PNG), with separate width/height columns
- `image` crate only used in `get_entry_image` (thumbnail generation) — isolated from all polling and copy paths
- Thumbnails (max 300×200) generated on-the-fly for frontend display
- Duplicate detection uses `DefaultHasher` hash of the raw RGBA bytes
- When copied from the app, image is marked with `last_app_copy = "__image__"` to prevent re-adding
- Search excludes image entries (only text entries are searchable)
- Size limit: images larger than 3840×2160 are silently skipped
- Buffer size validated before hashing and before `arboard::set_image()` (must equal `w * h * 4`)

---

## [Unplanned] — Custom Font Setting — 2026-05-25

### ✅ What Changed
- `src/index.html` — Added "Font" row in settings panel with a text input for custom font family
- `src/styles/settings.css` — Added `.settings-font-input` styles (monospace, themed, focus state)
- `src/js/app.js` — Added `font_family` to theme editor fields; font input populates on settings open; saves on blur/Enter and applies immediately via `--font-family` CSS variable

### ⏭️ What Was Not Changed
- No backend Rust changes (font_family was already in ThemeConfig)
- Existing theme.js handles applying the font

### ❌ Errors Faced
- None

### 📝 Notes
- Font change applies immediately when the user types and presses Enter or clicks away
- Also editable in the Theme customization panel

---

## [Unplanned] — App Icons from Assets Folder — 2026-05-25

### ✅ What Changed
- Copied `Assets/logo.ico` → `src-tauri/icons/icon.ico`
- Copied `Assets/logo.png` → `src-tauri/icons/32x32.png`
- Copied `Assets/logo.png` → `src-tauri/icons/128x128.png`
- Copied `Assets/logo.png` → `src-tauri/icons/128x128@2x.png`
- All app icons (bundle icons, tray icon, window icon) now use the project's `Assets/logo` files instead of generic placeholders

### ⏭️ What Was Not Changed
- `icon.icns` and `icon.svg` left as-is (no equivalents in Assets)
- `tauri.conf.json` icon paths unchanged (filenames match)

### ❌ Errors Faced
- None

---

## [Unplanned] — Wayland Toggle via Unix Socket IPC — 2026-05-24

### ✅ What Changed
- **Updated `src-tauri/src/main.rs`** — added CLI arg parsing: `ctrl-c toggle` connects to Unix socket and signals the running instance to toggle window visibility
- **Updated `src-tauri/src/lib.rs`**:
  - Added `APP_HANDLE` global `OnceLock<AppHandle>` for cross-thread access to the Tauri app handle
  - Added `get_socket_path()` — Linux-only, uses `$XDG_RUNTIME_DIR/ctrl-c/ctrl-c.sock` with fallback to `~/.cache/ctrl-c/ctrl-c.sock`
  - Added `send_toggle()` — connects to the socket, writes "toggle" message, exits cleanly
  - Added `start_socket_listener()` — background thread accepting Unix socket connections, toggles main window on "toggle" message
  - `run()` now calls `start_socket_listener()` before Tauri builder starts
  - Setup closure stores `AppHandle` in `APP_HANDLE`
  - Updated Wayland log message to show the actual binary path
- **Fixed `src-tauri/src/autostart.rs`** — added `#[cfg(target_os = "windows")]` guard on `get_app_name()` to silence dead_code warning on Linux

### ✅ Tests
- All 23 Rust tests pass (1 Wayland env test is expected-fail on Wayland)
- Toggle IPC verified manually: app starts, `ctrl-c toggle` returns exit 0, app processes toggle

### ⏭️ What Was Not Changed
- No changes to Windows/X11 hotkey behavior
- No frontend changes needed (toggle is OS-level)
- No new crate dependencies added

### ❌ Errors Faced
- None

### 📝 Notes
- Socket path uses `XDG_RUNTIME_DIR` (typically `/run/user/<uid>/`) for standard compliance, falls back to `~/.cache/`
- Stale socket file is removed on listener startup
- The DE keybind should run `/path/to/ctrl-c toggle` (the app prints the exact path at startup)

---

## [Unplanned] — Edit Copied Text Data — 2026-05-25

### ✅ What Changed
- **`src-tauri/src/database.rs`**:
  - Added `update_entry(id, content)` — updates `content` and `preview` for text entries only
  - Returns error if entry not found or if `content_type` is not `'text'` (images are read-only)
  - Preview auto-regenerated from new content (first 100 chars + "...")
- **`src-tauri/src/lib.rs`**:
  - Added `update_entry` Tauri command — proxies to `db.update_entry(id, content)`
  - Registered in `invoke_handler`
- **`src/js/api.js`**:
  - Added `updateEntry(id, content)` wrapper
- **`src/index.html`**:
  - Added edit overlay modal (`.edit-overlay`, `.edit-panel`) with textarea, Save/Cancel buttons, close icon
  - Backdrop click closes the overlay
- **`src/js/ui.js`**:
  - Added `showEdit(id, content)` — populates textarea with entry content, shows overlay, auto-focuses
  - Added `hideEdit()` — hides overlay
  - Exported both functions
- **`src/js/app.js`**:
  - Added edit button handler in card click delegation — fetches full entry content, calls `showEdit`
  - Save button handler — calls `api.updateEntry()`, refreshes list, shows toast
  - Cancel/close button handlers — call `hideEdit()`
- **`src/styles/settings.css`**:
  - Added edit overlay styles (`.edit-overlay`, `.edit-panel`, `.edit-textarea`, `.edit-footer`, etc.)
- **`src/styles/cards.css`**:
  - Added `.clip-action-btn.edit-btn:hover` style

### ✅ Tests
- All 30 Rust tests pass (27 existing + 3 new: `test_update_entry`, `test_update_entry_nonexistent`, `test_update_entry_image_rejected`)
- All 5 JS modules pass syntax check

### ⏭️ What Was Not Changed
- Image entries are not editable (update_entry rejects `content_type = 'image'`)
- No changes to clipboard polling, config, hotkey, autostart, private_mode
- No database migrations needed (existing schema supports it)

### ❌ Errors Faced
- None

### 📝 Notes
- Edit button only appears on text cards (not image cards)
- Content validation: empty/save trimmed content prevents empty saves
- Overlay respects the app's font family and theme

---

## [Unplanned] — Source App Tracking & Filter — 2026-05-25

### ✅ What Changed
- **`src-tauri/Cargo.toml`**: Added `Win32_UI_WindowsAndMessaging` feature to `windows-sys` for `GetForegroundWindow`/`GetWindowTextW`
- **`src-tauri/src/database.rs`**:
  - Added `source_app TEXT NOT NULL DEFAULT ''` column + migration
  - Added `source_app` field to `Entry` struct
  - Added `add_entry_with_app()` and `add_image_entry_with_app()` methods
  - Updated `add_entry_ext()` signature with `source_app` param
  - Updated `get_entries()` to filter by `source_app` (third param)
  - Added `get_app_names()` — returns distinct non-empty app names
  - All existing tests updated for new 3-param `get_entries` signature
- **`src-tauri/src/lib.rs`**:
  - Added `get_foreground_app()` helper (Windows: `GetForegroundWindow`+`GetWindowTextW`, Linux: `xdotool getactivewindow getwindowname`)
  - `check_clipboard` now captures `source_app` via `get_foreground_app()` for both text and images
  - Updated `get_entries` Tauri command with `source_app` filter param
  - Added `get_app_names` Tauri command
  - Registered `get_app_names` in invoke handler
- **`src/index.html`**:
  - Added filter icon button (funnel SVG) in header actions
  - Added collapsible filter panel below header with app list + Clear button
- **`src/styles/cards.css`**:
  - Added `.clip-source-app` style (small, dimmed text on each card)
  - Added `.filter-panel`, `.filter-app-btn`, `.filter-panel-clear` styles
- **`src/js/api.js`**: Added `getAppNames()`, updated `getEntries()` with `sourceApp` param
- **`src/js/search.js`**: Added `currentApp` state, `getAppFilter()`, `setAppFilter()` — triggers search refresh
- **`src/js/ui.js`**:
  - Cards show `source_app` label (if non-empty)
  - Added `showFilterPanel(appNames)` — renders app buttons, toggles on click, closes on selection
  - Added `hideFilterPanel()`
- **`src/js/app.js`**:
  - `loadEntries()` passes `sourceApp` from `search.getAppFilter()`
  - Filter icon click toggles panel, fetches app names from backend
  - Clear button resets app filter

### ✅ Tests
- All 29 Rust tests pass
- All 5 JS modules pass syntax check

### ⏭️ What Was Not Changed
- No changes to config, hotkey, autostart, private_mode modules
- No changes to clipboard polling interval or architecture
- Existing filters (date, search query) continue to work alongside app filter

### ❌ Errors Faced
- Windows compile error: `hwnd.0` is not a field on `*mut c_void` — fixed with `hwnd.is_null()`
- Test `test_search` had an extra assertion from replaceAll — removed

### 📝 Notes
- Source app is captured as the **foreground window title** at the moment of clipboard capture
- Empty app names (`''`) are excluded from the app list and filtering
- The app filter combines with date filter and search query (AND logic)
- On Wayland, `xdotool` may not be available; app name will be empty string
- Filter icon toggles the panel open/closed; selecting an app closes the panel and applies filter

---

## [Unplanned] — App Name Fix + Clear All Image Fix — 2026-05-25

### ✅ What Changed
- **`clear_all` (lib.rs)**: Now reads the current clipboard image and stores its hash (same as text handling), preventing the image from being re-added on next poll
- **`get_foreground_app()` (lib.rs)**:
  - **Windows**: Now gets the **executable file stem** (e.g., `chrome` from `C:\...\chrome.exe`) via `GetWindowThreadProcessId` → `OpenProcess` → `QueryFullProcessImageNameW` → `Path::file_stem()` instead of the full window title
  - **Linux**: Gets process name from `/proc/PID/comm` via `xdotool getactivewindow getwindowpid` instead of window title
- **`Cargo.toml`**: Added `Win32_System_Threading` and `Win32_Foundation` features to `windows-sys`

### ✅ Tests
- All 29 Rust tests pass
- All JS modules pass syntax check

### ⏭️ What Was Not Changed
- No database changes (source_app already exists)
- No frontend changes needed (source_app display and filter remain the same)

### ❌ Errors Faced
- None

### 📝 Notes
- Executable name is more stable than window title — "chrome" instead of "Some Video - YouTube - Google Chrome"
- On Windows, uses `PROCESS_QUERY_LIMITED_INFORMATION` (doesn't require extra privileges)
- On Linux, reads `/proc/PID/comm` which gives the kernel's process name (e.g., "chrome")

---

## [Unplanned] — App Name Formatting + Improved Filter UI — 2026-05-25

### ✅ What Changed
- **`src/js/ui.js`**: Added `APP_NAME_MAP` (70+ mappings from executable names to common display names like "chrome"→"Google Chrome", "code"→"Visual Studio Code") and `formatAppName()` function; card labels and filter buttons now show formatted names
- **Filter UI overhaul**:
  - **Badge**: Tiny dot on the filter toggle icon when a filter is active
  - **Chip**: Active filter shown as a persistent chip below the header with close button
  - **Dropdown panel**: Animated slide-down panel with search input, scrollable app list, close button
  - **Search within filter**: Text input filters the app list in real-time (by formatted name)
  - **List layout**: Vertical list (not pills) with hover highlights and active indicator dot
  - **Close handlers**: Close button, Escape key in search input
  - **`updateFilterBadge()`** called on every filter change to sync badge/chip visibility

### ✅ Tests
- All 29 Rust tests pass
- All JS modules pass syntax check

### ⏭️ What Was Not Changed
- Backend unchanged (no Rust changes this round)

### ❌ Errors Faced
- Invalid JS object key `1password:` — fixed with `'1password':`

### 📝 Notes
- `formatAppName()` runs in the frontend only; raw executable name stays in DB for correct filtering
- Unknown executables get first-letter capitalization (e.g., "myapp"→"Myapp")

---

## [Unplanned] — Image Naming & Search — 2026-05-25

### ✅ What Changed
- **`database.rs`**:
  - Added `name TEXT NOT NULL DEFAULT ''` column + migration
  - Added `name` field to `Entry` struct (in SELECT, INSERT, construction)
  - **Search overhaul**: query now matches `(content_type = 'text' AND content LIKE ?) OR name LIKE ?` — images with a matching name appear in search results
  - Added `set_entry_name(id, name)` method
- **`lib.rs`**: Added `set_entry_name` Tauri command + handler registration
- **`js/api.js`**: Added `setEntryName(id, name)` wrapper
- **`js/ui.js`**: Every card now shows a name row at the top:
  - Has name → displayed in bold (`has-name`)
  - No name → italic hint "Add name…" (`no-name`)
  - **Inline editing**: click the name area → turns into an input field
  - Enter saves, Escape cancels, blur saves
- **`styles/cards.css`**: Added `.clip-name`, `.clip-name.has-name`, `.clip-name.no-name`, `.clip-name-input` styles
- **`js/app.js`**: Added `startNameEdit()` — handles inline rename with Enter/blur save and Escape cancel

### ✅ Tests
- All 29 Rust tests pass
- All JS modules pass syntax check

### ⏭️ What Was Not Changed
- Existing search behavior unchanged for text entries without names
- Date filters, app filters, pinning etc. unaffected

### ❌ Errors Faced
- None

### 📝 Notes
- Names are **not** auto-generated — user must set them
- Search for text entries still matches content; images without names are excluded from search
- Once named, images appear in search results when the query matches their name
- Name editing is inline (no modal overlay) for speed

---

## [Bugfix] — Font Family Reset on Restart (#000000) — 2026-05-25

### ✅ What Changed
- **`src/js/app.js`**:
  - Removed `font_family` from `themeFields` array — the theme editor was creating a `<input type="color">` for it, which converted `"Inter, system-ui, sans-serif"` to `#000000` (invalid hex fallback)
  - Font family is now only editable via the dedicated text input in Settings (not the theme color editor)
  - Settings font input now checks for hex color values and falls back to default
- **`src/js/theme.js`**: `applyTheme()` now checks if `font_family` starts with `#` (hex color) and falls back to `'Inter, system-ui, sans-serif'` — fixes already-corrupted configs on restart

### ⏭️ What Was Not Changed
- No backend Rust changes
- No schema or migration changes

### ❌ Errors Faced
- None

### 📝 Notes
- Existing configs with `font_family = "#000000"` will auto-reset to default on next load
- The dedicated font input in Settings (Settings → Font) still works as before
- Theme presets were never setting font_family, so this was only triggered when opening the theme editor and clicking Save

---

## [Unplanned] — Names Restricted to Image Entries Only — 2026-05-25

### ✅ What Changed
- **`src/js/ui.js`**: Name element (`clip-name`) now only created and appended for image entries — text entries no longer show a name row
- **`src-tauri/src/database.rs`**: `set_entry_name()` now checks `content_type` and rejects text entries with `"Text entries cannot have names"` error
- **`Docs/changelog.md`**: Updated

### ⏭️ What Was Not Changed
- Existing image names preserved
- No schema changes
- No frontend JS changes beyond ui.js

### ❌ Errors Faced
- None

### 📝 Notes
- Backend also hardened to prevent setting names on text entries via direct IPC calls

---

## [Bugfix] — Unlock Re-adds Last Clipboard Entry — 2026-05-25

### ✅ What Changed
- **`src-tauri/src/lib.rs`** — `unlock_private_mode()` now reads the current clipboard content (text and image) and syncs it to `last_content` / `last_image_hash` before resuming monitoring. This prevents the next poll from treating it as a new entry.
  - Lock had already set `last_content = None`, but unlock never re-synced with the actual clipboard state

### ⏭️ What Was Not Changed
- No frontend changes
- No database schema changes

### ❌ Errors Faced
- None

### 📝 Notes
- Now uses the same pattern as `clear_all()` — reads clipboard after a state change and seeds the dedup cache

---

## [Unplanned] — Full Keyboard Navigation & Shortcuts — 2026-05-25

### ✅ What Changed
- **`src/styles/cards.css`** — Action buttons (copy/pin/edit/delete) now visible at 50% opacity on the selected card (not just on hover), making keyboard navigation usable
- **`src/js/app.js`** — Replaced the keydown handler with a comprehensive system:
  - **Escape** closes any open overlay (theme editor, edit, settings, filter panel) or hides to tray
  - **ArrowUp/Down** — navigate cards (existing, preserved)
  - **ArrowLeft/Right** on selected card — cycle focus through action buttons (copy → pin → edit → delete)
  - **Enter** on selected card — copy & paste (existing, preserved)
  - **Delete** on selected card — delete with confirmation (existing, preserved)
  - **P** on selected card — toggle pin/unpin
  - **E** on selected card — open edit overlay (text entries only)
  - **S** — focus search input
  - **R** — toggle recording (stop/start)
  - **F** — toggle filter panel
  - **L** — lock/private mode
  - **C** — clear all history (with confirmation)
  - **T** — hide window to tray
  - All shortcuts are blocked when an input/textarea is focused to avoid accidental triggers while typing

### ⏭️ What Was Not Changed
- No backend changes
- No HTML changes

### ❌ Errors Faced
- None

### 📝 Notes
- Action buttons become focusable via ArrowLeft/Right when a card is selected, using native `.focus()` on the `<button>` elements
- `selectedActionIndex` resets when navigating to a different card

---

## [Unplanned] — Ctrl+ Shortcuts & Search Arrow Navigation — 2026-05-25

### ✅ What Changed
- **`src/js/app.js`** — Rewired keyboard shortcuts to Ctrl+ combinations:
  - **`Ctrl+/`** — toggle search focus (focus if not focused, blur if focused)
  - **`Ctrl+L`** — lock private mode
  - **`Ctrl+R`** — toggle recording (stop/start)
  - **`Ctrl+I`** — toggle settings panel (open/close)
  - **`Ctrl+D`** — clear all history (with confirmation)
  - **`Ctrl+F`** — toggle filter panel
  - **ArrowDown in search** — moves focus to first card, blurs search input
  - Single-letter global shortcuts (S, R, F, L, C, T) removed in favor of Ctrl+ combos
  - Card-level shortcuts (P for pin, E for edit) kept as-is (no conflict)

### ⏭️ What Was Not Changed
- No backend changes
- No CSS/HTML changes

### ❌ Errors Faced
- None

### 📝 Notes
- Ctrl+ shortcuts work regardless of input focus (user can `Ctrl+L` while typing in search)
- `Ctrl+/` uses `e.key === '/'` which works across keyboard layouts for the `/` key
- `e.key.toLowerCase()` normalizes the Ctrl+ combo key check
- Also supports `metaKey` for macOS compatibility

---

## [Unplanned] — Change Password Requires Verification — 2026-05-25

### ✅ What Changed
- **`src/js/app.js`** — "Change Password" button now asks for the current password first:
  - `_changePasswordFlow` state variable tracks the two-step flow (`'verify'` → `'new'`)
  - First step: lock screen shows "Enter Current Password" with "Verify" button
  - After successful verification: switches to "Set New Password" screen
  - After setting: toast "Password changed", lock screen hidden
  - `_privateModeStatus` cached from the settings open handler for the button click handler
  - First-time "Set Password" flow unchanged

### ⏭️ What Was Not Changed
- No backend changes (reuses existing `unlockPrivateMode` + `setPrivateModePassword` commands)
- No CSS/HTML changes

### ❌ Errors Faced
- None

### 📝 Notes
- Wrong current password shows error + shake (same as unlock)
- If user closes settings and reopens, `_privateModeStatus` is refreshed

---

## [Unplanned] — Alt+V Selects First Entry — 2026-05-25

### ✅ What Changed
- **`src-tauri/src/lib.rs`** — All three show-window paths (global hotkey, tray icon click, tray menu "Show/Hide") now emit a `hotkey-show` Tauri event when the window is shown
- **`src/js/app.js`** — Listens for `hotkey-show` event and selects the first clipboard card (`selectedIndex = 0`), visually highlights it with the selected border

### ⏭️ What Was Not Changed
- No CSS/HTML changes
- No database or config changes

### ❌ Errors Faced
- None

### 📝 Notes
- Selecting the first card also calls `.focus()` on it so keyboard navigation (ArrowLeft/Right for action buttons) works immediately
- Works from global hotkey (Alt+V), tray icon click, and tray menu Show/Hide

---

## [Unplanned] — Wayland/Hyprland Hotkey Info — 2026-05-25

### ✅ What Changed
- **`src-tauri/src/lib.rs`**:
  - Startup now emits `wayland-hotkey-info` event with the toggle command path on Wayland
  - `register_hotkey` error message improved to show the command the user should bind
- **`src/js/app.js`**:
  - Listens for `wayland-hotkey-info` event → shows a 5-second toast with the command
  - Caches `__onWayland` and `__toggleCommand` for settings display
  - Settings hotkey row shows the full command path instead of "Alt+V" on Wayland
  - Hotkey click handler disabled on Wayland (no `SettingsHotkey` class, no listening mode)
- **`src/js/api.js`** — Unchanged (no new API needed, everything via events)

### ⏭️ What Was Not Changed
- Non-Wayland hotkey registration unchanged
- No database or config changes

### ❌ Errors Faced
- None

### 📝 Notes
- The toggle command is shown in settings as e.g., `/usr/bin/ctrl-c toggle` — user can copy and bind it in their Hyprland config
- Works for any Wayland compositor (not just Hyprland)

---

## [Unplanned] — MSI Installer Build — 2026-05-25

### ✅ What Changed
- Ran `npm run tauri build` — produced release binaries:
  - `src-tauri/target/release/bundle/msi/Ctrl+C_0.1.0_x64_en-US.msi`
  - `src-tauri/target/release/bundle/nsis/Ctrl+C_0.1.0_x64-setup.exe`
  - `src-tauri/target/release/ctrl-c.exe`

### ⏭️ What Was Not Changed
- No code changes

### ❌ Errors Faced
- None

### 📝 Notes
- MSI built with default WiX configuration
- Bundle identifier `com.ctrl-c.app` (warning about `.app` suffix is cosmetic on Windows)

---

## [Bugfix] — Edit Creates Duplicate Entry + MSI Rebuild — 2026-05-25

### ✅ What Changed
- **`src-tauri/src/lib.rs`** — `copy_and_paste()` now sets both `last_content` and `last_app_copy` BEFORE writing to the clipboard. Previously it wrote to the clipboard first, then set `last_app_copy`, creating a race window where the 500ms polling interval could detect the new clipboard content as a new entry before the dedup flag was set.
- Rebuilt MSI + NSIS installers

### ⏭️ What Was Not Changed
- `copy_to_clipboard()` already had the correct order (dedup flags first)
- `copy_image_and_paste()` already had the correct order
- No database or frontend changes

### ❌ Errors Faced
- None

### 📝 Notes
- Race window was ~microseconds but enough for the polling interval to fire and add a duplicate
- Fix is consistent with the pattern used in `copy_to_clipboard` and `copy_image_and_paste`

---

## [Cleanup] — Suppress `add_image_entry` Warning + Verify Hotkey Default — 2026-05-25

### ✅ What Changed
- **`src-tauri/src/database.rs`** — Added `#[cfg(test)]` to `add_image_entry()` since it's only used in unit tests. Eliminates the `dead_code` warning in release builds.
- Hotkey default confirmed as `Alt+V` in `config.rs` default, JS settings fallback, and hotkey listener fallback — no change needed.
- Rebuilt MSI + NSIS installers with zero warnings.

### ⏭️ What Was Not Changed
- No functional changes

### ❌ Errors Faced
- None

---

## [Bugfix] — Restart Dedup + Settings-Close — 2026-06-01

### ✅ What Changed
- **`src-tauri/src/lib.rs`** — Startup block now initializes `last_image_hash` with the current clipboard image (matching the existing text init). Prevents duplicate image entries after restart.
- **`src-tauri/src/clipboard.rs`** — Added `ignore_blur: Arc<AtomicBool>` to `ClipboardMonitor`.
- **`src-tauri/src/lib.rs`** — Added `set_ignore_blur` command + `Focused(false)` handler checks `ignore_blur`. Stops spurious blur events from hiding window during async overlay setup.
- **`src/js/api.js`** — Added `setIgnoreBlur()` wrapper.
- **`src/js/app.js`** — Settings click handler wraps async IPC with `setIgnoreBlur(true/false)`.

### ⏭️ What Was Not Changed
- No schema, config, or layout changes.
- Other overlays left as-is (no IPC before showing).

### ❌ Errors Faced
- None
