(async () => {
  const config = await theme.load();

  async function loadEntries(query, filter) {
    try {
      const entries = await api.getEntries(query || null, filter !== 'all' ? filter : null);
      ui.renderCards(entries, query || '');
    } catch (err) {
      console.error('Failed to load entries:', err);
    }
  }

  function onSearch(query, filter) {
    loadEntries(query, filter);
  }

  search.init(onSearch);

  const cardList = document.getElementById('card-list');
  cardList.addEventListener('click', async (e) => {
    const card = e.target.closest('.clip-card');
    if (!card) return;

    const id = parseInt(card.dataset.id);
    const btn = e.target.closest('.clip-action-btn');

    if (!btn) {
      try {
        const entry = await api.getEntries(null, null);
        const found = entry.find((e) => e.id === id);
        if (found) {
          await navigator.clipboard.writeText(found.content);
        }
      } catch (err) {
        console.error('Copy failed:', err);
      }
      return;
    }

    if (btn.classList.contains('copy-btn')) {
      try {
        const entries = await api.getEntries(null, null);
        const entry = entries.find((e) => e.id === id);
        if (entry) {
          await navigator.clipboard.writeText(entry.content);
        }
      } catch (err) {
        console.error('Copy failed:', err);
      }
    } else if (btn.classList.contains('pin-btn')) {
      try {
        await api.togglePin(id);
        ui.updatePinState(id, !card.classList.contains('pinned'));
        loadEntries(search.getQuery(), search.getFilter());
      } catch (err) {
        console.error('Toggle pin failed:', err);
      }
    } else if (btn.classList.contains('delete-btn')) {
      try {
        await api.deleteEntry(id);
        ui.removeCard(id);
      } catch (err) {
        console.error('Delete failed:', err);
      }
    }
  });

  let selectedIndex = -1;
  document.addEventListener('keydown', (e) => {
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
      cards[selectedIndex].click();
    } else if (e.key === 'Delete' && selectedIndex >= 0) {
      e.preventDefault();
      const deleteBtn = cards[selectedIndex].querySelector('.delete-btn');
      if (deleteBtn) deleteBtn.click();
    }
  });

  try {
    const { listen } = window.__TAURI__.event;
    await listen('clipboard-changed', (event) => {
      ui.prependCard(event.payload);
    });
  } catch (err) {
    console.error('Failed to listen for clipboard events:', err);
  }

  loadEntries('', 'all');
})();
