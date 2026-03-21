import { writable } from 'svelte/store';

export interface Account {
  id: string;
  name: string;
  email: string;
  imap_host: string;
  imap_port: number;
  smtp_host: string;
  smtp_port: number;
  username: string;
}

export const accounts = writable<Account[]>([]);
export const activeAccount = writable<Account | null>(null);
export const activeFolder = writable<string>('INBOX');
export const unreadCount = writable<number>(0);
export const folderCounts = writable<Array<[string, number, number]>>([]);
