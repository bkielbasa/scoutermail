<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface AttachmentInfo {
    attachment_id: number;
    uid: number;
    folder: string;
    filename: string | null;
    mime_type: string | null;
    size: number | null;
  }

  export let uid: number;
  export let folder: string;

  let attachments: AttachmentInfo[] = [];
  let savingId: number | null = null;
  let savedMessage: string | null = null;
  let savedTimeout: ReturnType<typeof setTimeout> | null = null;

  $: if (uid && folder) {
    loadAttachments(uid, folder);
  }

  async function loadAttachments(u: number, f: string) {
    try {
      attachments = await invoke<AttachmentInfo[]>('get_attachments', { uid: u, folder: f });
    } catch {
      attachments = [];
    }
  }

  function formatSize(bytes: number | null): string {
    if (bytes === null || bytes === undefined) return '';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  async function downloadAttachment(att: AttachmentInfo) {
    savingId = att.attachment_id;
    savedMessage = null;
    if (savedTimeout) clearTimeout(savedTimeout);
    try {
      const path = await invoke<string>('save_attachment', { attachmentId: att.attachment_id });
      savedMessage = `Saved to ${path}`;
      savedTimeout = setTimeout(() => { savedMessage = null; }, 3000);
    } catch (e) {
      savedMessage = `Error: ${e}`;
      savedTimeout = setTimeout(() => { savedMessage = null; }, 3000);
    } finally {
      savingId = null;
    }
  }
</script>

{#if attachments.length > 0}
  <div class="attachment-list">
    <div class="attachment-chips">
      {#each attachments as att (att.attachment_id)}
        <button
          class="attachment-chip"
          on:click={() => downloadAttachment(att)}
          disabled={savingId === att.attachment_id}
          title="Save to Downloads"
        >
          <span class="clip-icon">&#x1F4CE;</span>
          <span class="att-name">{att.filename || 'attachment'}</span>
          {#if att.size}
            <span class="att-size">({formatSize(att.size)})</span>
          {/if}
        </button>
      {/each}
    </div>
    {#if savedMessage}
      <div class="saved-msg">{savedMessage}</div>
    {/if}
  </div>
{/if}

<style>
  .attachment-list {
    margin-bottom: 12px;
  }

  .attachment-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .attachment-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-secondary);
    font-size: 12px;
    font-family: inherit;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }

  .attachment-chip:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .attachment-chip:disabled {
    opacity: 0.5;
    cursor: wait;
  }

  .clip-icon {
    font-size: 13px;
  }

  .att-name {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .att-size {
    color: var(--text-dim);
    font-size: 11px;
  }

  .saved-msg {
    margin-top: 6px;
    font-size: 11px;
    color: var(--text-dim);
  }
</style>
