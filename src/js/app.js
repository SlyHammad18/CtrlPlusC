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
        await window.api.copyToClipboard(entry.content);
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

  document.getElementById('btn-lock')?.addEventListener('click', handleLockAction);

  document.getElementById('lock-btn')?.addEventListener('click', handleUnlockOrSetPassword);

  document.getElementById('lock-input')?.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleUnlockOrSetPassword();
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

  setInterval(() => {
    loadEntries(window.search.getQuery(), window.search.getFilter());
  }, 5000);
})();
