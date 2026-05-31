const APP_NAME_MAP = {
  chrome: 'Google Chrome',
  firefox: 'Firefox',
  msedge: 'Microsoft Edge',
  edge: 'Microsoft Edge',
  opera: 'Opera',
  iexplore: 'Internet Explorer',
  code: 'Visual Studio Code',
  codeoss: 'VS Code (OSS)',
  sublime_text: 'Sublime Text',
  notepad: 'Notepad',
  notepadpp: 'Notepad++',
  winword: 'Microsoft Word',
  wordpad: 'WordPad',
  excel: 'Microsoft Excel',
  powerpnt: 'Microsoft PowerPoint',
  outlook: 'Microsoft Outlook',
  onenote: 'Microsoft OneNote',
  mspub: 'Microsoft Publisher',
  access: 'Microsoft Access',
  thunderbird: 'Thunderbird',
  slack: 'Slack',
  discord: 'Discord',
  telegram: 'Telegram',
  whatsapp: 'WhatsApp',
  signal: 'Signal',
  zoom: 'Zoom',
  teams: 'Microsoft Teams',
  terminal: 'Terminal',
  windowsterminal: 'Windows Terminal',
  cmd: 'Command Prompt',
  powershell: 'PowerShell',
  pwsh: 'PowerShell',
  explorer: 'File Explorer',
  sublime: 'Sublime Text',
  atom: 'Atom',
  brackets: 'Brackets',
  postman: 'Postman',
  insomnia: 'Insomnia',
  figma: 'Figma',
  photoshop: 'Adobe Photoshop',
  illustrator: 'Adobe Illustrator',
  acrobat: 'Adobe Acrobat',
  acrord32: 'Adobe Acrobat',
  spotify: 'Spotify',
  vlc: 'VLC Media Player',
  mpc: 'Media Player Classic',
  mpc64: 'Media Player Classic',
  putty: 'PuTTY',
  winscp: 'WinSCP',
  filezilla: 'FileZilla',
  gitbash: 'Git Bash',
  bash: 'Bash',
  docker: 'Docker',
  docker_desktop: 'Docker Desktop',
  dbeaver: 'DBeaver',
  mysqlworkbench: 'MySQL Workbench',
  sqlserver: 'SQL Server Management Studio',
  ssms: 'SQL Server Management Studio',
  obsidian: 'Obsidian',
  notion: 'Notion',
  evernote: 'Evernote',
  brave: 'Brave',
  vivaldi: 'Vivaldi',
  tor: 'Tor Browser',
  chromimum: 'Chromium',
  wechat: 'WeChat',
  dingtalk: 'DingTalk',
  feishu: 'Feishu',
  alacritty: 'Alacritty',
  kitty: 'Kitty',
  iterm2: 'iTerm2',
  mobaxterm: 'MobaXterm',
  bitwarden: 'Bitwarden',
  keepass: 'KeePass',
  '1password': '1Password',
};

function formatAppName(name) {
  if (!name) return '';
  const lower = name.toLowerCase().replace(/\.exe$/, '');
  if (APP_NAME_MAP[lower]) return APP_NAME_MAP[lower];
  return lower.charAt(0).toUpperCase() + lower.slice(1);
}

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
    card.className = 'clip-card' + (entry.is_pinned ? ' pinned' : '') + ' entering' + (entry.content_type === 'image' ? ' clip-card-image' : '');
    card.dataset.id = entry.id;

    if (entry.content_type === 'image') {
      const nameEl = document.createElement('div');
      nameEl.className = 'clip-name';
      nameEl.dataset.id = entry.id;
      if (entry.name) {
        nameEl.textContent = entry.name;
        nameEl.classList.add('has-name');
      } else {
        nameEl.textContent = 'Add name…';
        nameEl.classList.add('no-name');
      }

      const imgWrap = document.createElement('div');
      imgWrap.className = 'clip-image-wrap';

      const img = document.createElement('img');
      img.className = 'clip-image-thumb';
      img.alt = 'Clipboard image';
      imgWrap.appendChild(img);

      const imgLabel = document.createElement('div');
      imgLabel.className = 'clip-image-label';
      imgLabel.textContent = entry.preview || 'Image';

      card.appendChild(nameEl);
      card.appendChild(imgWrap);
      card.appendChild(imgLabel);

      window.api.getEntryImage(entry.id).then((dataUri) => {
        if (dataUri) img.src = dataUri;
      }).catch(() => {});

      const preview = document.createElement('div');
      preview.className = 'clip-preview';
      preview.style.display = 'none';
      card.appendChild(preview);
    } else {
      const preview = document.createElement('div');
      preview.className = 'clip-preview';
      preview.innerHTML = window.search.highlight(
        entry.preview || entry.content,
        query
      );
      card.appendChild(preview);
    }

    if (entry.source_app) {
      const appLabel = document.createElement('div');
      appLabel.className = 'clip-source-app';
      appLabel.textContent = formatAppName(entry.source_app);
      card.appendChild(appLabel);
    }

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

    const editBtn = document.createElement('button');
    editBtn.className = 'clip-action-btn edit-btn';
    editBtn.title = 'Edit';
    editBtn.innerHTML =
      '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>';

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

    if (entry.content_type !== 'image') {
      actions.appendChild(editBtn);
    }
    actions.appendChild(copyBtn);
    actions.appendChild(pinBtn);
    actions.appendChild(deleteBtn);
    footer.appendChild(timestamp);
    footer.appendChild(actions);
    card.appendChild(footer);

    return card;
  }

  function renderCards(entries, query) {
    cardList.innerHTML = '';
    cardList.appendChild(emptyState);

    if (!entries || entries.length === 0) {
      emptyState.style.display = 'flex';
      const hasQuery = query && query.length > 0;
      if (hasQuery) {
        emptyState.innerHTML = `
          <div class="empty-art">
            <svg width="52" height="52" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" class="empty-icon-svg">
              <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
            </svg>
          </div>
          <p class="empty-text">No results for "${query}"</p>
          <p class="empty-hint">Try a different search term</p>
        `;
      } else {
        emptyState.innerHTML = `
          <div class="empty-art">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" class="empty-icon-svg">
              <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>
              <rect x="8" y="2" width="8" height="4" rx="1" ry="1"/>
              <line x1="12" y1="11" x2="12" y2="17"/>
              <polyline points="9 14 12 11 15 14"/>
            </svg>
          </div>
          <p class="empty-text">No clipboard history</p>
          <p class="empty-hint">Select some text and copy it —<br/>it will appear here</p>
          <div class="empty-shortcut">
            <span class="shortcut-key">${window.api.getHotkeyDisplay ? 'Alt+V' : 'Alt+V'}</span>
            <span class="shortcut-label">to toggle window</span>
          </div>
        `;
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

    const d = new Date(entry.timestamp.replace(' ', 'T') + 'Z');
    const now = new Date();
    const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterdayStart = new Date(todayStart);
    yesterdayStart.setDate(yesterdayStart.getDate() - 1);
    const weekStart = new Date(todayStart);
    weekStart.setDate(weekStart.getDate() - weekStart.getDay());
    const lastWeekStart = new Date(weekStart);
    lastWeekStart.setDate(lastWeekStart.getDate() - 7);

    let targetLabel = 'Today';
    if (d >= todayStart) targetLabel = 'Today';
    else if (d >= yesterdayStart) targetLabel = 'Yesterday';
    else if (d >= weekStart) targetLabel = 'This Week';
    else if (d >= lastWeekStart) targetLabel = 'Last Week';
    else targetLabel = 'Older';

    const order = ['Pinned', 'Today', 'Yesterday', 'This Week', 'Last Week', 'Older'];
    const dividers = cardList.querySelectorAll('.group-divider');
    let inserted = false;

    for (const div of dividers) {
      const span = div.querySelector('.group-label');
      if (span && span.textContent === targetLabel) {
        cardList.insertBefore(card, div.nextSibling);
        inserted = true;
        break;
      }
    }

    if (!inserted) {
      let refNode = null;
      for (const div of dividers) {
        const span = div.querySelector('.group-label');
        if (span) {
          const targetIdx = order.indexOf(targetLabel);
          const thisIdx = order.indexOf(span.textContent);
          if (thisIdx > targetIdx) {
            refNode = div;
            break;
          }
        }
      }
      const newDivider = document.createElement('div');
      newDivider.className = 'group-divider';
      const newLabel = document.createElement('span');
      newLabel.className = 'group-label';
      newLabel.textContent = targetLabel;
      newDivider.appendChild(newLabel);
      if (refNode) {
        cardList.insertBefore(newDivider, refNode);
        cardList.insertBefore(card, refNode);
      } else {
        cardList.appendChild(newDivider);
        cardList.appendChild(card);
      }
    }

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
      if (cardList.children.length <= 1) {
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
    if (!el.parentNode) cardList.appendChild(el);
    el.innerHTML = `
      <p class="empty-text" style="color:var(--danger)">${message}</p>
      <button id="btn-retry" class="filter-btn" style="margin-top:8px">Retry</button>
    `;
    const retry = document.getElementById('btn-retry');
    if (retry) {
      retry.addEventListener('click', () => {
        loadEntriesRef('');
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

  function showEdit(id, content) {
    const overlay = document.getElementById('edit-overlay');
    const textarea = document.getElementById('edit-textarea');
    if (!overlay || !textarea) return;
    overlay.dataset.editId = id;
    textarea.value = content;
    overlay.classList.add('visible');
    setTimeout(() => textarea.focus(), 100);
  }

  function hideEdit() {
    const overlay = document.getElementById('edit-overlay');
    if (overlay) overlay.classList.remove('visible');
  }

  function updateFilterBadge() {
    const badge = document.getElementById('filter-badge');
    const chip = document.getElementById('filter-chip');
    const chipLabel = document.getElementById('filter-chip-label');
    const currentApp = window.search.getAppFilter();
    if (badge) badge.style.display = currentApp ? '' : 'none';
    if (chip && chipLabel) {
      if (currentApp) {
        chip.style.display = 'flex';
        chipLabel.textContent = formatAppName(currentApp);
      } else {
        chip.style.display = 'none';
      }
    }
  }

  function renderFilterList(appNames) {
    const list = document.getElementById('filter-app-list');
    const input = document.getElementById('filter-search-input');
    if (!list) return;

    const currentApp = window.search.getAppFilter();
    const query = input ? input.value.trim().toLowerCase() : '';

    let filtered = appNames;
    if (query) {
      filtered = appNames.filter(n => formatAppName(n).toLowerCase().includes(query));
    }

    list.innerHTML = '';
    if (!filtered || filtered.length === 0) {
      const empty = document.createElement('div');
      empty.className = 'filter-app-empty';
      empty.textContent = query ? 'No matching apps' : 'No app data yet';
      list.appendChild(empty);
      return;
    }

    filtered.forEach((name) => {
      const btn = document.createElement('button');
      btn.className = 'filter-app-btn' + (name === currentApp ? ' active' : '');
      btn.textContent = formatAppName(name);
      btn.dataset.app = name;
      btn.addEventListener('click', () => {
        if (name === currentApp) {
          window.search.setAppFilter('');
        } else {
          window.search.setAppFilter(name);
        }
        closeFilterPanel();
      });
      list.appendChild(btn);
    });
  }

  function closeFilterPanel() {
    const panel = document.getElementById('filter-panel');
    if (panel) panel.classList.remove('open');
    updateFilterBadge();
  }

  function showFilterPanel(appNames) {
    const panel = document.getElementById('filter-panel');
    const input = document.getElementById('filter-search-input');
    if (!panel) return;

    _cachedAppNames = appNames;
    panel.classList.add('open');
    if (input) input.value = '';
    renderFilterList(appNames);
    if (input) setTimeout(() => input.focus(), 100);
  }

  let _cachedAppNames = [];

  function hideFilterPanel() {
    closeFilterPanel();
  }

  return { renderCards, prependCard, removeCard, updatePinState, showToast, showConfirm, showError, setLoadEntries, showLockScreen, hideLockScreen, showPasswordSetup, lockShake, setLockError, showSettings, hideSettings, showEdit, hideEdit, showFilterPanel, hideFilterPanel, updateFilterBadge, renderFilterList };
})();
