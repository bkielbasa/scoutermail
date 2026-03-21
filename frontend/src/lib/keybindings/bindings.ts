import type { Mode } from '$lib/stores/ui';

export interface BindingDef {
  keys: string;
  action: string;
  mode: Mode;
  description: string;
}

export const defaultBindings: BindingDef[] = [
  // NORMAL mode — navigation
  { keys: 'j', action: 'list-down', mode: 'NORMAL', description: 'Move down in list' },
  { keys: 'k', action: 'list-up', mode: 'NORMAL', description: 'Move up in list' },
  { keys: 'J', action: 'thread-next', mode: 'NORMAL', description: 'Next thread' },
  { keys: 'K', action: 'thread-prev', mode: 'NORMAL', description: 'Previous thread' },
  { keys: 'Enter', action: 'open-message', mode: 'NORMAL', description: 'Open message' },
  { keys: 'Tab', action: 'toggle-pane', mode: 'NORMAL', description: 'Toggle focus pane' },

  // NORMAL mode — compose actions
  { keys: 'r', action: 'reply', mode: 'NORMAL', description: 'Reply' },
  { keys: 'R', action: 'reply-all', mode: 'NORMAL', description: 'Reply all' },
  { keys: 'f', action: 'forward', mode: 'NORMAL', description: 'Forward' },
  { keys: 'c', action: 'compose', mode: 'NORMAL', description: 'Compose new message' },

  // NORMAL mode — message actions
  { keys: 'a', action: 'archive', mode: 'NORMAL', description: 'Archive' },
  { keys: 'd', action: 'delete', mode: 'NORMAL', description: 'Delete' },
  { keys: 's', action: 'star', mode: 'NORMAL', description: 'Star/unstar' },
  { keys: 'u', action: 'mark-unread', mode: 'NORMAL', description: 'Mark unread' },
  { keys: '!', action: 'mark-spam', mode: 'NORMAL', description: 'Mark as spam' },

  // NORMAL mode — view
  { keys: 'h', action: 'toggle-html', mode: 'NORMAL', description: 'Toggle HTML view' },
  { keys: 'H', action: 'show-headers', mode: 'NORMAL', description: 'Show headers' },
  { keys: 'v', action: 'enter-visual', mode: 'NORMAL', description: 'Enter visual mode' },
  { keys: '+', action: 'font-increase', mode: 'NORMAL', description: 'Increase font size' },
  { keys: '-', action: 'font-decrease', mode: 'NORMAL', description: 'Decrease font size' },
  { keys: '=', action: 'font-reset', mode: 'NORMAL', description: 'Reset font size' },

  // NORMAL mode — jump
  { keys: 'gg', action: 'list-top', mode: 'NORMAL', description: 'Jump to top of list' },
  { keys: 'G', action: 'list-bottom', mode: 'NORMAL', description: 'Jump to bottom of list' },

  // NORMAL mode — goto (multi-key)
  { keys: 'gi', action: 'goto-inbox', mode: 'NORMAL', description: 'Go to inbox' },
  { keys: 'gs', action: 'goto-sent', mode: 'NORMAL', description: 'Go to sent' },
  { keys: 'gd', action: 'goto-drafts', mode: 'NORMAL', description: 'Go to drafts' },
  { keys: 'ga', action: 'goto-archive', mode: 'NORMAL', description: 'Go to archive' },

  // NORMAL mode — account switching
  { keys: '1', action: 'switch-account-1', mode: 'NORMAL', description: 'Switch to account 1' },
  { keys: '2', action: 'switch-account-2', mode: 'NORMAL', description: 'Switch to account 2' },
  { keys: '3', action: 'switch-account-3', mode: 'NORMAL', description: 'Switch to account 3' },
  { keys: '4', action: 'switch-account-4', mode: 'NORMAL', description: 'Switch to account 4' },
  { keys: '5', action: 'switch-account-5', mode: 'NORMAL', description: 'Switch to account 5' },
  { keys: '6', action: 'switch-account-6', mode: 'NORMAL', description: 'Switch to account 6' },
  { keys: '7', action: 'switch-account-7', mode: 'NORMAL', description: 'Switch to account 7' },
  { keys: '8', action: 'switch-account-8', mode: 'NORMAL', description: 'Switch to account 8' },
  { keys: '9', action: 'switch-account-9', mode: 'NORMAL', description: 'Switch to account 9' },

  // VISUAL mode
  { keys: 'j', action: 'visual-extend-down', mode: 'VISUAL', description: 'Extend selection down' },
  { keys: 'k', action: 'visual-extend-up', mode: 'VISUAL', description: 'Extend selection up' },
  { keys: 'a', action: 'visual-archive', mode: 'VISUAL', description: 'Archive selected' },
  { keys: 'd', action: 'visual-delete', mode: 'VISUAL', description: 'Delete selected' },
  { keys: 'Escape', action: 'exit-visual', mode: 'VISUAL', description: 'Exit visual mode' },
];
