import { writable, get } from 'svelte/store';

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

const ACCOUNT_COLORS = [
  '#7c3aed', // purple (accent)
  '#3b82f6', // blue
  '#10b981', // emerald
  '#f59e0b', // amber
  '#ef4444', // red
  '#ec4899', // pink
  '#06b6d4', // cyan
  '#84cc16', // lime
  '#f97316', // orange
];

export function getAccountColor(index: number): string {
  return ACCOUNT_COLORS[index % ACCOUNT_COLORS.length];
}

export function getAccountColorById(accountId: string): string {
  const accts = get(accounts);
  const index = accts.findIndex((a) => a.id === accountId);
  return getAccountColor(index >= 0 ? index : 0);
}
