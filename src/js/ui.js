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
  // Backend now stores friendly names (e.g. "GNOME Terminal"); return as-is
  // so we don't mangle them with naive title-casing.
  return name.trim();
}

window.ui = (() => {
  const cardList = document.getElementById('card-list');
  const emptyState = document.getElementById('empty-state');

  function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = String(text ?? '');
    return div.innerHTML;
  }

  const CHEVRON_SVG =
    '<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 6 15 12 9 18"/></svg>';

  const imageObserver = new IntersectionObserver((entries, obs) => {
    entries.forEach((entry) => {
      if (!entry.isIntersecting) return;
      const img = entry.target;
      obs.unobserve(img);
      const id = img.dataset.imageId;
      if (id) {
        window.api.getEntryImage(parseInt(id)).then((dataUri) => {
          if (dataUri) img.src = dataUri;
        }).catch(() => {});
      }
    });
  }, { root: cardList, rootMargin: '300px 0px' });

  const TRASH_SVG =
    '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>';

  const FOLDER_PLUS_SVG =
    '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4l2 3h9a2 2 0 0 1 2 2z"/><line x1="12" y1="11" x2="12" y2="17"/><line x1="9" y1="14" x2="15" y2="14"/></svg>';

  const FOLDER_MINUS_SVG =
    '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4l2 3h9a2 2 0 0 1 2 2z"/><line x1="9" y1="14" x2="15" y2="14"/></svg>';

  let collapsedSections = new Set();
  let collapsePersistRef = null;

  function setCollapsedSections(list) {
    collapsedSections = new Set(list || []);
  }

  function setCollapsePersist(fn) {
    collapsePersistRef = fn;
  }

  function toggleSection(section) {
    const query = window.search && window.search.getQuery ? window.search.getQuery() : '';
    if (query && query.length) return;
    const label = section.dataset.group;
    const collapsed = section.classList.toggle('collapsed');
    const btn = section.querySelector('.group-divider');
    if (btn) {
      btn.setAttribute('aria-expanded', String(!collapsed));
      btn.title = collapsed ? 'Expand ' + label : 'Collapse ' + label;
    }
    const body = section.querySelector('.group-items');
    if (body) body.inert = collapsed;
    if (collapsed) {
      section.querySelectorAll('.clip-card.selected').forEach((c) => c.classList.remove('selected'));
    }
    if (collapsed) collapsedSections.add(label);
    else collapsedSections.delete(label);
    if (collapsePersistRef) collapsePersistRef([...collapsedSections]);
  }

  function buildGroupSection(group, query, forceExpanded) {
    const section = document.createElement('section');
    section.className = 'group';
    section.dataset.group = group.label;

    const collapsed = !forceExpanded && collapsedSections.has(group.label);
    if (collapsed) section.classList.add('collapsed');

    const itemsId = 'group-items-' + group.label.toLowerCase().replace(/\s+/g, '-');

    const divider = document.createElement('button');
    divider.type = 'button';
    divider.tabIndex = -1;
    divider.className = 'group-divider';
    divider.setAttribute('aria-expanded', String(!collapsed));
    divider.setAttribute('aria-controls', itemsId);
    divider.title = collapsed ? 'Expand ' + group.label : 'Collapse ' + group.label;

    const chevron = document.createElement('span');
    chevron.className = 'group-chevron';
    chevron.setAttribute('aria-hidden', 'true');
    chevron.innerHTML = CHEVRON_SVG;
    divider.appendChild(chevron);

    const label = document.createElement('span');
    label.className = 'group-label';
    label.textContent = group.label;
    divider.appendChild(label);

    const count = document.createElement('span');
    count.className = 'group-count';
    count.textContent = group.items.length;
    divider.appendChild(count);

    const rule = document.createElement('span');
    rule.className = 'group-rule';
    divider.appendChild(rule);

    if (group.label !== 'Pinned') {
      const delBtn = document.createElement('button');
      delBtn.type = 'button';
      delBtn.className = 'group-delete-btn';
      delBtn.title = 'Delete all ' + group.label.toLowerCase() + ' entries';
      delBtn.setAttribute('aria-label', 'Delete all ' + group.items.length + ' entries in ' + group.label);
      delBtn.innerHTML = TRASH_SVG;
      let deleting = false;
      delBtn.addEventListener('click', async (e) => {
        e.stopPropagation();
        if (deleting || group.items.length === 0) return;
        const confirmed = await window.ui.showConfirm(
          'Delete all ' + group.items.length + ' entries from ' + group.label + '?'
        );
        if (!confirmed) return;
        deleting = true;
        try {
          const n = await window.api.deleteGroup(group.label);
          window.ui.showToast(n === 1 ? '1 entry deleted' : n + ' entries deleted', 2500);
          loadEntriesRef(window.search && window.search.getQuery ? window.search.getQuery() : '');
        } catch (err) {
          console.error('Failed to delete group:', err);
          deleting = false;
          window.ui.showToast('Delete failed');
        }
      });
      divider.appendChild(delBtn);
    }

    const body = document.createElement('div');
    body.className = 'group-items';
    body.id = itemsId;
    body.inert = collapsed;
    const inner = document.createElement('div');
    inner.className = 'group-items-inner';
    group.items.forEach((entry) => inner.appendChild(createCard(entry, query)));
    body.appendChild(inner);

    divider.addEventListener('click', () => toggleSection(section));

    section.appendChild(divider);
    section.appendChild(body);
    return section;
  }

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
      img.loading = 'lazy';
      img.decoding = 'async';
      img.dataset.imageId = entry.id;
      imgWrap.appendChild(img);

      const imgLabel = document.createElement('div');
      imgLabel.className = 'clip-image-label';
      imgLabel.textContent = entry.preview || 'Image';

      card.appendChild(nameEl);
      card.appendChild(imgWrap);
      card.appendChild(imgLabel);

      imageObserver.observe(img);

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

    // Search/filter results span all folders; show where the entry lives.
    if (entry.folder_name) {
      const folderBadge = document.createElement('div');
      folderBadge.className = 'clip-folder-badge';
      folderBadge.title = 'In folder: ' + entry.folder_name;
      folderBadge.innerHTML =
        '<svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4l2 3h9a2 2 0 0 1 2 2z"/></svg>';
      const folderText = document.createElement('span');
      folderText.textContent = entry.folder_name;
      folderBadge.appendChild(folderText);
      card.appendChild(folderBadge);
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
    deleteBtn.innerHTML = TRASH_SVG;

    if (entry.content_type !== 'image') {
      actions.appendChild(editBtn);
    }
    actions.appendChild(copyBtn);
    actions.appendChild(pinBtn);

    if (entry.folder_id == null) {
      const folderBtn = document.createElement('button');
      folderBtn.className = 'clip-action-btn folder-add-btn';
      folderBtn.title = 'Add to folder';
      folderBtn.innerHTML = FOLDER_PLUS_SVG;
      actions.appendChild(folderBtn);
      actions.appendChild(deleteBtn);
    } else {
      const unfolderBtn = document.createElement('button');
      unfolderBtn.className = 'clip-action-btn unfolder-btn';
      unfolderBtn.title = 'Remove from folder';
      unfolderBtn.innerHTML = FOLDER_MINUS_SVG;
      actions.appendChild(unfolderBtn);
    }

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
          <p class="empty-text">No results for "${escapeHtml(query)}"</p>
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
          <p class="empty-hint">Copy anything, text or images, and it will appear here</p>
          <div class="empty-shortcut">
            <span class="shortcut-key">Alt+V</span>
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
    const searchActive = Boolean(query && query.length);
    groups.forEach((group) => {
      fragment.appendChild(buildGroupSection(group, query, searchActive));
    });
    cardList.appendChild(fragment);
    updateCount();
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
    const sections = cardList.querySelectorAll('.group');
    let inserted = false;

    for (const section of sections) {
      if (section.dataset.group === targetLabel) {
        const inner = section.querySelector('.group-items-inner');
        const count = section.querySelector('.group-count');
        if (inner) {
          inner.insertBefore(card, inner.firstChild);
          if (count) count.textContent = parseInt(count.textContent || '0', 10) + 1;
          inserted = true;
        }
        break;
      }
    }

    if (!inserted) {
      let refNode = null;
      for (const section of sections) {
        const idx = order.indexOf(section.dataset.group);
        if (idx > order.indexOf(targetLabel)) {
          refNode = section;
          break;
        }
      }
      const newSection = buildGroupSection({ label: targetLabel, items: [] }, window.search.getQuery(), false);
      const inner = newSection.querySelector('.group-items-inner');
      const count = newSection.querySelector('.group-count');
      inner.insertBefore(card, inner.firstChild);
      if (count) count.textContent = '1';
      if (refNode) {
        cardList.insertBefore(newSection, refNode);
      } else {
        cardList.appendChild(newSection);
      }
    }

    updateCount();
    // New clipboard content changes folder counts too (e.g. the poll path).
    window.folders?.updateCounts?.();

    requestAnimationFrame(() => {
      card.classList.remove('entering');
    });
  }

  function removeCard(id) {
    const card = cardList.querySelector(`[data-id="${id}"]`);
    if (!card) return;

    card.classList.add('removing');
    setTimeout(() => {
      const group = card.closest('.group');
      card.remove();
      if (group && !group.querySelector('.clip-card')) {
        group.remove();
      }
      updateCount();
      window.folders?.updateCounts?.();
      if (!cardList.querySelector('.clip-card') && !cardList.querySelector('.group')) {
        emptyState.style.display = 'flex';
      }
    }, 250);
  }

  function updateCount() {
    const el = document.getElementById('footer-count');
    if (!el) return;
    const n = cardList.querySelectorAll('.clip-card').length;
    el.textContent = n ? `${n} ${n === 1 ? 'entry' : 'entries'}` : '';
  }

  function showLoading() {
    emptyState.style.display = 'none';
    cardList.innerHTML = '';
    for (let i = 0; i < 4; i++) {
      const sk = document.createElement('div');
      sk.className = 'skeleton-card';
      cardList.appendChild(sk);
    }
  }

  function flashCopied(id) {
    const card = cardList.querySelector(`[data-id="${id}"]`);
    if (!card) return;
    card.classList.remove('copied');
    void card.offsetWidth;
    card.classList.add('copied');
    const copyBtn = card.querySelector('.copy-btn');
    const prev = copyBtn ? copyBtn.innerHTML : null;
    if (copyBtn && prev) {
      copyBtn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>';
    }
    setTimeout(() => {
      card.classList.remove('copied');
      if (copyBtn && prev) copyBtn.innerHTML = prev;
    }, 700);
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

  function showClearAllDialog() {
    return new Promise((resolve) => {
      const overlay = document.createElement('div');
      overlay.className = 'confirm-overlay';

      const box = document.createElement('div');
      box.className = 'confirm-box';

      const text = document.createElement('p');
      text.className = 'confirm-text';
      text.textContent = 'Clear clipboard history?';

      const hint = document.createElement('p');
      hint.className = 'confirm-hint';
      hint.textContent =
        'Pinned entries are saved from deletion if you choose. ' +
        'Entries in folders are always kept.';

      const actions = document.createElement('div');
      actions.className = 'confirm-actions confirm-actions-col';

      const cancelBtn = document.createElement('button');
      cancelBtn.className = 'confirm-btn confirm-cancel';
      cancelBtn.textContent = 'Cancel';

      const keepBtn = document.createElement('button');
      keepBtn.className = 'confirm-btn confirm-keep';
      keepBtn.textContent = 'Clear all, keep pinned';

      const allBtn = document.createElement('button');
      allBtn.className = 'confirm-btn confirm-ok';
      allBtn.textContent = 'Delete everything';

      actions.appendChild(keepBtn);
      actions.appendChild(allBtn);
      actions.appendChild(cancelBtn);
      box.appendChild(text);
      box.appendChild(hint);
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

      cancelBtn.addEventListener('click', () => close(null));
      keepBtn.addEventListener('click', () => close('keep'));
      allBtn.addEventListener('click', () => close('all'));
      overlay.addEventListener('click', (e) => {
        if (e.target === overlay) close(null);
      });
    });
  }

  function showAddEntry() {
    return new Promise((resolve) => {
      const overlay = document.createElement('div');
      overlay.className = 'add-entry-overlay';

      const panel = document.createElement('div');
      panel.className = 'add-entry-panel';

      const header = document.createElement('div');
      header.className = 'add-entry-header';
      header.textContent = 'Add to clipboard';
      header.classList.add('draggable-header');

      const dropZone = document.createElement('div');
      dropZone.className = 'add-entry-body';

      const preview = document.createElement('div');
      preview.className = 'add-entry-preview';
      preview.style.display = 'none';

      const textarea = document.createElement('textarea');
      textarea.className = 'add-entry-text';
      textarea.placeholder = 'Type or paste text…';
      textarea.rows = 5;
      textarea.spellcheck = false;

      const hint = document.createElement('p');
      hint.className = 'add-entry-hint';
      hint.textContent = 'Drop an image here, or paste one with Ctrl+V';

      const footer = document.createElement('div');
      footer.className = 'edit-footer';

      const cancelBtn = document.createElement('button');
      cancelBtn.className = 'settings-action-btn';
      cancelBtn.textContent = 'Cancel';

      const addBtn = document.createElement('button');
      addBtn.className = 'settings-action-btn theme-save-btn';
      addBtn.textContent = 'Add';

      footer.appendChild(cancelBtn);
      footer.appendChild(addBtn);

      dropZone.appendChild(preview);
      dropZone.appendChild(textarea);
      dropZone.appendChild(hint);
      panel.appendChild(header);
      panel.appendChild(dropZone);
      panel.appendChild(footer);
      overlay.appendChild(panel);
      document.body.appendChild(overlay);

      let image = null;

      function onKey(ev) {
        if (ev.key === 'Escape') {
          ev.preventDefault();
          ev.stopPropagation();
          cleanup();
          resolve(null);
        }
      }

      function setImage(pending, src, caption) {
        image = pending;
        preview.style.display = 'flex';
        preview.innerHTML = '';
        const thumb = document.createElement('img');
        thumb.className = 'add-entry-thumb';
        thumb.src = src;
        thumb.alt = 'Image to add';
        preview.appendChild(thumb);

        if (caption) {
          const chip = document.createElement('span');
          chip.className = 'add-entry-chip';
          chip.textContent = caption;
          preview.appendChild(chip);
        }

        const remove = document.createElement('button');
        remove.type = 'button';
        remove.className = 'add-entry-remove';
        remove.title = 'Remove image';
        remove.innerHTML =
          '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>';
        remove.addEventListener('click', () => {
          image = null;
          preview.style.display = 'none';
          preview.innerHTML = '';
          addBtn.textContent = 'Add';
        });
        preview.appendChild(remove);
        addBtn.textContent = 'Add image';
      }

      // In-webview decode: used when drop delivers actual File objects.
      async function decodeAndSet(file) {
        addBtn.disabled = true;
        try {
          const blobUrl = URL.createObjectURL(file);
          const bitmap = await createImageBitmap(file);
          let w = bitmap.width;
          let h = bitmap.height;
          const maxW = 3840;
          const maxH = 2160;
          const scale = Math.min(1, maxW / w, maxH / h);
          const canvas = document.createElement('canvas');
          canvas.width = Math.round(w * scale);
          canvas.height = Math.round(h * scale);
          const ctx = canvas.getContext('2d');
          ctx.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
          bitmap.close();
          const pixelData = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
          setImage(
            { rgba: new Uint8Array(pixelData.buffer), width: canvas.width, height: canvas.height },
            blobUrl,
            'Image ' + canvas.width + 'x' + canvas.height
          );
        } catch (err) {
          console.error('Image decode failed:', err);
          window.ui.showToast('Could not read that image');
        } finally {
          addBtn.disabled = false;
        }
      }

      // WebKitGTK delivers drops from other apps as file:// URIs
      // (text/uri-list), not as File objects — the backend decodes those.
      async function decodeAndSetPath(path) {
        addBtn.disabled = true;
        try {
          const info = await window.api.previewImageFile(path);
          setImage(
            { path },
            info.thumb_data_uri,
            info.name + ' · Image ' + info.width + 'x' + info.height
          );
        } catch (err) {
          console.error('Image read failed:', err);
          window.ui.showToast('Could not read that image');
        } finally {
          addBtn.disabled = false;
        }
      }

      dropZone.addEventListener('dragover', (e) => {
        e.preventDefault();
        dropZone.classList.add('drag-over');
      });
      dropZone.addEventListener('dragleave', () => dropZone.classList.remove('drag-over'));
      dropZone.addEventListener('drop', handleDrop);

      // Whole overlay accepts drops too — the dashed zone is a visual target,
      // but a file dragged from another window can land anywhere.
      overlay.addEventListener('dragover', (e) => e.preventDefault());
      overlay.addEventListener('drop', (e) => {
        e.preventDefault();
        dropZone.classList.remove('drag-over');
        handleDrop(e);
      });

      function handleDrop(e) {
        const file = e.dataTransfer?.files?.[0];
        if (file && file.type.startsWith('image/')) {
          decodeAndSet(file);
          return;
        }
        if (file && file.type === 'text/plain') {
          file.text().then((t) => { textarea.value = (textarea.value + t).trim(); });
          return;
        }
        // WebKitGTK: drops from other apps carry file:// URIs instead of File objects
        const uriData = e.dataTransfer.getData('text/uri-list') || e.dataTransfer.getData('text/plain') || '';
        const uri = uriData.split(/\r?\n/).map((s) => s.trim()).find((u) => u.startsWith('file://'));
        if (uri) {
          let path;
          try {
            path = decodeURIComponent(new URL(uri).pathname);
          } catch (_) {
            path = null;
          }
          if (path) decodeAndSetPath(path);
          return;
        }
        // Image dragged from another browser window arrives as a data URI
        const dataUri = uriData.split(/\r?\n/).map((s) => s.trim()).find((u) => u.startsWith('data:image/'));
        if (dataUri && dataUri.length < 3 * 1024 * 1024) {
          try {
            const bin = atob(dataUri.split(',')[1]);
            const bytes = new Uint8Array(bin.length);
            for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
            const file2 = new File([bytes], 'dropped-image', { type: 'image/png' });
            decodeAndSet(file2);
          } catch (_) { /* malformed data URI */ }
        }
      }

      // Moving the window from inside the modal: drive real GTK
      // gtk_window_begin_move_drag via the backend (webview-side
      // startDragging / drag-region is unreliable on this webview).
      header.addEventListener('mousedown', (e) => {
        if (e.button !== 0) return;
        e.preventDefault();
        window.api.beginWindowDrag().catch((err) => console.error('Drag failed:', err));
      });

      // Native drag-drop events. wry intercepts OS file drops at the GTK
      // layer and emits tauri://drag-* events; the webview never receives
      // them as DOM DragEvents on WebKitGTK (webkit bug 265857), so DOM
      // drop handlers alone cannot work here.
      const eventApi = window.__TAURI__?.event;
      const unlisteners = [];
      if (eventApi?.listen) {
        const dragOverClass = 'drag-over';
        Promise.all([
          eventApi.listen('tauri://drag-enter', () => dropZone.classList.add(dragOverClass)),
          eventApi.listen('tauri://drag-over', () => dropZone.classList.add(dragOverClass)),
          eventApi.listen('tauri://drag-leave', () => dropZone.classList.remove(dragOverClass)),
          eventApi.listen('tauri://drag-drop', (e) => {
            dropZone.classList.remove(dragOverClass);
            const paths = (e.payload || {}).paths || [];
            if (paths.length > 0) decodeAndSetPath(paths[0]);
          }),
        ]).then((uns) => unlisteners.push(...uns)).catch((err) => {
          console.error('Failed to register drag events:', err);
        });
      }

      // Suppress the webview's default "open the dropped file" behavior
      const suppress = (e) => e.preventDefault();
      document.addEventListener('dragover', suppress);
      document.addEventListener('drop', suppress);

      function cleanup() {
        document.removeEventListener('keydown', onKey, true);
        document.removeEventListener('dragover', suppress);
        document.removeEventListener('drop', suppress);
        unlisteners.forEach((u) => { try { u(); } catch (_) {} });
        overlay.classList.remove('folder-picker-visible');
        setTimeout(() => overlay.remove(), 160);
      }

      textarea.addEventListener('paste', (e) => {
        const item = Array.from(e.clipboardData?.items || []).find((it) => it.type.startsWith('image/'));
        if (item) {
          e.preventDefault();
          const file = item.getAsFile();
          if (file) decodeAndSet(file);
        }
      });

      async function submit() {
        const folderId = window.folders ? window.folders.getCurrentId() : null;
        if (image && image.path) {
          try {
            const entry = await window.api.addImageFileEntry(image.path, folderId);
            cleanup();
            resolve('image');
            window.ui.showToast('Image copied to clipboard');
            loadEntriesRef('');
          } catch (err) {
            console.error('Add image failed:', err);
            window.ui.showToast(String(err?.message || err).replace(/^Error['"]?\s*:?\s*/i, '').trim() || 'Failed to add image');
          }
          return;
        }
        if (image) {
          try {
            await window.api.addImageToClipboard(image.rgba, image.width, image.height, folderId);
            cleanup();
            resolve('image');
            window.ui.showToast('Image copied to clipboard');
            loadEntriesRef('');
          } catch (err) {
            console.error('Add image failed:', err);
            window.ui.showToast(String(err?.message || err).replace(/^Error['"]?\s*:?\s*/i, '').trim() || 'Failed to add image');
          }
          return;
        }
        const text = textarea.value.trim();
        if (!text) {
          window.ui.showToast('Type text or drop an image first');
          return;
        }
        try {
          await window.api.addTextEntry(text, folderId);
          cleanup();
          resolve('text');
          window.ui.showToast('Added to clipboard');
          loadEntriesRef('');
        } catch (err) {
          console.error('Add entry failed:', err);
          window.ui.showToast(String(err?.message || err).replace(/^Error['"]?\s*:?\s*/i, '').trim() || 'Failed to add entry');
        }
      }

      addBtn.addEventListener('click', submit);
      cancelBtn.addEventListener('click', () => {
        cleanup();
        resolve(null);
      });
      // Deliberately NOT closing on backdrop click: while adding, the user
      // drags a file from another window, so stray outside clicks must not
      // dismiss the dialog. Cancel button / Escape close it.
      header.addEventListener('dblclick', (e) => e.stopPropagation());

      document.addEventListener('keydown', onKey, true);
      requestAnimationFrame(() => overlay.classList.add('folder-picker-visible'));
      setTimeout(() => textarea.focus(), 80);
    });
  }

  function showFolderPicker(cardId) {
    return new Promise((resolve) => {
      const overlay = document.createElement('div');
      overlay.className = 'folder-picker-overlay';

      const panel = document.createElement('div');
      panel.className = 'folder-picker';

      const header = document.createElement('div');
      header.className = 'folder-picker-header';
      header.textContent = 'Add to folder';

      const listEl = document.createElement('div');
      listEl.className = 'folder-picker-list';

      function close(result) {
        overlay.classList.remove('folder-picker-visible');
        setTimeout(() => overlay.remove(), 200);
        resolve(result);
      }

      function closeOnKey(ev) {
        if (ev.key === 'Escape') {
          ev.preventDefault();
          ev.stopPropagation();
          document.removeEventListener('keydown', closeOnKey, true);
          close(null);
        }
      }

      window.api.getFolders().then((folders) => {
        if (!folders || folders.length === 0) {
          const empty = document.createElement('div');
          empty.className = 'folder-picker-empty';
          empty.textContent = 'No folders yet — create one in the sidebar';
          listEl.appendChild(empty);
        } else {
          folders.forEach((folder) => {
            const btn = document.createElement('button');
            btn.type = 'button';
            btn.className = 'folder-picker-btn';
            btn.title = 'Add to ' + folder.name;
            btn.textContent = folder.name;
            btn.addEventListener('click', async () => {
              document.removeEventListener('keydown', closeOnKey, true);
              try {
                await window.api.moveEntryToFolder(cardId, folder.id);
                close(folder.id);
                loadEntriesRef('');
                window.ui.showToast('Added to "' + folder.name + '"');
              } catch (err) {
                console.error('Move to folder failed:', err);
                close(null);
                window.ui.showToast('Failed to add to folder');
              }
            });
            listEl.appendChild(btn);
          });
        }
      }).catch((err) => {
        console.error('Failed to load folders:', err);
        const empty = document.createElement('div');
        empty.className = 'folder-picker-empty';
        empty.textContent = 'Failed to load folders';
        listEl.appendChild(empty);
      });

      overlay.addEventListener('click', (e) => {
        if (e.target === overlay) {
          document.removeEventListener('keydown', closeOnKey, true);
          close(null);
        }
      });

      panel.appendChild(header);
      panel.appendChild(listEl);
      overlay.appendChild(panel);
      document.body.appendChild(overlay);
      document.addEventListener('keydown', closeOnKey, true);
      requestAnimationFrame(() => {
        overlay.classList.add('folder-picker-visible');
      });
    });
  }

  function showError(message) {
    const el = document.getElementById('empty-state');
    if (!el) return;
    el.style.display = 'flex';
    if (!el.parentNode) cardList.appendChild(el);
    // Error strings can contain clipboard content (failed loads echo the raw
    // error); never interpolate them into innerHTML unescaped.
    el.innerHTML = `
      <p class="empty-text" style="color:var(--danger)"></p>
      <button id="btn-retry" class="filter-btn" style="margin-top:8px">Retry</button>
    `;
    const textEl = el.querySelector('.empty-text');
    if (textEl) textEl.textContent = message;
    const retry = document.getElementById('btn-retry');
    if (retry) {
      retry.addEventListener('click', () => {
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

  return { renderCards, prependCard, removeCard, updatePinState, showToast, showConfirm, showClearAllDialog, showFolderPicker, showAddEntry, showError, setLoadEntries, setCollapsedSections, setCollapsePersist, showLockScreen, hideLockScreen, showPasswordSetup, lockShake, setLockError, showSettings, hideSettings, showEdit, hideEdit, showFilterPanel, hideFilterPanel, updateFilterBadge, renderFilterList, showLoading, flashCopied, updateCount };
})();
