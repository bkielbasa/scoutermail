<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let activeFolder: string = 'INBOX';

  const dispatch = createEventDispatcher<{ select: string }>();

  type FolderCount = [string, number, number]; // [name, total, unread]

  let folders: FolderCount[] = [];

  // Common folders shown at top in this order
  const priorityFolders = ['INBOX', 'Sent', 'Drafts', 'Archive', 'Trash'];

  $: sortedFolders = sortFolders(folders);

  function sortFolders(raw: FolderCount[]): FolderCount[] {
    const prioritySet = new Set(priorityFolders.map((f) => f.toLowerCase()));
    const top: FolderCount[] = [];
    const rest: FolderCount[] = [];

    for (const f of raw) {
      if (prioritySet.has(f[0].toLowerCase())) {
        top.push(f);
      } else {
        rest.push(f);
      }
    }

    // Sort top by priority order
    top.sort((a, b) => {
      const ai = priorityFolders.findIndex((p) => p.toLowerCase() === a[0].toLowerCase());
      const bi = priorityFolders.findIndex((p) => p.toLowerCase() === b[0].toLowerCase());
      return ai - bi;
    });

    // Rest sorted alphabetically
    rest.sort((a, b) => a[0].localeCompare(b[0]));

    return [...top, ...rest];
  }

  async function loadCounts(): Promise<void> {
    try {
      folders = await invoke<FolderCount[]>('get_folder_counts');
    } catch {
      folders = [];
    }
  }

  function handleSelect(name: string): void {
    dispatch('select', name);
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
      dispatch('select', activeFolder);
    }
  }

  onMount(() => {
    loadCounts();
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="folder-list">
  <div class="folder-header">Folders</div>
  {#each sortedFolders as [name, total, unread]}
    <button
      class="folder-item"
      class:active={name === activeFolder}
      on:click={() => handleSelect(name)}
    >
      <span class="folder-name">{name}</span>
      {#if unread > 0}
        <span class="unread-badge">{unread}</span>
      {/if}
    </button>
  {/each}
  {#if sortedFolders.length === 0}
    <div class="empty">No folders found</div>
  {/if}
</div>

<style>
  .folder-list {
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    width: 200px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    z-index: 20;
    display: flex;
    flex-direction: column;
    font-family: var(--font-mono);
    font-size: 13px;
    overflow-y: auto;
  }
  .folder-header {
    padding: 10px 12px 6px;
    color: var(--text-dim);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .folder-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    text-align: left;
    font-family: inherit;
    font-size: inherit;
    width: 100%;
  }
  .folder-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .folder-item.active {
    background: var(--bg-selected);
    color: var(--accent);
  }
  .folder-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .unread-badge {
    background: var(--accent);
    color: var(--bg-primary);
    font-size: 11px;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: 8px;
    min-width: 18px;
    text-align: center;
    flex-shrink: 0;
    margin-left: 8px;
  }
  .empty {
    padding: 12px;
    color: var(--text-dim);
    font-size: 12px;
  }
</style>
