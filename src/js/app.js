(async () => {
  const config = await window.theme.load();

  if (config && config.ui && Array.isArray(config.ui.collapsed_sections)) {
    window.ui.setCollapsedSections(config.ui.collapsed_sections);
  }

  let collapsedSaveTimer = null;
  window.ui.setCollapsePersist((sections) => {
    clearTimeout(collapsedSaveTimer);
    collapsedSaveTimer = setTimeout(async () => {
      try {
        const cfg = await window.api.getConfig();
        cfg.ui = { collapsed_sections: sections };
        await window.api.saveConfig(cfg);
      } catch (err) {
        console.error('Failed to save section state:', err);
      }
    }, 300);
  });

  async function loadEntries(query, filter) {
    try {
      const app = window.search.getAppFilter();
      const entries = await window.api.getEntries(
        query || null,
        filter !== 'all' ? filter : null,
        app || null
      );
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
      if (!entry) return;
      // Prevent the blur handler from hiding the window before hide_and_paste does.
      await window.api.setIgnoreBlur(true);
      try {
        if (entry.content_type === 'image') {
          await window.api.copyImageAndPaste(id);
          window.ui.flashCopied(id);
          window.ui.showToast('Image copied to clipboard');
        } else {
          await window.api.copyAndPaste(entry.content);
          window.ui.flashCopied(id);
          window.ui.showToast('Copied to clipboard');
        }
      } finally {
        await window.api.setIgnoreBlur(false);
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
  const origSetAppFilter = window.search.setAppFilter;
  window.search.setAppFilter = function(app) {
    origSetAppFilter(app);
    window.ui.updateFilterBadge();
  };

  function startNameEdit(nameEl) {
    if (nameEl.querySelector('.clip-name-input')) return;
    const id = parseInt(nameEl.dataset.id);
    const current = nameEl.textContent === 'Add name…' ? '' : nameEl.textContent;
    const input = document.createElement('input');
    input.type = 'text';
    input.className = 'clip-name-input';
    input.value = current;
    input.placeholder = 'Add name…';
    nameEl.textContent = '';
    nameEl.appendChild(input);
    input.focus();
    input.select();

    function finish(save) {
      const val = input.value.trim();
      if (save && val) {
        window.api.setEntryName(id, val).then(() => {
          nameEl.textContent = val;
          nameEl.className = 'clip-name has-name';
        }).catch(() => {
          nameEl.textContent = current || 'Add name…';
          nameEl.className = 'clip-name' + (current ? ' has-name' : ' no-name');
        });
      } else {
        nameEl.textContent = current || 'Add name…';
        nameEl.className = 'clip-name' + (current ? ' has-name' : ' no-name');
      }
    }

    input.addEventListener('keydown', (ev) => {
      if (ev.key === 'Enter') { ev.preventDefault(); finish(true); }
      if (ev.key === 'Escape') { ev.preventDefault(); finish(false); }
    });
    input.addEventListener('blur', () => finish(true));
  }

  document.getElementById('btn-refresh')?.addEventListener('click', () => {
    loadEntries(window.search.getQuery(), window.search.getFilter());
  });

  document.getElementById('btn-edit-save')?.addEventListener('click', async () => {
    const overlay = document.getElementById('edit-overlay');
    const textarea = document.getElementById('edit-textarea');
    if (!overlay || !textarea) return;
    const id = parseInt(overlay.dataset.editId);
    const content = textarea.value.trim();
    if (!content) return;
    try {
      await window.api.updateEntry(id, content);
      window.ui.hideEdit();
      window.ui.showToast('Entry updated');
      loadEntries(window.search.getQuery(), window.search.getFilter());
    } catch (err) {
      console.error('Update failed:', err);
      window.ui.showError('Update failed: ' + (err?.message || err));
    }
  });

  document.getElementById('btn-edit-cancel')?.addEventListener('click', () => {
    window.ui.hideEdit();
  });

  document.getElementById('btn-edit-close')?.addEventListener('click', () => {
    window.ui.hideEdit();
  });

  document.getElementById('edit-overlay')?.addEventListener('click', (e) => {
    if (e.target === e.currentTarget) window.ui.hideEdit();
  });

  const cardList = document.getElementById('card-list');
  cardList.addEventListener('click', async (e) => {
    const nameEl = e.target.closest('.clip-name');
    if (nameEl && !e.target.closest('.clip-name-input')) {
      startNameEdit(nameEl);
      return;
    }

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
    } else if (btn.classList.contains('edit-btn')) {
      const entry = await window.api.getEntries(null, null).then(entries => entries.find(e => e.id === id));
      if (entry) window.ui.showEdit(id, entry.content);
      return;
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
  let selectedActionIndex = -1;
  let selectedEl = null;

  function selectCard(cards, idx) {
    cards.forEach((c) => c.classList.remove('selected'));
    selectedEl = null;
    if (idx >= 0 && idx < cards.length) {
      cards[idx].classList.add('selected');
      selectedEl = cards[idx];
      cards[idx].scrollIntoView({ block: 'nearest' });
    }
    selectedActionIndex = -1;
  }

  function validateSelection(cards) {
    if (selectedEl && !cards.includes(selectedEl)) {
      selectedEl.classList.remove('selected');
      selectedEl = null;
      selectedIndex = -1;
    }
  }

  function isInputFocused() {
    const tag = document.activeElement?.tagName;
    return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT';
  }

  function isOverlayOpen(id) {
    const el = document.getElementById(id);
    return el && (el.classList.contains('visible') || el.classList.contains('open'));
  }

  document.addEventListener('keydown', async (e) => {
    const cards = Array.from(cardList.querySelectorAll('.clip-card')).filter(
      (c) => !c.closest('.group.collapsed')
    );
    const searchInput = document.getElementById('search-input');

    // Escape: close overlays or hide to tray
    if (e.key === 'Escape') {
      if (isOverlayOpen('theme-overlay')) {
        document.getElementById('btn-theme-close')?.click();
        e.preventDefault();
        return;
      }
      if (isOverlayOpen('edit-overlay')) {
        window.ui.hideEdit();
        e.preventDefault();
        return;
      }
      if (isOverlayOpen('settings-overlay')) {
        document.getElementById('btn-settings-close')?.click();
        e.preventDefault();
        return;
      }
      if (isOverlayOpen('filter-panel')) {
        window.ui.hideFilterPanel();
        e.preventDefault();
        return;
      }
      if (appWindow) {
        appWindow.hide();
        e.preventDefault();
      }
      return;
    }

    // Ctrl+/ toggle search focus
    if ((e.ctrlKey || e.metaKey) && e.key === '/') {
      e.preventDefault();
      if (document.activeElement === searchInput) {
        searchInput?.blur();
      } else {
        searchInput?.focus();
      }
      return;
    }

    // Ctrl+ shortcuts (work regardless of input focus)
    if (e.ctrlKey || e.metaKey) {
      switch (e.key.toLowerCase()) {
        case 'l':
          e.preventDefault();
          handleLockAction();
          return;
        case 'r':
          e.preventDefault();
          btnStop?.click();
          return;
        case 'i':
          e.preventDefault();
          if (isOverlayOpen('settings-overlay')) {
            document.getElementById('btn-settings-close')?.click();
          } else {
            document.getElementById('btn-settings')?.click();
          }
          return;
        case 'd':
          e.preventDefault();
          document.getElementById('btn-clear-all')?.click();
          return;
        case 'f':
          e.preventDefault();
          document.getElementById('btn-filter')?.click();
          return;
      }
    }

    // ArrowDown from search focuses first card
    if (e.key === 'ArrowDown' && document.activeElement === searchInput && cards.length > 0) {
      e.preventDefault();
      selectedIndex = 0;
      selectCard(cards, 0);
      searchInput?.blur();
      return;
    }

    // Card navigation & actions (when not focused in an input)
    if (!isInputFocused()) {
      validateSelection(cards);

      if (e.key === 'ArrowDown' && cards.length > 0) {
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, cards.length - 1);
        selectCard(cards, selectedIndex);
        return;
      }

      if (e.key === 'ArrowUp' && cards.length > 0) {
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        selectCard(cards, selectedIndex);
        return;
      }

      if (selectedIndex >= 0 && selectedIndex < cards.length) {
        const card = cards[selectedIndex];
        const actionBtns = card.querySelectorAll('.clip-action-btn');

        if (e.key === 'ArrowRight') {
          e.preventDefault();
          if (actionBtns.length > 0) {
            selectedActionIndex = Math.min(selectedActionIndex + 1, actionBtns.length - 1);
            actionBtns[selectedActionIndex]?.focus();
          }
          return;
        }

        if (e.key === 'ArrowLeft') {
          e.preventDefault();
          if (actionBtns.length > 0) {
            selectedActionIndex = Math.max(selectedActionIndex - 1, 0);
            actionBtns[selectedActionIndex]?.focus();
          }
          return;
        }

        if (e.key === 'Enter') {
          e.preventDefault();
          copyById(parseInt(card.dataset.id));
          return;
        }

        if (e.key === 'Delete') {
          e.preventDefault();
          const id = parseInt(card.dataset.id);
          const confirmed = await window.ui.showConfirm('Delete this clipboard entry?');
          if (!confirmed) return;
          try {
            await window.api.deleteEntry(id);
            window.ui.removeCard(id);
            window.ui.showToast('Entry deleted');
          } catch (err) {
            console.error('Delete failed:', err);
          }
          return;
        }

        if (e.key === 'p' || e.key === 'P') {
          e.preventDefault();
          const id = parseInt(card.dataset.id);
          try {
            await window.api.togglePin(id);
            window.ui.updatePinState(id, !card.classList.contains('pinned'));
            loadEntries(window.search.getQuery(), window.search.getFilter());
          } catch (err) {
            console.error('Toggle pin failed:', err);
          }
          return;
        }

        if (e.key === 'e' || e.key === 'E') {
          e.preventDefault();
          const id = parseInt(card.dataset.id);
          const entries = await window.api.getEntries(null, null);
          const entry = entries.find(ent => ent.id === id);
          if (entry && entry.content_type === 'text') {
            window.ui.showEdit(id, entry.content);
          }
          return;
        }
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

  let _changePasswordFlow = false; // 'verify' | 'new' | false
  let _privateModeStatus = null;

  async function handleUnlockOrSetPassword() {
    const input = document.getElementById('lock-input');
    const password = input ? input.value.trim() : '';
    if (!password) return;

    const title = document.getElementById('lock-title');

    // Change password: verify old password first
    if (_changePasswordFlow === 'verify') {
      try {
        const ok = await window.api.unlockPrivateMode(password);
        if (ok) {
          _changePasswordFlow = 'new';
          if (input) { input.value = ''; }
          document.getElementById('lock-error').textContent = '';
          document.getElementById('lock-title').textContent = 'Set New Password';
          document.getElementById('lock-subtitle').textContent = 'Enter your new password';
          document.getElementById('lock-btn').textContent = 'Set Password';
          if (input) setTimeout(() => input.focus(), 50);
        } else {
          window.ui.setLockError('Wrong password');
          window.ui.lockShake();
          if (input) input.value = '';
        }
      } catch (err) {
        window.ui.setLockError(err?.message || 'Verification failed');
        window.ui.lockShake();
      }
      return;
    }

    // Change password: set new password
    if (_changePasswordFlow === 'new') {
      try {
        await window.api.setPrivateModePassword(password);
        _changePasswordFlow = false;
        window.ui.hideLockScreen();
        window.ui.showToast('Password changed');
      } catch (err) {
        window.ui.setLockError(err?.message || 'Failed to set password');
        window.ui.lockShake();
      }
      return;
    }

    // First-time setup
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
      // Unlock
      try {
        const ok = await window.api.unlockPrivateMode(password);
        if (ok) {
          window.ui.hideLockScreen();
  window.ui.showLoading();
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
  const recordLabel = document.getElementById('record-label');

  btnStop?.addEventListener('click', async () => {
    monitoring = !monitoring;
    try {
      await window.api.setMonitoring(monitoring);
      btnStop.title = monitoring ? 'Stop recording' : 'Start recording';
      if (recordDot) recordDot.classList.toggle('paused', !monitoring);
      if (recordLabel) recordLabel.textContent = monitoring ? 'REC' : 'PAUSED';
    } catch (err) {
      monitoring = !monitoring;
      console.error('Toggle monitoring failed:', err);
    }
  });

  document.getElementById('btn-filter')?.addEventListener('click', async () => {
    const panel = document.getElementById('filter-panel');
    if (panel?.classList.contains('open')) {
      window.ui.hideFilterPanel();
      return;
    }
    try {
      const names = await window.api.getAppNames();
      window.ui.showFilterPanel(names);
    } catch (err) {
      console.error('Failed to load app names:', err);
    }
  });

  document.getElementById('btn-filter-close')?.addEventListener('click', () => {
    window.ui.hideFilterPanel();
  });

  document.getElementById('filter-search-input')?.addEventListener('input', () => {
    window.ui.renderFilterList(_lastAppNames);
  });

  document.getElementById('filter-search-input')?.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') window.ui.hideFilterPanel();
  });

  let _lastAppNames = [];
  const _origShowFilter = window.ui.showFilterPanel;
  window.ui.showFilterPanel = function(names) {
    _lastAppNames = names;
    _origShowFilter(names);
  };

  document.getElementById('btn-filter-clear')?.addEventListener('click', () => {
    window.search.setAppFilter('');
    window.ui.hideFilterPanel();
  });

  document.getElementById('btn-filter-chip-close')?.addEventListener('click', () => {
    window.search.setAppFilter('');
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
    await window.api.setIgnoreBlur(true);
    try {
      const cfg = await window.api.getConfig();
      const hotkeyEl = document.getElementById('setting-hotkey');
      if (window.__onWayland) {
        hotkeyEl.textContent = window.__toggleCommand;
        hotkeyEl.title = 'Set this as your DE keybind command';
        hotkeyEl.style.cursor = 'default';
        hotkeyEl.classList.remove('settings-hotkey');
      } else {
        hotkeyEl.textContent = cfg.hotkey?.toggle_window || 'Alt+V';
        hotkeyEl.title = 'Click to change';
        hotkeyEl.style.cursor = '';
        hotkeyEl.classList.add('settings-hotkey');
      }

      try {
        const enabled = await window.api.isAutostartEnabled();
        document.getElementById('setting-autostart').checked = enabled;
      } catch (_) { /* best effort */ }

      const currentFont = cfg.theme?.font_family;
      document.getElementById('setting-font').value = (currentFont && !currentFont.startsWith('#')) ? currentFont : 'Inter, system-ui, sans-serif';

      document.getElementById('setting-max-entries').value = cfg.behavior?.max_entries ?? 0;

      _privateModeStatus = await window.api.getPrivateModeStatus();
      const pwBtn = document.getElementById('btn-settings-password');
      pwBtn.textContent = _privateModeStatus.has_password ? 'Change Password' : 'Set Password';

      window.ui.showSettings();
    } finally {
      await window.api.setIgnoreBlur(false);
    }
  });

  let listeningHotkey = false;
  const hotkeyEl = document.getElementById('setting-hotkey');
  let hotkeyHandler = null;

  hotkeyEl?.addEventListener('click', function () {
    if (listeningHotkey || window.__onWayland) return;
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

  document.getElementById('setting-font')?.addEventListener('blur', async (e) => {
    const val = e.target.value.trim() || 'Inter, system-ui, sans-serif';
    try {
      const cfg = await window.api.getConfig();
      cfg.theme.font_family = val;
      await window.api.saveConfig(cfg);
      document.documentElement.style.setProperty('--font-family', val);
      window.ui.showToast('Font updated');
    } catch (err) {
      console.error('Font save failed:', err);
    }
  });

  document.getElementById('setting-font')?.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      e.target.blur();
    }
  });

  document.getElementById('setting-max-entries')?.addEventListener('blur', async (e) => {
    let val = parseInt(e.target.value, 10);
    if (Number.isNaN(val) || val < 0) val = 0;
    e.target.value = val;
    try {
      const cfg = await window.api.getConfig();
      cfg.behavior = cfg.behavior || {};
      cfg.behavior.max_entries = val;
      await window.api.saveConfig(cfg);
      window.ui.showToast(val === 0 ? 'Unlimited history' : `History limited to ${val}`);
    } catch (err) {
      console.error('Max entries save failed:', err);
    }
  });

  document.getElementById('setting-max-entries')?.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      e.target.blur();
    }
  });

  document.getElementById('btn-settings-password')?.addEventListener('click', () => {
    window.ui.hideSettings();
    if (_privateModeStatus && _privateModeStatus.has_password) {
      _changePasswordFlow = 'verify';
      const overlay = document.getElementById('lock-overlay');
      overlay.classList.add('visible');
      const input = document.getElementById('lock-input');
      if (input) { input.value = ''; input.classList.remove('shake'); input.type = 'password'; }
      document.getElementById('lock-error').textContent = '';
      document.getElementById('lock-title').textContent = 'Enter Current Password';
      document.getElementById('lock-subtitle').textContent = 'Verify your identity to change password';
      document.getElementById('lock-btn').textContent = 'Verify';
      if (input) setTimeout(() => input.focus(), 100);
    } else {
      window.ui.showPasswordSetup();
    }
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
      name: 'Graphite',
      theme: {
        bg_primary: '#0B0D12', bg_secondary: '#12151D', bg_card: '#171B24',
        text_primary: '#F1F4F9', text_secondary: '#9AA4B2',
        accent: '#4E8AFF', accent_hover: '#3D72E8',
        danger: '#E5484D', success: '#2FB58A', border: '#1F2430',
        bg_modal: '#141822', border_card: '#262D3B', accent_subtle: '#182A4D',
      },
    },
    {
      name: 'Void Purple',
      theme: {
        bg_primary: '#0E0A18', bg_secondary: '#161127', bg_card: '#1C1631',
        text_primary: '#EFEAFB', text_secondary: '#A79BC9',
        accent: '#8B5CF6', accent_hover: '#7C3AED',
        danger: '#F87171', success: '#34D399', border: '#241C3E',
        bg_modal: '#181329', border_card: '#2E2450', accent_subtle: '#241A4A',
      },
    },
    {
      name: 'Synthwave',
      theme: {
        bg_primary: '#100816', bg_secondary: '#1A0F26', bg_card: '#221436',
        text_primary: '#FFD9EF', text_secondary: '#C88FAE',
        accent: '#F935B3', accent_hover: '#D6209A',
        danger: '#FF6B6B', success: '#3DFFC0', border: '#3A1848',
        bg_modal: '#1C1030', border_card: '#451E58', accent_subtle: '#330B26',
      },
    },
    {
      name: 'Midnight Ocean',
      theme: {
        bg_primary: '#041018', bg_secondary: '#0A1C2A', bg_card: '#0F2434',
        text_primary: '#E3F6FC', text_secondary: '#7FB0C4',
        accent: '#22D3EE', accent_hover: '#0EA5C4',
        danger: '#FF5F5F', success: '#34D399', border: '#0E2E44',
        bg_modal: '#0C1E2C', border_card: '#123A54', accent_subtle: '#062F45',
      },
    },
    {
      name: 'Cyberpunk Terminal',
      theme: {
        bg_primary: '#0A0C0A', bg_secondary: '#111611', bg_card: '#161D16',
        text_primary: '#EDFBED', text_secondary: '#9BB69B',
        accent: '#34D399', accent_hover: '#2CBE88',
        danger: '#FF5C5C', success: '#2ED17C', border: '#1C241C',
        bg_modal: '#141A14', border_card: '#243024', accent_subtle: '#0D2E22',
      },
    },
    {
      name: 'Arctic Frost',
      theme: {
        bg_primary: '#F4F6FA', bg_secondary: '#FFFFFF', bg_card: '#FFFFFF',
        text_primary: '#1A2233', text_secondary: '#5A6B82',
        accent: '#2563EB', accent_hover: '#1D4ED8',
        danger: '#DC2626', success: '#15803D', border: '#D7DEE9',
        bg_modal: '#FAFBFD', border_card: '#C9D4E3', accent_subtle: '#E2EBFB',
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
      const graphite = presets.find(p => p.name === 'Graphite').theme;
      applyThemeToEditor(graphite);
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
    const choice = await window.ui.showClearAllDialog();
    if (!choice) return;
    const keepPinned = choice === 'keep';
    try {
      await window.api.clearAll(keepPinned);
      loadEntries(window.search.getQuery(), window.search.getFilter());
      window.ui.showToast(keepPinned ? 'History cleared (pinned kept)' : 'History cleared');
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

    window.__TAURI__.event.listen('hotkey-show', () => {
      window.ui.hideSettings();
      window.ui.hideEdit();
      document.getElementById('theme-overlay')?.classList.remove('visible');
      window.ui.hideFilterPanel();

      const cards = Array.from(cardList.querySelectorAll('.clip-card')).filter(
        (c) => !c.closest('.group.collapsed')
      );
      if (cards.length > 0) {
        selectedIndex = 0;
        selectCard(cards, 0);
        cards[0].focus();
      }
    });

    window.__TAURI__.event.listen('wayland-hotkey-info', (event) => {
      const cmd = event.payload;
      window.ui.showToast(
        'Wayland: set DE keybind to run "' + cmd + ' toggle" for Alt+V',
        5000
      );
      document.getElementById('setting-hotkey').textContent = cmd + ' toggle';
      document.getElementById('setting-hotkey').title = 'Set this as your DE keybind command';
      document.getElementById('setting-hotkey').style.cursor = 'default';
      window.__onWayland = true;
      window.__toggleCommand = cmd + ' toggle';
    });

    window.__TAURI__.event.listen('paste-error', (event) => {
      const msg = String(event.payload || '').replace(/\s+/g, ' ').trim();
      window.ui.showToast('Copied, but paste failed: ' + msg, 8000);
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

  try {
    const mode = await window.api.getFocusMode();
    if (mode === 'no-focus') {
      const hint = document.getElementById('focus-mode-hint');
      const text = hint.querySelector('.focus-mode-hint-text');
      text.textContent =
        'No GNOME Shell extension: the picker is mouse-only here. ' +
        'Install "Window Calls" (extensions.gnome.org/extension/4724) and ' +
        're-login to restore keyboard navigation.';
      hint.style.display = 'flex';
    }
  } catch (err) {
    console.error('Failed to check focus mode:', err);
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
