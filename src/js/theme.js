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
    const font = t.font_family && !t.font_family.startsWith('#') ? t.font_family : 'Geist, Inter, system-ui, sans-serif';
    root.style.setProperty('--font-family', font);
    root.style.setProperty('--font-size', t.font_size);

    const accent = t.accent;
    if (accent && /^#[0-9a-fA-F]{6}$/.test(accent)) {
      const r = parseInt(accent.slice(1, 3), 16);
      const g = parseInt(accent.slice(3, 5), 16);
      const b = parseInt(accent.slice(5, 7), 16);
      root.style.setProperty('--accent-rgb', `${r} ${g} ${b}`);
    }
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
