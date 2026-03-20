<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { searchOpen, searchQuery } from '$lib/stores/ui';
  import { messages, selectedIndex, type Message } from '$lib/stores/messages';
  import { registerHandler } from '$lib/keybindings/engine';

  let inputEl: HTMLInputElement;
  let query = '';
  let error = '';

  onMount(async () => {
    await tick();
    if (inputEl) inputEl.focus();

    registerHandler('search-execute', executeSearch);
  });

  async function executeSearch(): Promise<void> {
    if (!query.trim()) return;
    error = '';
    try {
      const results = await invoke<Message[]>('search_messages', { query, limit: 50 });
      messages.set(results);
      selectedIndex.set(0);
      searchQuery.set(query);
      close();
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  function close(): void {
    searchOpen.set(false);
    query = '';
    error = '';
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      close();
    } else if (event.key === 'Enter') {
      event.preventDefault();
      event.stopPropagation();
      executeSearch();
    }
  }
</script>

<div class="search-bar">
  <span class="search-icon">/</span>
  <input
    bind:this={inputEl}
    bind:value={query}
    class="search-input"
    type="text"
    placeholder="Search messages..."
    on:keydown={handleKeydown}
  />
  <span class="search-hint">enter search &middot; esc cancel</span>
</div>
{#if error}
  <div class="search-error">{error}</div>
{/if}

<style>
  .search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .search-icon {
    font-family: var(--font-mono);
    font-size: 14px;
    color: var(--accent);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 13px;
    outline: none;
  }

  .search-input::placeholder {
    color: var(--text-dim);
  }

  .search-hint {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-dim);
    flex-shrink: 0;
  }

  .search-error {
    padding: 4px 12px;
    background: rgba(239, 68, 68, 0.1);
    color: #ef4444;
    font-size: 12px;
    border-bottom: 1px solid var(--border);
  }
</style>
