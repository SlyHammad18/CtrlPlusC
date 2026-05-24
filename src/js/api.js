const api = (() => {
  const invoke = window.__TAURI__?.core?.invoke;

  if (!invoke) {
    console.error('Ctrl+C: Tauri IPC not available');
    return {};
  }

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

    setPrivateMode: (locked) =>
      invoke('set_private_mode', { locked }),
  };
})();
