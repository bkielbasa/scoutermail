import { get } from 'svelte/store';
import { mode, commandInput, searchOpen, helpOpen, commandSuggestions } from '$lib/stores/ui';
import type { Mode } from '$lib/stores/ui';

export type Action = string;
export type KeyHandler = (args?: string) => void;

export interface Binding {
  keys: string;
  action: string;
  mode: Mode;
  description: string;
}

const KNOWN_COMMANDS = [
  'account', 'backup', 'move', 'label', 'unlabel', 'labeled', 'filter',
  'contacts', 'calendar', 'folders', 'drafts', 'rules',
  'signature', 'spam', 'print', 'snooze', 'unified',
  'template', 'set',
];

const handlers = new Map<Action, KeyHandler>();
let bindings: Binding[] = [];
let keyBuffer = '';
let bufferTimeout: ReturnType<typeof setTimeout> | null = null;

export function registerHandler(action: Action, handler: KeyHandler): void {
  handlers.set(action, handler);
}

export function setBindings(newBindings: Binding[]): void {
  bindings = newBindings;
}

function clearBuffer(): void {
  keyBuffer = '';
  if (bufferTimeout !== null) {
    clearTimeout(bufferTimeout);
    bufferTimeout = null;
  }
}

function resetBufferTimeout(): void {
  if (bufferTimeout !== null) {
    clearTimeout(bufferTimeout);
  }
  bufferTimeout = setTimeout(() => {
    keyBuffer = '';
    bufferTimeout = null;
  }, 500);
}

function execute(action: Action, args?: string): void {
  const handler = handlers.get(action);
  if (handler) {
    handler(args);
  }
}

export function executeCommand(cmd: string): void {
  const parts = cmd.trim().split(/\s+/);
  if (parts.length === 0 || parts[0] === '') return;
  const action = `cmd:${parts[0]}`;
  const args = parts.slice(1).join(' ');
  execute(action, args || undefined);
}

function keyName(event: KeyboardEvent): string {
  if (event.ctrlKey && event.key === 'Enter') return 'Ctrl+Enter';
  if (event.key === 'Escape') return 'Escape';
  if (event.key === 'Enter') return 'Enter';
  if (event.key === 'Tab') return 'Tab';
  if (event.key === 'Backspace') return 'Backspace';
  // For single character keys, use the key value directly
  if (event.key.length === 1) return event.key;
  return event.key;
}

function updateCommandSuggestions(): void {
  const input = get(commandInput);
  // Only show suggestions for the command name portion (before first space)
  if (input && !input.includes(' ')) {
    const matches = KNOWN_COMMANDS.filter((c) => c.startsWith(input));
    commandSuggestions.set(matches);
  } else {
    commandSuggestions.set([]);
  }
}

function handleCommandMode(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.preventDefault();
    commandInput.set('');
    commandSuggestions.set([]);
    mode.set('NORMAL');
    return;
  }

  if (event.key === 'Enter') {
    event.preventDefault();
    const cmd = get(commandInput);
    commandInput.set('');
    commandSuggestions.set([]);
    mode.set('NORMAL');
    executeCommand(cmd);
    return;
  }

  if (event.key === 'Tab') {
    event.preventDefault();
    const input = get(commandInput);
    if (!input || input.includes(' ')) return;

    const matches = KNOWN_COMMANDS.filter((c) => c.startsWith(input));
    if (matches.length === 1) {
      commandInput.set(matches[0] + ' ');
      commandSuggestions.set([]);
    } else if (matches.length > 1) {
      // Find longest common prefix
      let prefix = matches[0];
      for (const m of matches) {
        while (!m.startsWith(prefix)) {
          prefix = prefix.slice(0, -1);
        }
      }
      commandInput.set(prefix);
      commandSuggestions.set(matches);
    }
    return;
  }

  if (event.key === 'Backspace') {
    event.preventDefault();
    const current = get(commandInput);
    if (current.length === 0) {
      commandSuggestions.set([]);
      mode.set('NORMAL');
    } else {
      commandInput.set(current.slice(0, -1));
      updateCommandSuggestions();
    }
    return;
  }

  // Capture printable characters into commandInput
  if (event.key.length === 1 && !event.ctrlKey && !event.metaKey) {
    event.preventDefault();
    commandInput.update((v) => v + event.key);
    updateCommandSuggestions();
  }
}

function handleInsertMode(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.preventDefault();
    mode.set('NORMAL');
    return;
  }

  if (event.ctrlKey && event.key === 'Enter') {
    event.preventDefault();
    execute('send');
    return;
  }

  // Everything else passes through
}

function handleNormalOrVisualMode(event: KeyboardEvent): void {
  const currentMode = get(mode);

  // Don't intercept modifier-only combos (except specific ones) or meta key shortcuts
  if (event.metaKey || event.altKey) return;
  if (event.ctrlKey && event.key !== 'Enter') return;

  // Let input/textarea/select elements handle their own keystrokes
  const tag = (event.target as HTMLElement)?.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

  event.preventDefault();

  const key = keyName(event);

  // Special keys in NORMAL mode
  if (currentMode === 'NORMAL') {
    if (key === ':') {
      commandInput.set('');
      mode.set('COMMAND');
      return;
    }

    if (key === '/') {
      searchOpen.set(true);
      return;
    }

    if (key === '?') {
      helpOpen.update((v) => !v);
      return;
    }
  }

  // Build key buffer and match against bindings
  keyBuffer += key;
  resetBufferTimeout();

  // Find exact and partial matches for current mode
  const modeBindings = bindings.filter((b) => b.mode === currentMode);
  const exactMatch = modeBindings.find((b) => b.keys === keyBuffer);
  const partialMatch = modeBindings.some(
    (b) => b.keys.startsWith(keyBuffer) && b.keys !== keyBuffer
  );

  if (exactMatch) {
    clearBuffer();
    execute(exactMatch.action);
  } else if (!partialMatch) {
    clearBuffer();
  }
  // If partial match, keep waiting for more keys
}

export function handleKeyDown(event: KeyboardEvent): void {
  const currentMode = get(mode);

  switch (currentMode) {
    case 'COMMAND':
      handleCommandMode(event);
      break;
    case 'INSERT':
      handleInsertMode(event);
      break;
    case 'NORMAL':
    case 'VISUAL':
      handleNormalOrVisualMode(event);
      break;
  }
}
