<script lang="ts">
  import { listDirectories } from '../api';
  import type { DirectoryEntry } from '../types';

  let {
    value = $bindable('.'),
    disabled = false,
  }: {
    value?: string;
    disabled?: boolean;
  } = $props();

  let open = $state(false);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let currentPath = $state('');
  let parentPath = $state<string | null>(null);
  let directories = $state<DirectoryEntry[]>([]);

  async function loadDirectories(path: string) {
    loading = true;
    error = null;
    try {
      const response = await listDirectories(path);
      currentPath = response.currentPath;
      parentPath = response.parentPath;
      directories = response.directories;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load directories';
    } finally {
      loading = false;
    }
  }

  function handleOpen() {
    if (disabled) return;
    open = true;
    loadDirectories(value || '.');
  }

  function handleClose() {
    open = false;
  }

  function handleSelect() {
    value = currentPath;
    open = false;
  }

  function navigateTo(path: string) {
    loadDirectories(path);
  }

  function navigateUp() {
    if (parentPath) {
      loadDirectories(parentPath);
    }
  }

  // Get display path (show '.' for current directory)
  function getDisplayPath(path: string): string {
    return path || '.';
  }

  // Get breadcrumb segments from path
  function getBreadcrumbs(path: string): { name: string; path: string }[] {
    if (!path) return [];
    const parts = path.split('/').filter(Boolean);
    const breadcrumbs: { name: string; path: string }[] = [];
    let accumulated = '';
    for (const part of parts) {
      accumulated += '/' + part;
      breadcrumbs.push({ name: part, path: accumulated });
    }
    return breadcrumbs;
  }
</script>

<div class="folder-picker">
  <div class="input-wrapper">
    <input
      type="text"
      bind:value={value}
      placeholder="Select folder..."
    />
    <button type="button" class="browse-btn" {disabled} onclick={handleOpen}>
      Browse
    </button>
  </div>

  {#if open}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-overlay" onclick={handleClose} onkeydown={(e) => e.key === 'Escape' && handleClose()}>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="modal" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
        <div class="modal-header">
          <h3>Select Folder</h3>
          <button type="button" class="close-btn" onclick={handleClose}>&times;</button>
        </div>

        <div class="breadcrumbs">
          {#each getBreadcrumbs(currentPath) as crumb, i}
            {#if i > 0}
              <span class="separator">/</span>
            {/if}
            <button type="button" class="crumb" onclick={() => navigateTo(crumb.path)}>
              {crumb.name}
            </button>
          {/each}
        </div>

        <div class="directory-list">
          {#if loading}
            <div class="loading">Loading...</div>
          {:else if error}
            <div class="error">{error}</div>
          {:else}
            {#if parentPath !== null}
              <button type="button" class="directory-item parent" onclick={navigateUp}>
                <span class="folder-icon">..</span>
                <span class="folder-name">Parent Directory</span>
              </button>
            {/if}
            {#if directories.length === 0}
              <div class="empty">No subdirectories</div>
            {:else}
              {#each directories as dir}
                <button type="button" class="directory-item" onclick={() => navigateTo(dir.path)}>
                  <span class="folder-icon">&#128193;</span>
                  <span class="folder-name">{dir.name}</span>
                </button>
              {/each}
            {/if}
          {/if}
        </div>

        <div class="modal-footer">
          <div class="selected-path" title={currentPath}>
            {currentPath}
          </div>
          <div class="modal-actions">
            <button type="button" class="cancel-btn" onclick={handleClose}>Cancel</button>
            <button type="button" class="select-btn" onclick={handleSelect}>Select</button>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .folder-picker {
    position: relative;
  }

  .input-wrapper {
    display: flex;
    gap: 0.5rem;
  }

  .input-wrapper input {
    flex: 1;
    cursor: pointer;
  }

  .input-wrapper input:disabled {
    cursor: not-allowed;
  }

  .browse-btn {
    padding: 0.5rem 1rem;
    background: var(--primary);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s ease;
  }

  .browse-btn:hover:not(:disabled) {
    background: var(--secondary);
  }

  .browse-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--bg-card);
    border-radius: 12px;
    width: 90%;
    max-width: 500px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border-color);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .modal-header h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: var(--text-muted);
    padding: 0;
    line-height: 1;
    box-shadow: none;
  }

  .close-btn:hover {
    color: inherit;
    transform: none;
    box-shadow: none;
  }

  .breadcrumbs {
    padding: 0.75rem 1.25rem;
    background: rgba(0, 0, 0, 0.1);
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.85rem;
    overflow-x: auto;
  }

  .separator {
    color: var(--text-muted);
  }

  .crumb {
    background: none;
    border: none;
    color: var(--primary);
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.85rem;
    box-shadow: none;
  }

  .crumb:hover {
    background: rgba(108, 92, 231, 0.1);
    transform: none;
    box-shadow: none;
  }

  .directory-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
    min-height: 200px;
    max-height: 300px;
  }

  .directory-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    padding: 0.75rem 1rem;
    background: none;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    transition: background 0.2s ease;
    box-shadow: none;
  }

  .directory-item:hover {
    background: rgba(108, 92, 231, 0.1);
    transform: none;
    box-shadow: none;
  }

  .directory-item.parent {
    color: var(--text-muted);
  }

  .folder-icon {
    font-size: 1.25rem;
  }

  .folder-name {
    color: var(--text-muted);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .loading,
  .error,
  .empty {
    padding: 2rem;
    text-align: center;
    color: var(--text-muted);
  }

  .error {
    color: #e74c3c;
  }

  .modal-footer {
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .selected-path {
    font-size: 0.8rem;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }

  .cancel-btn,
  .select-btn {
    padding: 0.5rem 1.25rem;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .cancel-btn {
    background: transparent;
    border: 1px solid var(--border-color);
    color: inherit;
  }

  .cancel-btn:hover {
    background: rgba(255, 255, 255, 0.05);
    transform: none;
    box-shadow: none;
  }

  .select-btn {
    background: var(--primary);
    border: none;
    color: white;
  }

  .select-btn:hover {
    background: var(--secondary);
  }

  @media (prefers-color-scheme: light) {
    .breadcrumbs {
      background: rgba(0, 0, 0, 0.03);
    }

    .cancel-btn:hover {
      background: rgba(0, 0, 0, 0.05);
    }
  }
</style>
