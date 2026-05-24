const ui = (() => {
  const cardList = document.getElementById('card-list');
  const emptyState = document.getElementById('empty-state');

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
    preview.innerHTML = search.highlight(
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
      return;
    }

    emptyState.style.display = 'none';

    const fragment = document.createDocumentFragment();
    entries.forEach((entry) => {
      fragment.appendChild(createCard(entry, query));
    });
    cardList.appendChild(fragment);
  }

  function prependCard(entry) {
    emptyState.style.display = 'none';
    const existing = cardList.querySelector(`[data-id="${entry.id}"]`);
    if (existing) return;

    const card = createCard(entry, search.getQuery());
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

  return { renderCards, prependCard, removeCard, updatePinState };
})();
