import { writable } from 'svelte/store';

export type Mode = 'NORMAL' | 'INSERT' | 'VISUAL' | 'COMMAND';
export type FocusPane = 'list' | 'reading';

export const mode = writable<Mode>('NORMAL');
export const focusPane = writable<FocusPane>('list');
export const commandInput = writable('');
export const searchOpen = writable(false);
export const searchQuery = writable('');
export const helpOpen = writable(false);
export const unifiedMode = writable(false);
export const loading = writable(false);
export const readingFontSize = writable(13);
