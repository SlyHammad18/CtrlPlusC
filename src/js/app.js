(async () => {
  const config = await window.theme.load();

  async function loadEntries(query, filter) {
    try {
      const entries = await window.api.getEntries(query || null, filter !== 'all' ? filter : null);
      window.ui.renderCards(entries, query || '');
    } catch (err) {
      console.error('Failed to load entries:', err);
      window.ui.showError('Failed to load: ' + (err?.message || err));
    }
  }

  async function copyById(id) {
    try {
      const entries = await window.api.getEntries(null, null);
      const entry = entries.find((e) => e.id === id);
      if (entry) {
        await window.api.copyAndPaste(entry.content);
        window.ui.showToast('Copied to clipboard');
      }
    } catch (err) {
      console.error('Copy failed:', err);
    }
  }

  window.ui.setLoadEntries((q) => loadEntries(q, window.search.getFilter()));

  function onSearch(query, filter) {
    loadEntries(query, filter);
  }

  window.search.init(onSearch);

  document.getElementById('btn-refresh')?.addEventListener('click', () => {
    loadEntries(window.search.getQuery(), window.search.getFilter());
  });

  const cardList = document.getElementById('card-list');
  cardList.addEventListener('click', async (e) => {
    const card = e.target.closest('.clip-card');
    if (!card) return;

    const id = parseInt(card.dataset.id);
    const btn = e.target.closest('.clip-action-btn');

    if (!btn) {
      copyById(id);
      return;
    }

    if (btn.classList.contains('copy-btn')) {
      copyById(id);
    } else if (btn.classList.contains('pin-btn')) {
      try {
        await window.api.togglePin(id);
        window.ui.updatePinState(id, !card.classList.contains('pinned'));
        loadEntries(window.search.getQuery(), window.search.getFilter());
      } catch (err) {
        console.error('Toggle pin failed:', err);
      }
    } else if (btn.classList.contains('delete-btn')) {
      const confirmed = await window.ui.showConfirm('Delete this clipboard entry?');
      if (!confirmed) return;
      try {
        await window.api.deleteEntry(id);
        window.ui.removeCard(id);
        window.ui.showToast('Entry deleted');
      } catch (err) {
        console.error('Delete failed:', err);
      }
    }
  });

  let selectedIndex = -1;
  document.addEventListener('keydown', async (e) => {
    const cards = cardList.querySelectorAll('.clip-card');
    if (cards.length === 0) return;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, cards.length - 1);
      cards.forEach((c) => c.classList.remove('selected'));
      cards[selectedIndex].classList.add('selected');
      cards[selectedIndex].scrollIntoView({ block: 'nearest' });
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
      cards.forEach((c) => c.classList.remove('selected'));
      cards[selectedIndex].classList.add('selected');
      cards[selectedIndex].scrollIntoView({ block: 'nearest' });
    } else if (e.key === 'Enter' && selectedIndex >= 0) {
      e.preventDefault();
      copyById(parseInt(cards[selectedIndex].dataset.id));
    } else if (e.key === 'Delete' && selectedIndex >= 0) {
      e.preventDefault();
      const id = parseInt(cards[selectedIndex].dataset.id);
      const confirmed = await window.ui.showConfirm('Delete this clipboard entry?');
      if (!confirmed) return;
      try {
        await window.api.deleteEntry(id);
        window.ui.removeCard(id);
        window.ui.showToast('Entry deleted');
      } catch (err) {
        console.error('Delete failed:', err);
      }
    }
  });

  async function handleLockAction() {
    try {
      const status = await window.api.getPrivateModeStatus();
      if (!status.has_password) {
        window.ui.showPasswordSetup();
      } else {
        await window.api.lockPrivateMode();
        window.ui.showLockScreen();
      }
    } catch (err) {
      console.error('Lock failed:', err);
    }
  }

  async function handleUnlockOrSetPassword() {
    const input = document.getElementById('lock-input');
    const password = input ? input.value.trim() : '';
    if (!password) return;

    const title = document.getElementById('lock-title');
    if (title && title.textContent === 'Set Password') {
      try {
        await window.api.setPrivateModePassword(password);
        window.ui.showLockScreen();
        window.ui.showToast('Password set');
      } catch (err) {
        window.ui.setLockError(err?.message || 'Failed to set password');
        window.ui.lockShake();
      }
    } else {
      try {
        const ok = await window.api.unlockPrivateMode(password);
        if (ok) {
          window.ui.hideLockScreen();
          await loadEntries('', 'all');
        } else {
          window.ui.setLockError('Wrong password');
          window.ui.lockShake();
          if (input) input.value = '';
        }
      } catch (err) {
        window.ui.setLockError(err?.message || 'Unlock failed');
        window.ui.lockShake();
      }
    }
  }

  let monitoring = true;
  const btnStop = document.getElementById('btn-stop');
  const recordDot = document.getElementById('record-dot');
  const stopIcon = document.getElementById('stop-icon');
  const playIcon = document.getElementById('play-icon');

  btnStop?.addEventListener('click', async () => {
    monitoring = !monitoring;
    try {
      await window.api.setMonitoring(monitoring);
      btnStop.title = monitoring ? 'Stop recording' : 'Start recording';
      if (recordDot) recordDot.classList.toggle('paused', !monitoring);
      if (stopIcon) stopIcon.style.display = monitoring ? '' : 'none';
      if (playIcon) playIcon.style.display = monitoring ? 'none' : '';
    } catch (err) {
      monitoring = !monitoring;
      console.error('Toggle monitoring failed:', err);
    }
  });

  document.getElementById('btn-lock')?.addEventListener('click', handleLockAction);

  document.getElementById('lock-btn')?.addEventListener('click', handleUnlockOrSetPassword);

  document.getElementById('lock-input')?.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleUnlockOrSetPassword();
    }
  });

  document.getElementById('btn-settings')?.addEventListener('click', async () => {
    const cfg = await window.api.getConfig();
    document.getElementById('setting-hotkey').textContent = cfg.hotkey?.toggle_window || 'Alt+V';

    try {
      const enabled = await window.api.isAutostartEnabled();
      document.getElementById('setting-autostart').checked = enabled;
    } catch (_) { /* best effort */ }

    const status = await window.api.getPrivateModeStatus();
    const pwBtn = document.getElementById('btn-settings-password');
    pwBtn.textContent = status.has_password ? 'Change Password' : 'Set Password';

    window.ui.showSettings();
  });

  let listeningHotkey = false;
  const hotkeyEl = document.getElementById('setting-hotkey');
  let hotkeyHandler = null;

  hotkeyEl?.addEventListener('click', function () {
    if (listeningHotkey) return;
    listeningHotkey = true;
    this.classList.add('listening');
    this.textContent = 'Press keys...';

    hotkeyHandler = async (e) => {
      e.preventDefault();
      e.stopPropagation();

      const parts = [];
      if (e.ctrlKey) parts.push('Ctrl');
      if (e.altKey) parts.push('Alt');
      if (e.shiftKey) parts.push('Shift');
      if (e.metaKey) parts.push('Super');

      const key = e.key;
      if (['Control', 'Alt', 'Shift', 'Meta'].includes(key)) return;

      const keyMap = {
        'ArrowUp': 'ArrowUp', 'ArrowDown': 'ArrowDown', 'ArrowLeft': 'ArrowLeft', 'ArrowRight': 'ArrowRight',
        ' ': 'Space', 'Escape': 'Escape', 'Enter': 'Enter', 'Tab': 'Tab', 'Delete': 'Delete', 'Backspace': 'Backspace',
        'Insert': 'Insert', 'Home': 'Home', 'End': 'End', 'PageUp': 'PageUp', 'PageDown': 'PageDown',
      };
      const mapped = keyMap[key] || (key.length === 1 ? key.toUpperCase() : null);
      if (!mapped) return;

      parts.push(mapped);
      const hotkeyStr = parts.join('+');

      document.removeEventListener('keydown', hotkeyHandler);
      listeningHotkey = false;
      hotkeyEl.classList.remove('listening');

      try {
        const cfg = await window.api.getConfig();
        cfg.hotkey = cfg.hotkey || { toggle_window: 'Alt+V' };
        cfg.hotkey.toggle_window = hotkeyStr;
        await window.api.saveConfig(cfg);
        await window.api.registerHotkey(hotkeyStr);
        hotkeyEl.textContent = hotkeyStr;
        window.ui.showToast(`Hotkey set to ${hotkeyStr}`);
      } catch (err) {
        hotkeyEl.textContent = 'Error';
        setTimeout(async () => {
          const cfg2 = await window.api.getConfig();
          hotkeyEl.textContent = cfg2.hotkey?.toggle_window || 'Alt+V';
        }, 1500);
        window.ui.showToast('Failed to register hotkey');
        console.error('Hotkey change failed:', err);
      }
    };

    document.addEventListener('keydown', hotkeyHandler);
  });

  document.getElementById('btn-settings-close')?.addEventListener('click', () => {
    if (listeningHotkey) {
      listeningHotkey = false;
      document.getElementById('setting-hotkey').classList.remove('listening');
    }
    window.ui.hideSettings();
  });

  document.getElementById('settings-overlay')?.addEventListener('click', (e) => {
    if (e.target === document.getElementById('settings-overlay')) {
      window.ui.hideSettings();
    }
  });

  document.getElementById('setting-autostart')?.addEventListener('change', async (e) => {
    const enabled = e.target.checked;
    try {
      if (enabled) {
        await window.api.enableAutostart();
      } else {
        await window.api.disableAutostart();
      }
      const cfg = await window.api.getConfig();
      cfg.autostart = enabled;
      await window.api.saveConfig(cfg);
      window.ui.showToast(enabled ? 'Autostart enabled' : 'Autostart disabled');
    } catch (err) {
      e.target.checked = !enabled;
      window.ui.showToast('Failed to update autostart');
      console.error('Autostart toggle failed:', err);
    }
  });

  document.getElementById('btn-settings-password')?.addEventListener('click', () => {
    window.ui.hideSettings();
    window.ui.showPasswordSetup();
  });

  const themeFields = [
    { key: 'bg_primary', label: 'Background' },
    { key: 'bg_secondary', label: 'Surface' },
    { key: 'bg_card', label: 'Card' },
    { key: 'bg_modal', label: 'Modal' },
    { key: 'text_primary', label: 'Text' },
    { key: 'text_secondary', label: 'Text Dim' },
    { key: 'accent', label: 'Accent' },
    { key: 'accent_hover', label: 'Accent Hover' },
    { key: 'accent_subtle', label: 'Accent Subtle' },
    { key: 'danger', label: 'Danger' },
    { key: 'success', label: 'Success' },
    { key: 'border', label: 'Border' },
    { key: 'border_card', label: 'Card Border' },
  ];

  const presets = [
    {
      name: 'Void Purple',
      theme: {
        bg_primary: '#0A0612', bg_secondary: '#120D1F', bg_card: '#1C1530',
        text_primary: '#EDE9FE', text_secondary: '#9B8FC0',
        accent: '#7C3AED', accent_hover: '#6D28D9',
        danger: '#F87171', success: '#34D399', border: '#2A1F45',
        bg_modal: '#160F28', border_card: '#2E2250', accent_subtle: '#1E1040',
      },
    },
    {
      name: 'Synthwave',
      theme: {
        bg_primary: '#0F0817', bg_secondary: '#180E26', bg_card: '#221438',
        text_primary: '#FFD6EE', text_secondary: '#C07FA0',
        accent: '#FF2D9B', accent_hover: '#E0187F',
        danger: '#FF6B6B', success: '#3DFFC0', border: '#3A1848',
        bg_modal: '#1C1130', border_card: '#3D1C50', accent_subtle: '#2A0820',
      },
    },
    {
      name: 'Midnight Ocean',
      theme: {
        bg_primary: '#030B14', bg_secondary: '#091828', bg_card: '#102338',
        text_primary: '#E0F7FF', text_secondary: '#5B9AB8',
        accent: '#00D4FF', accent_hover: '#00AACF',
        danger: '#FF5F5F', success: '#00E5A0', border: '#0E2E44',
        bg_modal: '#0D1E30', border_card: '#133650', accent_subtle: '#002A40',
      },
    },
    {
      name: 'Cyberpunk Terminal',
      theme: {
        bg_primary: '#0A0A0A', bg_secondary: '#141414', bg_card: '#1C1C1C',
        text_primary: '#EAEAEA', text_secondary: '#707070',
        accent: '#39FF14', accent_hover: '#2ECC10',
        danger: '#FF4444', success: '#39FF14', border: '#252525',
        bg_modal: '#181818', border_card: '#2E2E2E', accent_subtle: '#0A2200',
      },
    },
    {
      name: 'Arctic Frost',
      theme: {
        bg_primary: '#EEF2F6', bg_secondary: '#FFFFFF', bg_card: '#FFFFFF',
        text_primary: '#1A2E3D', text_secondary: '#5A7A94',
        accent: '#0077CC', accent_hover: '#005FA3',
        danger: '#D93025', success: '#1A7F4B', border: '#D0DDE8',
        bg_modal: '#F5F8FB', border_card: '#C8D8E8', accent_subtle: '#E0EFFA',
      },
    },
    {
      name: 'Obsidian',
      theme: {
        bg_primary: '#0A0A0A', bg_secondary: '#141414', bg_card: '#1E1E1E',
        text_primary: '#E8E8E8', text_secondary: '#707070',
        accent: '#AAAAAA', accent_hover: '#CCCCCC',
        danger: '#E05555', success: '#55AA77', border: '#242424',
        bg_modal: '#181818', border_card: '#2C2C2C', accent_subtle: '#1A1A1A',
      },
    },
  ];

  function applyThemeToEditor(theme) {
    themeFields.forEach((field) => {
      const val = theme[field.key];
      if (!val) return;
      document.documentElement.style.setProperty(`--${field.key.replace(/_/g, '-')}`, val);
      const picker = document.getElementById(`theme-${field.key}`);
      if (picker) picker.value = val;
      const hex = picker?.nextElementSibling;
      if (hex) hex.value = val;
    });
  }

  function buildThemeEditor(cfg) {
    const presetsEl = document.getElementById('theme-presets');
    presetsEl.innerHTML = '';
    const root = document.documentElement;
    presets.forEach((p) => {
      const btn = document.createElement('button');
      btn.className = 'theme-preset-btn';
      btn.textContent = p.name;
      const match = Object.entries(p.theme).every(([k, v]) => {
        const cssVal = root.style.getPropertyValue(`--${k.replace(/_/g, '-')}`).trim();
        return cssVal && cssVal.toLowerCase() === v.toLowerCase();
      });
      if (match) btn.classList.add('active');
      btn.addEventListener('click', () => {
        presetsEl.querySelectorAll('.theme-preset-btn').forEach((b) => b.classList.remove('active'));
        btn.classList.add('active');
        applyThemeToEditor(p.theme);
      });
      presetsEl.appendChild(btn);
    });

    const body = document.getElementById('theme-editor-body');
    body.innerHTML = '';
    themeFields.forEach((field) => {
      const row = document.createElement('div');
      row.className = 'theme-field';

      const label = document.createElement('label');
      label.textContent = field.label;
      label.htmlFor = `theme-${field.key}`;

      const picker = document.createElement('input');
      picker.type = 'color';
      picker.id = `theme-${field.key}`;
      picker.value = cfg.theme[field.key] || '';

      const hex = document.createElement('input');
      hex.type = 'text';
      hex.className = 'theme-hex';
      hex.value = picker.value;

      function clearPresetHighlight() {
        document.querySelectorAll('.theme-preset-btn').forEach((b) => b.classList.remove('active'));
      }

      picker.addEventListener('input', () => {
        clearPresetHighlight();
        hex.value = picker.value;
        document.documentElement.style.setProperty(`--${field.key.replace(/_/g, '-')}`, picker.value);
      });

      hex.addEventListener('input', () => {
        if (/^#[0-9a-fA-F]{6}$/.test(hex.value)) {
          clearPresetHighlight();
          picker.value = hex.value;
          document.documentElement.style.setProperty(`--${field.key.replace(/_/g, '-')}`, hex.value);
        }
      });

      row.appendChild(label);
      row.appendChild(picker);
      row.appendChild(hex);
      body.appendChild(row);
    });
  }

  document.getElementById('btn-settings-theme')?.addEventListener('click', async () => {
    const cfg = await window.api.getConfig();
    buildThemeEditor(cfg);
    document.getElementById('theme-overlay').classList.add('visible');
  });

  document.getElementById('btn-theme-close')?.addEventListener('click', () => {
    document.getElementById('theme-overlay').classList.remove('visible');
    window.theme.load();
  });

  document.getElementById('theme-overlay')?.addEventListener('click', (e) => {
    if (e.target === document.getElementById('theme-overlay')) {
      document.getElementById('theme-overlay').classList.remove('visible');
      window.theme.load();
    }
  });

  document.getElementById('btn-theme-save')?.addEventListener('click', async () => {
    try {
      const cfg = await window.api.getConfig();
      themeFields.forEach((field) => {
        const el = document.getElementById(`theme-${field.key}`);
        if (el) cfg.theme[field.key] = el.value;
      });
      await window.api.saveConfig(cfg);
      document.getElementById('theme-overlay').classList.remove('visible');
      window.ui.showToast('Theme saved');
    } catch (err) {
      window.ui.showToast('Failed to save theme');
      console.error('Theme save failed:', err);
    }
  });

  document.getElementById('btn-theme-reset')?.addEventListener('click', async () => {
    try {
      const obsidian = presets.find(p => p.name === 'Obsidian').theme;
      applyThemeToEditor(obsidian);
      window.ui.showToast('Theme reset to defaults');
    } catch (err) {
      console.error('Theme reset failed:', err);
    }
  });

  const appWindow = window.__TAURI__?.window?.getCurrentWindow?.();

  document.getElementById('btn-close')?.addEventListener('click', () => {
    if (appWindow) appWindow.hide();
  });

  document.getElementById('btn-clear-all')?.addEventListener('click', async () => {
    const confirmed = await window.ui.showConfirm('Delete all clipboard history?');
    if (!confirmed) return;
    try {
      await window.api.clearAll();
      window.ui.renderCards([], '');
      window.ui.showToast('History cleared');
    } catch (err) {
      window.ui.showToast('Failed to clear history');
      console.error('Clear all failed:', err);
    }
  });

  await loadEntries('', 'all');

  if (window.__TAURI__?.event?.listen) {
    window.__TAURI__.event.listen('private-mode-locked', () => {
      window.ui.showLockScreen();
    });
  }

  try {
    const status = await window.api.getPrivateModeStatus();
    if (status.locked) {
      window.ui.showLockScreen();
    }
  } catch (err) {
    console.error('Failed to check lock status:', err);
  }

  setInterval(async () => {
    try {
      const entry = await window.api.checkClipboard();
      if (entry) {
        window.ui.prependCard(entry);
      }
    } catch (err) {
      console.error('Clipboard check failed:', err);
    }
  }, 500);


})();
