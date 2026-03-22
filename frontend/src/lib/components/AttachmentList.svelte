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

  let previewAttachment: AttachmentInfo | null = null;
  let previewData: string | null = null;
  let previewMime: string | null = null;
  let loadingPreview = false;

  $: if (uid && folder) {
    loadAttachments(uid, folder);
  }

  async function loadAttachments(u: number, f: string) {
    try {
      attachments = await invoke<AttachmentInfo[]>('get_attachments', { uid: u, folder: f });
    } catch {
      attachments = [];
    }
    // Close preview when message changes
    previewAttachment = null;
    previewData = null;
    previewMime = null;
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

  async function togglePreview(att: AttachmentInfo) {
    if (previewAttachment?.attachment_id === att.attachment_id) {
      previewAttachment = null;
      previewData = null;
      previewMime = null;
      return;
    }
    loadingPreview = true;
    try {
      const [data, _filename, mime] = await invoke<[string, string | null, string | null]>(
        'get_attachment_base64', { attachmentId: att.attachment_id }
      );
      previewData = data;
      previewMime = mime;
      previewAttachment = att;
    } catch (e) {
      savedMessage = `Failed to load preview: ${e}`;
      if (savedTimeout) clearTimeout(savedTimeout);
      savedTimeout = setTimeout(() => { savedMessage = null; }, 3000);
    } finally {
      loadingPreview = false;
    }
  }

  function isImage(mime: string | null): boolean {
    return !!mime && mime.startsWith('image/');
  }

  function isPdf(mime: string | null): boolean {
    return mime === 'application/pdf';
  }

  function isText(mime: string | null): boolean {
    return !!mime && (mime.startsWith('text/') || mime === 'application/json');
  }

  function decodeBase64Text(b64: string): string {
    return new TextDecoder().decode(Uint8Array.from(atob(b64), c => c.charCodeAt(0)));
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && previewAttachment) {
      previewAttachment = null;
      previewData = null;
      previewMime = null;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if attachments.length > 0}
  <div class="attachment-list">
    <div class="attachment-chips">
      {#each attachments as att (att.attachment_id)}
        <div class="attachment-chip-group">
          <button
            class="attachment-chip"
            class:active={previewAttachment?.attachment_id === att.attachment_id}
            on:click={() => togglePreview(att)}
            disabled={loadingPreview}
            title="Preview attachment"
          >
            <span class="clip-icon">&#x1F4CE;</span>
            <span class="att-name">{att.filename || 'attachment'}</span>
            {#if att.size}
              <span class="att-size">({formatSize(att.size)})</span>
            {/if}
          </button>
          <button
            class="download-btn"
            on:click|stopPropagation={() => downloadAttachment(att)}
            disabled={savingId === att.attachment_id}
            title="Save to Downloads"
          >&#x2B07;</button>
        </div>
      {/each}
    </div>
    {#if savedMessage}
      <div class="saved-msg">{savedMessage}</div>
    {/if}
    {#if previewAttachment && previewData}
      <div class="attachment-preview">
        <div class="preview-header">
          <span class="preview-name">{previewAttachment.filename || 'attachment'}</span>
          <div class="preview-actions">
            <button class="preview-action-btn" on:click={() => downloadAttachment(previewAttachment)}>Download</button>
            <button class="preview-action-btn" on:click={() => { previewAttachment = null; previewData = null; previewMime = null; }}>Close</button>
          </div>
        </div>
        <div class="preview-body">
          {#if isImage(previewMime)}
            <img src="data:{previewMime};base64,{previewData}" alt={previewAttachment.filename || 'attachment'} />
          {:else if isPdf(previewMime)}
            <iframe src="data:application/pdf;base64,{previewData}" title="PDF preview"></iframe>
          {:else if isText(previewMime)}
            <pre>{decodeBase64Text(previewData)}</pre>
          {:else}
            <div class="no-preview">
              <p>Preview not available for {previewMime || 'unknown type'}</p>
              <p class="no-preview-hint">Click "Download" to save the file.</p>
            </div>
          {/if}
        </div>
      </div>
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

  .attachment-chip-group {
    display: inline-flex;
    align-items: stretch;
    gap: 0;
  }

  .attachment-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 4px 0 0 4px;
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

  .attachment-chip.active {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--bg-secondary);
  }

  .attachment-chip:disabled {
    opacity: 0.5;
    cursor: wait;
  }

  .download-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 4px 6px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-left: none;
    border-radius: 0 4px 4px 0;
    color: var(--text-dim);
    font-size: 11px;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }

  .download-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .download-btn:disabled {
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

  .attachment-preview {
    border: 1px solid var(--border);
    border-radius: 4px;
    margin-top: 8px;
    background: var(--bg-secondary);
    overflow: hidden;
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }

  .preview-name {
    color: var(--text-primary);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .preview-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }

  .preview-action-btn {
    padding: 2px 8px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--text-secondary);
    font-size: 11px;
    font-family: inherit;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }

  .preview-action-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .preview-body {
    padding: 8px;
    max-height: 500px;
    overflow: auto;
  }

  .preview-body img {
    max-width: 100%;
    height: auto;
    border-radius: 4px;
  }

  .preview-body iframe {
    width: 100%;
    height: 500px;
    border: none;
  }

  .preview-body pre {
    white-space: pre-wrap;
    font-size: 12px;
    color: var(--text-primary);
    margin: 0;
  }

  .no-preview {
    text-align: center;
    padding: 20px;
    color: var(--text-dim);
  }

  .no-preview p {
    margin: 4px 0;
    font-size: 13px;
  }

  .no-preview-hint {
    font-size: 11px !important;
  }
</style>
