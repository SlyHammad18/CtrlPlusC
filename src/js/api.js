window.api = (() => {
  let invoke;

  if (window.__TAURI__?.core?.invoke) {
    invoke = window.__TAURI__.core.invoke;
  } else if (window.__TAURI_INTERNALS__?.invoke) {
    const rawInvoke = window.__TAURI_INTERNALS__.invoke;
    invoke = (cmd, args) =>
      rawInvoke(cmd, {
        ...args,
        __tauri_module: null,
        __tauri_command: cmd,
      });
  } else {
    console.error('Ctrl+C: No Tauri IPC available');
    const es = document.getElementById('empty-state');
    if (es) es.innerHTML = '<p style="color:var(--danger)">IPC not available</p>';
  }

  if (!invoke) return {};

  return {
    addEntry: (content, isPrivate = false) =>
      invoke('add_entry', { content, isPrivate }),

    getEntries: (query = null, dateFilter = null) =>
      invoke('get_entries', { query, dateFilter }),

    deleteEntry: (id) =>
      invoke('delete_entry', { id }),

    togglePin: (id) =>
      invoke('toggle_pin', { id }),

    getConfig: () =>
      invoke('get_config'),

    saveConfig: (config) =>
      invoke('save_config', { config }),

    getPrivateModeStatus: () =>
      invoke('get_private_mode_status'),

    setPrivateModePassword: (password) =>
      invoke('set_private_mode_password', { password }),

    lockPrivateMode: () =>
      invoke('lock_private_mode'),

    unlockPrivateMode: (password) =>
      invoke('unlock_private_mode', { password }),

    copyAndPaste: (text) =>
      invoke('copy_and_paste', { text }),

    copyToClipboard: (text) =>
      invoke('copy_to_clipboard', { text }),

    checkClipboard: () =>
      invoke('check_clipboard'),

    enableAutostart: () =>
      invoke('enable_autostart'),

    disableAutostart: () =>
      invoke('disable_autostart'),

    isAutostartEnabled: () =>
      invoke('is_autostart_enabled'),

    registerHotkey: (hotkeyStr) =>
      invoke('register_hotkey', { hotkeyStr }),
  };
})();
