window.theme = (() => {
  function applyTheme(config) {
    const root = document.documentElement;
    const t = config.theme;

    root.style.setProperty('--bg-primary', t.bg_primary);
    root.style.setProperty('--bg-secondary', t.bg_secondary);
    root.style.setProperty('--bg-card', t.bg_card);
    root.style.setProperty('--bg-modal', t.bg_modal);
    root.style.setProperty('--text-primary', t.text_primary);
    root.style.setProperty('--text-secondary', t.text_secondary);
    root.style.setProperty('--accent', t.accent);
    root.style.setProperty('--accent-hover', t.accent_hover);
    root.style.setProperty('--accent-subtle', t.accent_subtle);
    root.style.setProperty('--danger', t.danger);
    root.style.setProperty('--success', t.success);
    root.style.setProperty('--border', t.border);
    root.style.setProperty('--border-card', t.border_card);
    root.style.setProperty('--border-radius', t.border_radius);
    root.style.setProperty('--font-family', t.font_family);
    root.style.setProperty('--font-size', t.font_size);
  }

  async function load() {
    try {
      const config = await window.api.getConfig();
      applyTheme(config);
      return config;
    } catch (err) {
      console.error('Failed to load theme:', err);
      return null;
    }
  }

  return { load, applyTheme };
})();
