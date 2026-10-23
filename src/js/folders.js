window.folders = (() => {
  let folders = [];
  let currentFolderId = null;
  let changeListeners = [];

  const list = document.getElementById('folder-list');
  const createForm = document.getElementById('folder-create-form');
  const createInput = document.getElementById('folder-create-input');
  const btnSidebar = document.getElementById('btn-sidebar');
  const contentRow = document.getElementById('content-row');

  const viewHeader = document.getElementById('folder-view-header');
  const viewName = document.getElementById('folder-view-name');
  const viewCount = document.getElementById('folder-view-count');
  const btnRename = document.getElementById('btn-folder-rename');
  const btnDelete = document.getElementById('btn-folder-delete');

  const FOLDER_SVG =
    '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4l2 3h9a2 2 0 0 1 2 2z"/></svg>';
  const LIST_SVG =
    '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>';

  function notify() {
    changeListeners.forEach((fn) => fn());
  }

  function select(id) {
    currentFolderId = id;
    render();
    notify();
  }

  // --- Folder view header (main window) ---
  function updateViewHeader() {
    if (!viewHeader) return;
    const current = currentFolderId !== null ? folders.find((f) => f.id === currentFolderId) : null;
    if (!current) {
      viewHeader.style.display = 'none';
      return;
    }
    viewHeader.style.display = 'flex';
    viewName.textContent = current.name;
    viewName.title = current.name;
    viewCount.textContent = current.entry_count === 1 ? '1 entry' : current.entry_count + ' entries';
  }

  function startRename() {
    const current = folders.find((f) => f.id === currentFolderId);
    if (!current || viewName.querySelector('input')) return;

    const input = document.createElement('input');
    input.type = 'text';
    input.className = 'folder-rename-input';
    input.value = current.name;
    input.maxLength = 64;
    viewName.textContent = '';
    viewName.appendChild(input);
    input.focus();
    input.select();

    function finish(save) {
      const val = input.value.trim();
      if (save && val && val !== current.name) {
        window.api.renameFolder(current.id, val).then(() => {
          refresh().catch(() => {});
        }).catch((err) => {
          console.error('Rename failed:', err);
          window.ui.showToast('Rename failed');
        });
      } else {
        refresh().catch(() => {});
      }
    }

    input.addEventListener('keydown', (ev) => {
      if (ev.key === 'Enter') { ev.preventDefault(); ev.stopPropagation(); finish(true); }
      if (ev.key === 'Escape') { ev.preventDefault(); finish(false); }
    });
    input.addEventListener('blur', () => finish(true));
  }

  async function deleteCurrent() {
    const current = folders.find((f) => f.id === currentFolderId);
    if (!current) return;
    const confirmed = await window.ui.showConfirm(
      `Delete folder "${current.name}"?` +
      ` Its ${current.entry_count === 1 ? 'entry returns' : current.entry_count + ' entries return'} to history.`
    );
    if (!confirmed) return;
    try {
      await window.api.deleteFolder(current.id);
      select(null);
      await refresh();
      notify();
      window.ui.showToast('Folder deleted');
    } catch (err) {
      console.error('Folder delete failed:', err);
      window.ui.showToast('Folder delete failed');
    }
  }

  btnRename?.addEventListener('click', startRename);
  btnDelete?.addEventListener('click', deleteCurrent);

  // --- Sidebar ---
  function buildAllItem() {
    const item = document.createElement('button');
    item.type = 'button';
    item.className = 'folder-item' + (currentFolderId === null ? ' active' : '');
    item.dataset.folderId = '';

    const icon = document.createElement('span');
    icon.className = 'folder-item-icon';
    icon.innerHTML = LIST_SVG;
    item.appendChild(icon);

    const label = document.createElement('span');
    label.className = 'folder-label-text';
    label.textContent = 'All History';
    item.appendChild(label);

    item.addEventListener('click', () => {
      if (currentFolderId !== null) select(null);
    });
    return item;
  }

  function buildFolderItem(folder) {
    const item = document.createElement('div');
    item.className = 'folder-item' + (currentFolderId === folder.id ? ' active' : '');
    item.dataset.folderId = String(folder.id);

    const icon = document.createElement('span');
    icon.className = 'folder-item-icon';
    icon.innerHTML = FOLDER_SVG;
    item.appendChild(icon);

    const label = document.createElement('span');
    label.className = 'folder-label-text';
    label.textContent = folder.name;
    label.title = folder.name;
    item.appendChild(label);

    const count = document.createElement('span');
    count.className = 'folder-count';
    count.textContent = folder.entry_count;
    if (folder.entry_count > 0) item.appendChild(count);

    label.addEventListener('click', () => select(folder.id));
    icon.addEventListener('click', () => select(folder.id));

    return item;
  }

  async function refresh() {
    folders = await window.api.getFolders();
    // Alphabetical (case-insensitive); the sidebar always renders
    // "All History" first, so only folder items need sorting.
    folders.sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }));
    render();
  }

  /// Re-fetch counts and re-render WITHOUT notifying (no pane reload). Used
  /// whenever entries change (new copies, deletes, moves) so sidebar counts
  /// and the folder-view header stay current.
  function updateCounts() {
    return window.api.getFolders()
      .then((fresh) => {
        folders = fresh;
        render();
      })
      .catch((err) => console.error('Failed to refresh folder counts:', err));
  }

  function render() {
    if (list) {
      list.innerHTML = '';
      list.appendChild(buildAllItem());
      folders.forEach((folder) => list.appendChild(buildFolderItem(folder)));
    }
    updateViewHeader();
  }

  // --- Sidebar show/hide (hidden by default; toggled from the header) ---
  function setSidebarVisible(visible) {
    contentRow.classList.toggle('sidebar-hidden', !visible);
    if (btnSidebar) btnSidebar.classList.toggle('active', visible);
  }

  btnSidebar?.addEventListener('click', () => {
    setSidebarVisible(contentRow.classList.contains('sidebar-hidden'));
  });

  // --- Create folder ---
  createForm?.addEventListener('submit', async (e) => {
    e.preventDefault();
    const name = createInput.value.trim();
    if (!name) return;
    try {
      const folder = await window.api.createFolder(name);
      createInput.value = '';
      await refresh();
      select(folder.id);
    } catch (err) {
      console.error('Folder create failed:', err);
      const msg = String(err?.message || err).replace(/^Error['"]?\s*:?\s*/i, '').trim();
      window.ui.showToast(msg || 'Create folder failed');
    }
  });

  // Load existing folders on startup (async, best effort)
  refresh().catch((err) => console.error('Failed to load folders:', err));

  return {
    getCurrentId: () => currentFolderId,
    isFolderView: () => currentFolderId !== null,
    select,
    refresh,
    updateCounts,
    onChange: (fn) => changeListeners.push(fn),
  };
})();
