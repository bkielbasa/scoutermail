import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';
import { unreadCount, folderCounts } from '$lib/stores/accounts';

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
  // Unified inbox fields (only present when in unified mode)
  account_id?: string;
  account_name?: string;
}

export const messages = writable<Message[]>([]);
export const selectedIndex = writable<number>(0);
export const visualSelection = writable<Set<number>>(new Set());
export const selectedMessage = derived(
  [messages, selectedIndex],
  ([$messages, $selectedIndex]) => $messages[$selectedIndex] ?? null
);
export const threadMessages = writable<Message[]>([]);

export async function loadMessages(folder: string, resetSelection = true): Promise<void> {
  const result = await invoke<Message[]>('get_messages', { folder });
  messages.set(result);
  if (resetSelection) {
    selectedIndex.set(0);
  } else {
    // Clamp index to valid range
    selectedIndex.update((i) => Math.min(i, Math.max(result.length - 1, 0)));
  }

  // Compute unread count for current folder
  const unread = result.filter((m) => !m.flags?.includes('Seen')).length;
  unreadCount.set(unread);
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
  const newMessages = await invoke<Message[]>('sync_folder', { folder });
  await loadMessages(folder);

  if (newMessages.length > 0) {
    notifyNewMail(newMessages);
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
