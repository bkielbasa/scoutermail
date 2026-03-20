<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    selectedMessage,
    threadMessages,
    type Message,
  } from '$lib/stores/messages';
  import { focusPane } from '$lib/stores/ui';
  import { registerHandler } from '$lib/keybindings/engine';

  let currentMessage: Message | null = null;
  let thread: Message[] = [];
  let currentFocus: string = 'list';
  let showHtml = true;
  let showHeaders = false;
  let expandedUids: Set<number> = new Set();

  function extractName(addr: string | null): string {
    if (!addr) return '(unknown)';
    const match = addr.match(/^(.+?)\s*<.+>$/);
    if (match) return match[1].replace(/^["']|["']$/g, '').trim();
    const atMatch = addr.match(/^([^@]+)@/);
    if (atMatch) return atMatch[1];
    return addr;
  }

  function formatDate(dateStr: string | null): string {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    if (isNaN(date.getTime())) return dateStr;
    return date.toLocaleString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    });
  }

  function formatShortDate(dateStr: string | null): string {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    if (isNaN(date.getTime())) return dateStr;
    return date.toLocaleString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    });
  }

  function formatRecipients(to: string | null): string {
    if (!to) return '';
    return to
      .split(',')
      .map((r) => extractName(r.trim()))
      .join(', ');
  }

  function buildRawHeaders(msg: Message): string {
    const lines: string[] = [];
    if (msg.from_addr) lines.push(`From: ${msg.from_addr}`);
    if (msg.to_addr) lines.push(`To: ${msg.to_addr}`);
    if (msg.cc) lines.push(`Cc: ${msg.cc}`);
    if (msg.subject) lines.push(`Subject: ${msg.subject}`);
    if (msg.date) lines.push(`Date: ${msg.date}`);
    if (msg.message_id) lines.push(`Message-ID: ${msg.message_id}`);
    if (msg.in_reply_to) lines.push(`In-Reply-To: ${msg.in_reply_to}`);
    if (msg.ref_headers) lines.push(`References: ${msg.ref_headers}`);
    if (msg.flags) lines.push(`Flags: ${msg.flags}`);
    if (msg.thread_id) lines.push(`Thread-ID: ${msg.thread_id}`);
    return lines.join('\n');
  }

  function toggleThread(uid: number): void {
    if (expandedUids.has(uid)) {
      expandedUids.delete(uid);
    } else {
      expandedUids.add(uid);
    }
    expandedUids = expandedUids; // trigger reactivity
  }

  function isExpanded(uid: number): boolean {
    return expandedUids.has(uid);
  }

  const unsubMessage = selectedMessage.subscribe((msg) => {
    currentMessage = msg;
    showHeaders = false;
    // Auto-expand the selected message in thread
    if (msg) {
      expandedUids = new Set([msg.uid]);
    } else {
      expandedUids = new Set();
    }
  });

  const unsubThread = threadMessages.subscribe((msgs) => {
    thread = msgs;
    // If we have a current message, ensure it's expanded
    if (currentMessage) {
      expandedUids = new Set([currentMessage.uid]);
    }
  });

  const unsubFocus = focusPane.subscribe((v) => (currentFocus = v));

  onMount(() => {
    registerHandler('toggle-html', () => {
      showHtml = !showHtml;
    });

    registerHandler('show-headers', () => {
      showHeaders = !showHeaders;
    });
  });

  onDestroy(() => {
    unsubMessage();
    unsubThread();
    unsubFocus();
  });
</script>

<div class="reading-pane-inner" class:focused={currentFocus === 'reading'}>
  {#if !currentMessage}
    <div class="empty-state">
      <p class="empty-title">No message selected</p>
      <p class="empty-hint">j/k to navigate, enter to open</p>
    </div>
  {:else}
    <div class="message-view">
      <h2 class="subject">{currentMessage.subject || '(no subject)'}</h2>
      <div class="meta-line">
        <span class="sender-name">{extractName(currentMessage.from_addr)}</span>
        <span class="arrow">&rarr;</span>
        <span class="recipients">{formatRecipients(currentMessage.to_addr)}</span>
        <span class="meta-sep">&middot;</span>
        <span class="msg-date">{formatDate(currentMessage.date)}</span>
      </div>

      {#if showHeaders}
        <pre class="raw-headers">{buildRawHeaders(currentMessage)}</pre>
      {/if}

      {#if thread.length > 1}
        <div class="thread-view">
          {#each thread as msg (msg.uid)}
            <div
              class="thread-message"
              class:current={msg.uid === currentMessage.uid}
            >
              <button
                class="thread-header"
                type="button"
                on:click={() => toggleThread(msg.uid)}
              >
                <span class="toggle-arrow">{isExpanded(msg.uid) ? '\u25BC' : '\u25B6'}</span>
                <span class="thread-sender">{extractName(msg.from_addr)}</span>
                <span class="thread-date">{formatShortDate(msg.date)}</span>
              </button>
              {#if isExpanded(msg.uid)}
                <div class="thread-body">
                  {#if showHtml && msg.body_html}
                    <iframe
                      srcdoc={msg.body_html}
                      sandbox="allow-same-origin"
                      title="Email content"
                      class="html-frame"
                      on:load={(e) => {
                        const iframe = e.currentTarget;
                        const doc = iframe.contentDocument;
                        if (doc) {
                          iframe.style.height = doc.documentElement.scrollHeight + 'px';
                        }
                      }}
                    ></iframe>
                  {:else}
                    <pre class="body-text">{msg.body_text || '(no content)'}</pre>
                  {/if}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {:else}
        <div class="single-body">
          {#if showHtml && currentMessage.body_html}
            <iframe
              srcdoc={currentMessage.body_html}
              sandbox="allow-same-origin"
              title="Email content"
              class="html-frame"
              on:load={(e) => {
                const iframe = e.currentTarget;
                const doc = iframe.contentDocument;
                if (doc) {
                  iframe.style.height = doc.documentElement.scrollHeight + 'px';
                }
              }}
            ></iframe>
          {:else}
            <pre class="body-text">{currentMessage.body_text || '(no content)'}</pre>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .reading-pane-inner {
    height: 100%;
    overflow-y: auto;
    padding: 16px 20px;
  }

  /* Empty state */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 8px;
  }

  .empty-title {
    color: var(--text-secondary);
    font-size: 15px;
  }

  .empty-hint {
    color: var(--text-dim);
    font-size: 12px;
  }

  /* Message view */
  .message-view {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .subject {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 8px 0;
    line-height: 1.3;
  }

  .meta-line {
    display: flex;
    align-items: baseline;
    gap: 6px;
    flex-wrap: wrap;
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }

  .sender-name {
    color: var(--accent);
    font-weight: 500;
  }

  .arrow {
    color: var(--text-dim);
  }

  .recipients {
    color: var(--text-secondary);
  }

  .meta-sep {
    color: var(--text-dim);
  }

  .msg-date {
    color: var(--text-dim);
    font-size: 12px;
  }

  /* Raw headers */
  .raw-headers {
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 10px 12px;
    font-size: 11px;
    color: var(--text-secondary);
    white-space: pre-wrap;
    word-break: break-all;
    margin-bottom: 16px;
    line-height: 1.5;
  }

  /* Thread view */
  .thread-view {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-top: 4px;
  }

  .thread-message {
    border-left: 2px solid transparent;
    border-radius: 2px;
  }

  .thread-message.current {
    border-left-color: var(--accent);
  }

  .thread-header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 10px;
    border: none;
    background: var(--bg-secondary);
    color: var(--text-secondary);
    cursor: pointer;
    font-family: inherit;
    font-size: 13px;
    text-align: left;
  }

  .thread-header:hover {
    background: var(--bg-tertiary);
  }

  .toggle-arrow {
    color: var(--text-dim);
    font-size: 10px;
    width: 12px;
    flex-shrink: 0;
  }

  .thread-sender {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
  }

  .thread-message.current .thread-sender {
    color: var(--accent);
  }

  .thread-date {
    color: var(--text-dim);
    font-size: 11px;
    flex-shrink: 0;
  }

  .thread-body {
    padding: 12px 10px 12px 30px;
  }

  /* Body content */
  .single-body {
    margin-top: 4px;
  }

  .body-text {
    white-space: pre-wrap;
    word-break: break-word;
    color: var(--text-primary);
    font-size: 13px;
    line-height: 1.6;
    margin: 0;
    font-family: inherit;
  }

  .html-frame {
    width: 100%;
    min-height: 100px;
    border: none;
    background: white;
    border-radius: 4px;
  }
</style>
