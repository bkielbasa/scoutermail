import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';
import { unreadCount, folderCounts } from '$lib/stores/accounts';
import { loading } from '$lib/stores/ui';

export interface Message {
  uid: number;
  message_id: string | null;
  folder: string;
  subject: string | null;
  from_addr: string | null;
  to_addr: string | null;
  cc: string | null;
  date: string | null;
  body_text: string | null;
  body_html: string | null;
  flags: string | null;
  thread_id: string | null;
  ref_headers: string | null;
  in_reply_to: string | null;
  reply_to: string | null;
  list_unsubscribe: string | null;
  // Unified inbox fields (only present when in unified mode)
  account_id?: string;
  account_name?: string;
}

export const messages = writable<Message[]>([]);
export const selectedIndex = writable<number>(0);
export const visualSelection = writable<Set<number>>(new Set());

export const PAGE_SIZE = 50;
export const page = writable<number>(0);
export const totalMessages = writable<number>(0);

export type FilterType = 'all' | 'unread' | 'starred';
export const activeFilter = writable<FilterType>('all');

export const filteredMessages = derived(
  [messages, activeFilter],
  ([$messages, $filter]) => {
    switch ($filter) {
      case 'unread':
        return $messages.filter((m) => !m.flags?.includes('Seen'));
      case 'starred':
        return $messages.filter((m) => m.flags?.includes('Flagged'));
      default:
        return $messages;
    }
  }
);

export const selectedMessage = derived(
  [filteredMessages, selectedIndex],
  ([$filtered, $idx]) => $filtered[$idx] ?? null
);
export const threadMessages = writable<Message[]>([]);

export async function loadMessages(folder: string, resetSelection = true): Promise<void> {
  loading.set(true);
  try {
    let currentPage = 0;
    page.subscribe((v) => (currentPage = v))();

    const [result, count] = await Promise.all([
      invoke<Message[]>('get_messages_headers', {
        folder,
        limit: PAGE_SIZE,
        offset: currentPage * PAGE_SIZE,
      }),
      invoke<number>('get_message_count', { folder }),
    ]);

    // Preserve already-loaded bodies when refreshing
    const existing = get(messages);
    const bodyCache = new Map<string, { body_text: string | null; body_html: string | null }>();
    for (const m of existing) {
      if (m.body_text !== null || m.body_html !== null) {
        bodyCache.set(`${m.uid}:${m.folder}`, { body_text: m.body_text, body_html: m.body_html });
      }
    }
    const merged = result.map((m) => {
      const cached = bodyCache.get(`${m.uid}:${m.folder}`);
      if (cached) {
        return { ...m, body_text: cached.body_text, body_html: cached.body_html };
      }
      return m;
    });

    messages.set(merged);
    totalMessages.set(count);

    if (resetSelection) {
      selectedIndex.set(0);
    } else {
      // Clamp index to valid range
      selectedIndex.update((i) => Math.min(i, Math.max(result.length - 1, 0)));
    }

    // Compute unread count for current folder
    const unread = result.filter((m) => !m.flags?.includes('Seen')).length;
    unreadCount.set(unread);
  } finally {
    loading.set(false);
  }
}

export async function loadNextPage(folder: string): Promise<void> {
  let currentPage = 0;
  let total = 0;
  page.subscribe((v) => (currentPage = v))();
  totalMessages.subscribe((v) => (total = v))();

  if ((currentPage + 1) * PAGE_SIZE < total) {
    page.set(currentPage + 1);
    await loadMessages(folder, true);
  }
}

export async function loadPrevPage(folder: string): Promise<void> {
  let currentPage = 0;
  page.subscribe((v) => (currentPage = v))();

  if (currentPage > 0) {
    page.set(currentPage - 1);
    await loadMessages(folder, true);
  }
}

export async function loadFullMessage(uid: number, folder: string): Promise<Message | null> {
  try {
    return await invoke<Message>('get_message', { uid, folder });
  } catch {
    return null;
  }
}

export async function loadUnifiedMessages(folder: string): Promise<void> {
  const result = await invoke<Message[]>('get_unified_messages', { folder });
  messages.set(result);
  selectedIndex.set(0);

  const unread = result.filter((m) => !m.flags?.includes('Seen')).length;
  unreadCount.set(unread);
}

export async function refreshFolderCounts(): Promise<void> {
  try {
    const counts = await invoke<Array<[string, number, number]>>('get_folder_counts');
    folderCounts.set(counts);
  } catch {
    // Non-critical
  }
}

export async function loadThreadMessages(threadId: string): Promise<void> {
  const result = await invoke<Message[]>('get_thread_messages', { threadId });
  threadMessages.set(result);
}

export async function syncFolder(folder: string): Promise<void> {
  loading.set(true);
  try {
    const newMessages = await invoke<Message[]>('sync_folder', { folder });
    await loadMessages(folder);

    if (newMessages.length > 0) {
      notifyNewMail(newMessages);
    }
  } finally {
    loading.set(false);
  }
}

async function notifyNewMail(newMessages: Message[]): Promise<void> {
  try {
    let permitted = await isPermissionGranted();
    if (!permitted) {
      const result = await requestPermission();
      permitted = result === 'granted';
    }
    if (!permitted) return;

    if (newMessages.length === 1) {
      const msg = newMessages[0];
      const from = msg.from_addr?.replace(/<.*>/, '').trim() || 'Unknown';
      sendNotification({
        title: from,
        body: msg.subject || '(no subject)',
      });
    } else {
      sendNotification({
        title: 'ScouterMail',
        body: `${newMessages.length} new messages`,
      });
    }
  } catch {
    // Notification errors are non-critical
  }
}
