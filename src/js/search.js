window.search = (() => {
  let debounceTimer = null;
  let currentQuery = '';
  let currentFilter = 'all';
  let currentApp = '';
  let onSearchCallback = null;

  const input = document.getElementById('search-input');
  const filterBtns = document.querySelectorAll('.filter-btn');

  function init(onSearch) {
    onSearchCallback = onSearch;

    const kbd = document.getElementById('search-kbd');
    const clear = document.getElementById('btn-search-clear');

    function syncButtons() {
      const hasText = input.value.length > 0;
      if (kbd) kbd.style.display = hasText ? 'none' : '';
      if (clear) clear.style.display = hasText ? 'flex' : 'none';
    }

    input.addEventListener('input', () => {
      clearTimeout(debounceTimer);
      syncButtons();
      debounceTimer = setTimeout(() => {
        currentQuery = input.value.trim();
        onSearchCallback(currentQuery, currentFilter);
      }, 300);
    });

    if (clear) {
      clear.addEventListener('click', () => {
        input.value = '';
        currentQuery = '';
        syncButtons();
        input.focus();
        if (onSearchCallback) onSearchCallback(currentQuery, currentFilter);
      });
    }

    filterBtns.forEach((btn) => {
      btn.addEventListener('click', () => {
        filterBtns.forEach((b) => b.classList.remove('active'));
        btn.classList.add('active');
        currentFilter = btn.dataset.filter;
        onSearchCallback(currentQuery, currentFilter);
      });
    });
  }

  function getQuery() {
    return currentQuery;
  }

  function getFilter() {
    return currentFilter;
  }

  function getAppFilter() {
    return currentApp;
  }

  function setAppFilter(app) {
    currentApp = app;
    if (onSearchCallback) onSearchCallback(currentQuery, currentFilter);
  }

  function highlight(text, query) {
    if (!query) return text;
    const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const regex = new RegExp(`(${escaped})`, 'gi');
    return text.replace(regex, '<span class="highlight">$1</span>');
  }

  return { init, getQuery, getFilter, getAppFilter, setAppFilter, highlight };
})();