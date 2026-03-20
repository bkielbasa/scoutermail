import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

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
}

export const messages = writable<Message[]>([]);
export const selectedIndex = writable<number>(0);
export const selectedMessage = derived(
  [messages, selectedIndex],
  ([$messages, $selectedIndex]) => $messages[$selectedIndex] ?? null
);
export const threadMessages = writable<Message[]>([]);

export async function loadMessages(folder: string): Promise<void> {
  const result = await invoke<Message[]>('get_messages', { folder });
  messages.set(result);
  selectedIndex.set(0);
}

export async function loadThreadMessages(threadId: string): Promise<void> {
  const result = await invoke<Message[]>('get_thread_messages', { threadId });
  threadMessages.set(result);
}

export async function syncFolder(folder: string): Promise<void> {
  await invoke('sync_folder', { folder });
  await loadMessages(folder);
}
