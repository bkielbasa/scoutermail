<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import AddressInput from './AddressInput.svelte';
  import { selectedMessage, type Message } from '$lib/stores/messages';
  import { mode } from '$lib/stores/ui';
  import { registerHandler } from '$lib/keybindings/engine';

  export let replyMode: 'compose' | 'reply' | 'reply-all' | 'forward' = 'compose';

  const dispatch = createEventDispatcher();

  let to = '';
  let cc = '';
  let bcc = '';
  let subject = '';
  let body = '';
  let showCc = false;
  let showBcc = false;
  let sending = false;
  let error = '';
  let inReplyTo: string | null = null;
  let references: string | null = null;
  let draftId: number | null = null;
  let saveTimer: ReturnType<typeof setInterval> | null = null;
  let draftSavedIndicator = false;

  let currentMessage: Message | null = null;
  const unsubMessage = selectedMessage.subscribe((msg) => {
    currentMessage = msg;
  });

  function stripSubjectPrefix(subj: string | null): string {
    if (!subj) return '';
    return subj.replace(/^(Re:\s*|Fwd:\s*)+/i, '').trim();
  }

  function buildQuotedBody(msg: Message): string {
    const date = msg.date ? new Date(msg.date).toLocaleString() : '';
    const from = msg.from_addr || '';
    const header = `\n\nOn ${date}, ${from} wrote:\n`;
    const quoted = (msg.body_text || '')
      .split('\n')
      .map((line) => `> ${line}`)
      .join('\n');
    return header + quoted;
  }

  function buildForwardBody(msg: Message): string {
    const lines = [
      '\n\n---------- Forwarded message ----------',
      `From: ${msg.from_addr || ''}`,
      `Date: ${msg.date || ''}`,
      `Subject: ${msg.subject || ''}`,
      `To: ${msg.to_addr || ''}`,
    ];
    if (msg.cc) {
      lines.push(`Cc: ${msg.cc}`);
    }
    lines.push('');
    lines.push(msg.body_text || '');
    return lines.join('\n');
  }

  function buildReferences(msg: Message): string | null {
    const parts: string[] = [];
    if (msg.ref_headers) {
      parts.push(msg.ref_headers);
    }
    if (msg.message_id) {
      parts.push(msg.message_id);
    }
    return parts.length > 0 ? parts.join(' ') : null;
  }

  async function saveDraft(): Promise<void> {
    if (!to && !subject && !body) return;
    try {
      const id = await invoke<number>('save_draft', {
        draft: {
          draft_id: draftId,
          to_addr: to,
          cc,
          bcc,
          subject,
          body,
          in_reply_to: inReplyTo,
          ref_headers: references,
          reply_mode: replyMode,
          updated_at: Math.floor(Date.now() / 1000),
        },
      });
      draftId = id;
      draftSavedIndicator = true;
      setTimeout(() => {
        draftSavedIndicator = false;
      }, 2000);
    } catch {
      // non-critical — draft save failure should not interrupt composing
    }
  }

  onMount(() => {
    mode.set('INSERT');

    if (currentMessage && replyMode !== 'compose') {
      const msg = currentMessage;
      const cleanSubject = stripSubjectPrefix(msg.subject);

      if (replyMode === 'reply' || replyMode === 'reply-all') {
        to = msg.from_addr || '';
        subject = `Re: ${cleanSubject}`;
        body = buildQuotedBody(msg);
        inReplyTo = msg.message_id || null;
        references = buildReferences(msg);

        if (replyMode === 'reply-all' && msg.cc) {
          cc = msg.cc;
          showCc = true;
        }
      } else if (replyMode === 'forward') {
        subject = `Fwd: ${cleanSubject}`;
        body = buildForwardBody(msg);
      }
    }

    registerHandler('send', handleSend);

    saveTimer = setInterval(saveDraft, 30000);
  });

  onDestroy(() => {
    unsubMessage();
    if (saveTimer) {
      clearInterval(saveTimer);
      saveTimer = null;
    }
  });

  function parseAddresses(input: string): string[] {
    return input
      .split(',')
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
  }

  async function handleSend(): Promise<void> {
    if (!to.trim()) {
      error = 'Recipient (To) is required.';
      return;
    }

    sending = true;
    error = '';

    try {
      await invoke('send_email', {
        req: {
          to: parseAddresses(to),
          cc: parseAddresses(cc),
          bcc: parseAddresses(bcc),
          subject,
          body_text: body,
          body_html: null,
          in_reply_to: inReplyTo,
          references: references ? references.split(/\s+/).filter(Boolean) : [],
        },
      });
      if (draftId) {
        try {
          await invoke('delete_draft', { draftId });
        } catch {
          // non-critical
        }
      }
      handleClose();
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      sending = false;
    }
  }

  async function handleClose(): Promise<void> {
    // Save draft one last time so user doesn't lose work
    await saveDraft();
    mode.set('NORMAL');
    dispatch('close');
  }
</script>

<div class="compose-overlay">
  <div class="compose-header">
    <h2 class="compose-title">
      {#if replyMode === 'reply'}Reply
      {:else if replyMode === 'reply-all'}Reply All
      {:else if replyMode === 'forward'}Forward
      {:else}Compose
      {/if}
    </h2>
    <button class="close-btn" on:click={handleClose} type="button">Esc</button>
  </div>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  <div class="compose-fields">
    <div class="field-row">
      <label class="field-label" for="compose-to">To</label>
      <AddressInput
        id="compose-to"
        bind:value={to}
        placeholder="recipient@example.com"
        disabled={sending}
      />
    </div>

    {#if showCc}
      <div class="field-row">
        <label class="field-label" for="compose-cc">Cc</label>
        <AddressInput
          id="compose-cc"
          bind:value={cc}
          placeholder="cc@example.com"
          disabled={sending}
        />
      </div>
    {/if}

    {#if showBcc}
      <div class="field-row">
        <label class="field-label" for="compose-bcc">Bcc</label>
        <AddressInput
          id="compose-bcc"
          bind:value={bcc}
          placeholder="bcc@example.com"
          disabled={sending}
        />
      </div>
    {/if}

    {#if !showCc || !showBcc}
      <div class="toggle-row">
        {#if !showCc}
          <button class="toggle-btn" on:click={() => (showCc = true)} type="button">+ Cc</button>
        {/if}
        {#if !showBcc}
          <button class="toggle-btn" on:click={() => (showBcc = true)} type="button">+ Bcc</button>
        {/if}
      </div>
    {/if}

    <div class="field-row">
      <label class="field-label" for="compose-subject">Subject</label>
      <input
        id="compose-subject"
        class="field-input"
        type="text"
        bind:value={subject}
        placeholder="Subject"
        disabled={sending}
      />
    </div>

    <div class="field-row body-row">
      <textarea
        class="body-input"
        bind:value={body}
        placeholder="Write your message..."
        disabled={sending}
      ></textarea>
    </div>
  </div>

  <div class="compose-footer">
    {#if draftSavedIndicator}
      <span class="draft-saved">(draft saved)</span>
    {/if}
    <span class="send-hint">Ctrl+Enter to send</span>
    <button
      class="send-btn"
      on:click={handleSend}
      disabled={sending}
      type="button"
    >
      {#if sending}Sending...{:else}Send{/if}
    </button>
  </div>
</div>

<style>
  .compose-overlay {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 16px 20px;
    overflow-y: auto;
  }

  .compose-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }

  .compose-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .close-btn {
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 4px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-family: monospace;
    font-size: 12px;
  }

  .close-btn:hover {
    color: var(--text-primary);
    border-color: var(--text-dim);
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #ef4444;
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 13px;
    margin-bottom: 12px;
  }

  .compose-fields {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
  }

  .field-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .field-label {
    font-family: monospace;
    font-size: 13px;
    color: var(--text-dim);
    width: 60px;
    flex-shrink: 0;
    text-align: right;
  }

  .field-input {
    flex: 1;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 8px 10px;
    border-radius: 4px;
    font-family: inherit;
    font-size: 13px;
    outline: none;
  }

  .field-input:focus {
    border-color: var(--accent);
  }

  .field-input:disabled {
    opacity: 0.5;
  }

  .toggle-row {
    display: flex;
    gap: 8px;
    padding-left: 68px;
  }

  .toggle-btn {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-size: 12px;
    padding: 2px 4px;
    font-family: monospace;
  }

  .toggle-btn:hover {
    text-decoration: underline;
  }

  .body-row {
    flex: 1;
    align-items: stretch;
  }

  .body-input {
    flex: 1;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 10px 12px;
    border-radius: 4px;
    font-family: inherit;
    font-size: 13px;
    line-height: 1.6;
    resize: none;
    outline: none;
    min-height: 200px;
  }

  .body-input:focus {
    border-color: var(--accent);
  }

  .body-input:disabled {
    opacity: 0.5;
  }

  .compose-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }

  .draft-saved {
    font-size: 11px;
    color: var(--text-dim);
    font-family: monospace;
    opacity: 0.7;
  }

  .send-hint {
    font-size: 11px;
    color: var(--text-dim);
    font-family: monospace;
  }

  .send-btn {
    background: var(--accent);
    border: none;
    color: white;
    padding: 8px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-family: inherit;
    font-size: 13px;
    font-weight: 500;
  }

  .send-btn:hover:not(:disabled) {
    filter: brightness(1.15);
  }

  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
