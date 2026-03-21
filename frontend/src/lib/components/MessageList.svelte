<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';
  import {
    messages,
    selectedIndex,
    selectedMessage,
    loadThreadMessages,
    visualSelection,
  } from '$lib/stores/messages';
  import { focusPane, mode } from '$lib/stores/ui';
  import { registerHandler } from '$lib/keybindings/engine';

  function formatDate(dateStr: string | null): string {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    if (isNaN(date.getTime())) return dateStr;
    const now = new Date();
    const isToday =
      date.getFullYear() === now.getFullYear() &&
      date.getMonth() === now.getMonth() &&
      date.getDate() === now.getDate();
    if (isToday) {
      return date.toLocaleTimeString(undefined, {
        hour: '2-digit',
        minute: '2-digit',
        hour12: false,
      });
    }
    const months = [
      'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
      'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec',
    ];
    return `${months[date.getMonth()]} ${date.getDate()}`;
  }

  function extractName(from: string | null): string {
    if (!from) return '(unknown)';
    // Match "Name <email>" pattern
    const match = from.match(/^(.+?)\s*<.+>$/);
    if (match) {
      return match[1].replace(/^["']|["']$/g, '').trim();
    }
    // Fall back to username before @
    const atMatch = from.match(/^([^@]+)@/);
    if (atMatch) {
      return atMatch[1];
    }
    return from;
  }

  function isUnread(flags: string | null): boolean {
    if (!flags) return true;
    return !flags.includes('Seen');
  }

  function selectAndLoadThread(index: number): void {
    selectedIndex.set(index);
    const msg = get(messages)[index];
    if (msg?.thread_id) {
      loadThreadMessages(msg.thread_id);
    }
  }

  onMount(() => {
    registerHandler('list-down', () => {
      const msgs = get(messages);
      selectedIndex.update((i) => Math.min(i + 1, msgs.length - 1));
    });

    registerHandler('list-up', () => {
      selectedIndex.update((i) => Math.max(i - 1, 0));
    });

    registerHandler('list-top', () => {
      selectedIndex.set(0);
    });

    registerHandler('list-bottom', () => {
      const msgs = get(messages);
      selectedIndex.set(Math.max(msgs.length - 1, 0));
    });

    registerHandler('open-message', async () => {
      const msg = get(selectedMessage);
      if (!msg) return;
      if (msg.thread_id) {
        loadThreadMessages(msg.thread_id);
      }
      // Mark as read if unread
      if (!msg.flags?.includes('Seen')) {
        const newFlags = msg.flags ? `${msg.flags} Seen` : 'Seen';
        try {
          await invoke('update_flags', { uid: msg.uid, folder: msg.folder, flags: newFlags });
          // Update the local store immediately
          messages.update((msgs) =>
            msgs.map((m) =>
              m.uid === msg.uid && m.folder === msg.folder ? { ...m, flags: newFlags } : m
            )
          );
        } catch {
          // Non-critical
        }
      }
      focusPane.set('reading');
    });

    registerHandler('toggle-pane', () => {
      focusPane.update((p) => (p === 'list' ? 'reading' : 'list'));
    });

    registerHandler('enter-visual', () => {
      mode.set('VISUAL');
      const idx = get(selectedIndex);
      visualSelection.set(new Set([idx]));
    });

    registerHandler('exit-visual', () => {
      mode.set('NORMAL');
      visualSelection.set(new Set());
    });

    registerHandler('visual-extend-down', () => {
      selectedIndex.update((i) => {
        const msgs = get(messages);
        const next = Math.min(i + 1, msgs.length - 1);
        visualSelection.update((s) => { s.add(next); return new Set(s); });
        return next;
      });
    });

    registerHandler('visual-extend-up', () => {
      selectedIndex.update((i) => {
        const prev = Math.max(i - 1, 0);
        visualSelection.update((s) => { s.add(prev); return new Set(s); });
        return prev;
      });
    });
  });

  let messageList: Message[] = [];
  let currentIndex = 0;
  let currentFocus: string = 'list';

  const unsubMessages = messages.subscribe((v) => (messageList = v));
  const unsubIndex = selectedIndex.subscribe((v) => {
    currentIndex = v;
    // Scroll selected item into view
    setTimeout(() => {
      const el = document.querySelector('.message-item.selected');
      el?.scrollIntoView({ block: 'nearest' });
    }, 0);
  });
  const unsubFocus = focusPane.subscribe((v) => (currentFocus = v));

  let currentVisualSelection: Set<number> = new Set();
  const unsubVisual = visualSelection.subscribe((v) => (currentVisualSelection = v));

  onDestroy(() => {
    unsubMessages();
    unsubIndex();
    unsubFocus();
    unsubVisual();
  });
</script>

<div class="message-list" class:focused={currentFocus === 'list'}>
  {#if messageList.length === 0}
    <p class="empty">No messages</p>
  {:else}
    {#each messageList as msg, i}
      <button
        class="message-item"
        class:selected={i === currentIndex}
        class:unread={isUnread(msg.flags)}
        class:visual-selected={currentVisualSelection.has(i)}
        on:click={() => selectAndLoadThread(i)}
        type="button"
      >
        <div class="msg-header">
          <span class="sender">{extractName(msg.from_addr)}</span>
          <span class="date">{formatDate(msg.date)}</span>
        </div>
        <div class="subject">{msg.subject || '(no subject)'}</div>
      </button>
    {/each}
  {/if}
</div>

<style>
  .message-list {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
  }

  .empty {
    color: var(--text-dim);
    padding: 12px;
  }

  .message-item {
    display: block;
    width: 100%;
    padding: 8px 12px;
    border: none;
    border-left: 2px solid transparent;
    background: none;
    text-align: left;
    cursor: pointer;
    color: var(--text-secondary);
    font-family: inherit;
    font-size: inherit;
    line-height: 1.4;
  }

  .message-item:hover {
    background: var(--bg-tertiary);
  }

  .message-item.selected {
    border-left-color: var(--accent);
    background: var(--accent-dim);
  }

  .message-item.visual-selected {
    background: var(--accent-dim);
    border-left-color: var(--accent);
  }

  .message-item.unread .sender,
  .message-item.unread .subject {
    color: var(--text-primary);
    font-weight: 600;
  }

  .msg-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
  }

  .sender {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .date {
    color: var(--text-dim);
    font-size: 11px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .subject {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 12px;
    margin-top: 2px;
  }
</style>
