<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  const dispatch = createEventDispatcher();

  interface Draft {
    draft_id: number | null;
    to_addr: string;
    cc: string;
    bcc: string;
    subject: string;
    body: string;
    in_reply_to: string | null;
    ref_headers: string | null;
    reply_mode: string;
    updated_at: number;
  }

  let drafts: Draft[] = [];
  let loading = true;
  let error = '';

  async function loadDrafts() {
    loading = true;
    error = '';
    try {
      drafts = await invoke<Draft[]>('get_drafts');
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function formatTime(epoch: number): string {
    if (!epoch) return '';
    const d = new Date(epoch * 1000);
    return d.toLocaleString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    });
  }

  function resumeDraft(draft: Draft) {
    dispatch('resume', draft);
  }

  async function deleteDraft(draftId: number, event: MouseEvent) {
    event.stopPropagation();
    try {
      await invoke('delete_draft', { draftId });
      drafts = drafts.filter((d) => d.draft_id !== draftId);
    } catch {
      // non-critical
    }
  }

  onMount(() => {
    loadDrafts();
  });
</script>

<div class="drafts-list">
  <div class="drafts-header">
    <h2>Drafts</h2>
  </div>

  {#if loading}
    <p class="status">Loading drafts...</p>
  {:else if error}
    <p class="status error">{error}</p>
  {:else if drafts.length === 0}
    <p class="status">No drafts</p>
  {:else}
    {#each drafts as draft}
      <div class="draft-item" on:click={() => resumeDraft(draft)} on:keydown={() => {}} role="button" tabindex="0">
        <div class="draft-row">
          <span class="draft-subject">{draft.subject || '(no subject)'}</span>
          <span class="draft-time">{formatTime(draft.updated_at)}</span>
        </div>
        <div class="draft-to">{draft.to_addr || '(no recipient)'}</div>
        {#if draft.draft_id != null}
          <button
            class="delete-btn"
            type="button"
            on:click={(e) => deleteDraft(draft.draft_id, e)}
          >Delete</button>
        {/if}
      </div>
    {/each}
  {/if}
</div>

<style>
  .drafts-list {
    padding: 16px 20px;
  }

  .drafts-header h2 {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 16px 0;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }

  .status {
    color: var(--text-dim);
    font-size: 13px;
  }

  .status.error {
    color: #ef4444;
  }

  .draft-item {
    display: block;
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-secondary);
    text-align: left;
    cursor: pointer;
    color: var(--text-primary);
    font-family: inherit;
    font-size: inherit;
    margin-bottom: 8px;
    position: relative;
  }

  .draft-item:hover {
    border-color: var(--accent);
  }

  .draft-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
  }

  .draft-subject {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .draft-time {
    color: var(--text-dim);
    font-size: 11px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .draft-to {
    color: var(--text-secondary);
    font-size: 12px;
    margin-top: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .delete-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    background: none;
    border: 1px solid var(--border);
    color: var(--text-dim);
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 3px;
    cursor: pointer;
    font-family: monospace;
    display: none;
  }

  .draft-item:hover .delete-btn {
    display: block;
  }

  .delete-btn:hover {
    color: #ef4444;
    border-color: #ef4444;
  }
</style>
