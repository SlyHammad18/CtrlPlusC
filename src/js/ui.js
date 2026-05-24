window.ui = (() => {
  const cardList = document.getElementById('card-list');
  const emptyState = document.getElementById('empty-state');

  function getGroupLabel(date) {
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);
    const weekStart = new Date(today);
    weekStart.setDate(weekStart.getDate() - weekStart.getDay());
    const lastWeekStart = new Date(weekStart);
    lastWeekStart.setDate(lastWeekStart.getDate() - 7);

    if (date >= today) return 'Today';
    if (date >= yesterday) return 'Yesterday';
    if (date >= weekStart) return 'This Week';
    if (date >= lastWeekStart) return 'Last Week';
    return 'Older';
  }

  function formatTimestamp(ts) {
    const date = new Date(ts.replace(' ', 'T') + 'Z');
    const now = new Date();
    const diffMs = now - date;
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins} min ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  }

  function createCard(entry, query) {
    const card = document.createElement('div');
    card.className = 'clip-card' + (entry.is_pinned ? ' pinned' : '') + ' entering';
    card.dataset.id = entry.id;

    const preview = document.createElement('div');
    preview.className = 'clip-preview';
    preview.innerHTML = window.search.highlight(
      entry.preview || entry.content,
      query
    );

    const footer = document.createElement('div');
    footer.className = 'clip-footer';

    const timestamp = document.createElement('span');
    timestamp.className = 'clip-timestamp';
    timestamp.textContent = formatTimestamp(entry.timestamp);

    const actions = document.createElement('div');
    actions.className = 'clip-actions';

    const copyBtn = document.createElement('button');
    copyBtn.className = 'clip-action-btn copy-btn';
    copyBtn.title = 'Copy';
    copyBtn.innerHTML =
      '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>';

    const pinBtn = document.createElement('button');
    pinBtn.className = 'clip-action-btn pin-btn';
    pinBtn.title = entry.is_pinned ? 'Unpin' : 'Pin';
    pinBtn.innerHTML =
      '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2z"/></svg>';

    const deleteBtn = document.createElement('button');
    deleteBtn.className = 'clip-action-btn delete-btn';
    deleteBtn.title = 'Delete';
    deleteBtn.innerHTML =
      '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>';

    actions.appendChild(copyBtn);
    actions.appendChild(pinBtn);
    actions.appendChild(deleteBtn);
    footer.appendChild(timestamp);
    footer.appendChild(actions);
    card.appendChild(preview);
    card.appendChild(footer);

    return card;
  }

  function renderCards(entries, query) {
    cardList.innerHTML = '';

    if (!entries || entries.length === 0) {
      emptyState.style.display = 'flex';
      const hasQuery = query && query.length > 0;
      if (hasQuery) {
        emptyState.innerHTML = `
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="empty-icon">
            <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
          <p class="empty-text">No results for "${query}"</p>
          <p class="empty-hint">Try a different search term</p>
        `;
      } else {
        emptyState.innerHTML = `
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="empty-icon">
            <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>
            <rect x="8" y="2" width="8" height="4" rx="1" ry="1"/>
          </svg>
          <p class="empty-text">No clipboard entries yet</p>
          <p class="empty-hint">Copy something to get started</p>
          <button id="btn-refresh" class="filter-btn" style="margin-top:10px">Refresh</button>
        `;
        document.getElementById('btn-refresh')?.addEventListener('click', () => loadEntriesRef(''));
      }
      return;
    }

    emptyState.style.display = 'none';

    const pinned = entries.filter(e => e.is_pinned);
    const unpinned = entries.filter(e => !e.is_pinned);

    const groups = [];
    if (pinned.length) groups.push({ label: 'Pinned', items: pinned });

    const grouped = {};
    unpinned.forEach(entry => {
      const date = new Date(entry.timestamp.replace(' ', 'T') + 'Z');
      const label = getGroupLabel(date);
      if (!grouped[label]) grouped[label] = [];
      grouped[label].push(entry);
    });

    const groupOrder = ['Today', 'Yesterday', 'This Week', 'Last Week', 'Older'];
    groupOrder.forEach(label => {
      if (grouped[label]?.length) groups.push({ label, items: grouped[label] });
    });

    const fragment = document.createDocumentFragment();
    groups.forEach((group) => {
      const divider = document.createElement('div');
      divider.className = 'group-divider';
      const label = document.createElement('span');
      label.className = 'group-label';
      label.textContent = group.label;
      divider.appendChild(label);
      fragment.appendChild(divider);
      group.items.forEach((entry) => {
        fragment.appendChild(createCard(entry, query));
      });
    });
    cardList.appendChild(fragment);
  }

  function prependCard(entry) {
    emptyState.style.display = 'none';
    const existing = cardList.querySelector(`[data-id="${entry.id}"]`);
    if (existing) return;

    const card = createCard(entry, window.search.getQuery());
    card.classList.remove('entering');
    card.classList.add('entering');
    cardList.insertBefore(card, cardList.firstChild);

    requestAnimationFrame(() => {
      card.classList.remove('entering');
    });
  }

  function removeCard(id) {
    const card = cardList.querySelector(`[data-id="${id}"]`);
    if (!card) return;

    card.classList.add('removing');
    setTimeout(() => {
      card.remove();
      if (cardList.children.length === 0) {
        emptyState.style.display = 'flex';
      }
    }, 250);
  }

  function updatePinState(id, isPinned) {
    const card = cardList.querySelector(`[data-id="${id}"]`);
    if (!card) return;

    card.classList.toggle('pinned', isPinned);
    const pinBtn = card.querySelector('.pin-btn');
    if (pinBtn) {
      pinBtn.title = isPinned ? 'Unpin' : 'Pin';
    }
  }

  function showToast(message, duration = 2000) {
    const existing = document.querySelector('.toast');
    if (existing) existing.remove();

    const toast = document.createElement('div');
    toast.className = 'toast';
    toast.textContent = message;
    document.body.appendChild(toast);

    requestAnimationFrame(() => {
      toast.classList.add('toast-visible');
    });

    setTimeout(() => {
      toast.classList.remove('toast-visible');
      setTimeout(() => toast.remove(), 300);
    }, duration);
  }

  function showConfirm(message) {
    return new Promise((resolve) => {
      const overlay = document.createElement('div');
      overlay.className = 'confirm-overlay';

      const box = document.createElement('div');
      box.className = 'confirm-box';

      const text = document.createElement('p');
      text.className = 'confirm-text';
      text.textContent = message;

      const actions = document.createElement('div');
      actions.className = 'confirm-actions';

      const cancelBtn = document.createElement('button');
      cancelBtn.className = 'confirm-btn confirm-cancel';
      cancelBtn.textContent = 'Cancel';

      const okBtn = document.createElement('button');
      okBtn.className = 'confirm-btn confirm-ok';
      okBtn.textContent = 'Delete';

      actions.appendChild(cancelBtn);
      actions.appendChild(okBtn);
      box.appendChild(text);
      box.appendChild(actions);
      overlay.appendChild(box);
      document.body.appendChild(overlay);

      requestAnimationFrame(() => {
        overlay.classList.add('confirm-visible');
      });

      function close(result) {
        overlay.classList.remove('confirm-visible');
        setTimeout(() => overlay.remove(), 200);
        resolve(result);
      }

      cancelBtn.addEventListener('click', () => close(false));
      okBtn.addEventListener('click', () => close(true));
      overlay.addEventListener('click', (e) => {
        if (e.target === overlay) close(false);
      });
    });
  }

  function showError(message) {
    const el = document.getElementById('empty-state');
    if (!el) return;
    el.style.display = 'flex';
    el.innerHTML = `
      <p class="empty-text" style="color:var(--danger)">${message}</p>
      <button id="btn-retry" class="filter-btn" style="margin-top:8px">Retry</button>
    `;
    const retry = document.getElementById('btn-retry');
    if (retry) {
      retry.addEventListener('click', () => {
        el.innerHTML = `
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="empty-icon">
            <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>
            <rect x="8" y="2" width="8" height="4" rx="1" ry="1"/>
          </svg>
          <p class="empty-text">No clipboard entries yet</p>
          <p class="empty-hint">Copy something to get started</p>
        `;
        loadEntriesRef('');
      });
    }
  }

  let loadEntriesRef = () => {};

  function setLoadEntries(fn) {
    loadEntriesRef = fn;
  }

  function showLockScreen() {
    const overlay = document.getElementById('lock-overlay');
    if (!overlay) return;
    overlay.classList.add('visible');
    const input = document.getElementById('lock-input');
    const error = document.getElementById('lock-error');
    const title = document.getElementById('lock-title');
    const subtitle = document.getElementById('lock-subtitle');
    const btn = document.getElementById('lock-btn');
    if (input) { input.value = ''; input.classList.remove('shake'); }
    if (error) error.textContent = '';
    if (title) title.textContent = 'Locked';
    if (subtitle) subtitle.textContent = 'Enter your password to unlock';
    if (btn) btn.textContent = 'Unlock';
    if (input) setTimeout(() => input.focus(), 100);
  }

  function hideLockScreen() {
    const overlay = document.getElementById('lock-overlay');
    if (overlay) overlay.classList.remove('visible');
  }

  function showPasswordSetup() {
    const overlay = document.getElementById('lock-overlay');
    if (!overlay) return;
    overlay.classList.add('visible');
    const input = document.getElementById('lock-input');
    const error = document.getElementById('lock-error');
    const title = document.getElementById('lock-title');
    const subtitle = document.getElementById('lock-subtitle');
    const btn = document.getElementById('lock-btn');
    if (input) { input.value = ''; input.classList.remove('shake'); input.type = 'password'; }
    if (error) error.textContent = '';
    if (title) title.textContent = 'Set Password';
    if (subtitle) subtitle.textContent = 'Create a password to lock your clipboard';
    if (btn) btn.textContent = 'Set Password';
    if (input) setTimeout(() => input.focus(), 100);
  }

  function lockShake() {
    const input = document.getElementById('lock-input');
    if (input) {
      input.classList.remove('shake');
      void input.offsetWidth;
      input.classList.add('shake');
    }
  }

  function setLockError(msg) {
    const error = document.getElementById('lock-error');
    if (error) error.textContent = msg;
  }

  function showSettings() {
    const overlay = document.getElementById('settings-overlay');
    if (overlay) overlay.classList.add('visible');
  }

  function hideSettings() {
    const overlay = document.getElementById('settings-overlay');
    if (overlay) overlay.classList.remove('visible');
  }

  return { renderCards, prependCard, removeCard, updatePinState, showToast, showConfirm, showError, setLoadEntries, showLockScreen, hideLockScreen, showPasswordSetup, lockShake, setLockError, showSettings, hideSettings };
})();
