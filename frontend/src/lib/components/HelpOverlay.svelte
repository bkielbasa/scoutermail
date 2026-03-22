<script lang="ts">
  import { helpOpen, unifiedMode } from '$lib/stores/ui';
  import { activeAccount, activeFolder } from '$lib/stores/accounts';
  import { activeFilter, totalMessages } from '$lib/stores/messages';
  import { defaultBindings } from '$lib/keybindings/bindings';

  const normalBindings = defaultBindings.filter((b) => b.mode === 'NORMAL');
  const visualBindings = defaultBindings.filter((b) => b.mode === 'VISUAL');

  const insertBindings = [
    { keys: 'Esc', description: 'Exit insert mode' },
    { keys: 'Ctrl+Enter', description: 'Send message' },
  ];

  const specialKeys = [
    { keys: '/', description: 'Open search' },
    { keys: ':', description: 'Command mode' },
    { keys: '?', description: 'Toggle help' },
  ];

  const commands = [
    { cmd: ':move <folder>', description: 'Move message to folder' },
    { cmd: ':label <name>', description: 'Add label to message' },
    { cmd: ':unlabel <name>', description: 'Remove label' },
    { cmd: ':labeled <name>', description: 'Show messages with label' },
    { cmd: ':filter <type>', description: 'Filter: unread, starred, all' },
    { cmd: ':contacts', description: 'Open contacts list' },
    { cmd: ':calendar', description: 'Open calendar view' },
    { cmd: ':folders', description: 'Toggle folder sidebar' },
    { cmd: ':drafts', description: 'Show saved drafts' },
    { cmd: ':signature', description: 'Edit email signature' },
    { cmd: ':template <cmd>', description: 'Templates: save/list/delete/<name>' },
    { cmd: ':rules', description: 'Open rules editor' },
    { cmd: ':unified', description: 'Unified inbox (all accounts)' },
    { cmd: ':spam', description: 'Mark as spam' },
    { cmd: ':print', description: 'Print current email' },
    { cmd: ':snooze <min>', description: 'Snooze message for N minutes' },
    { cmd: ':backup', description: 'Backup database' },
    { cmd: ':set <key> <val>', description: 'Change setting' },
  ];

  let activeTab: 'keys' | 'commands' = 'keys';

  let account: { name: string; email: string } | null = null;
  let folder = 'INBOX';
  let filter = 'all';
  let total = 0;
  let isUnified = false;

  activeAccount.subscribe((v) => (account = v));
  activeFolder.subscribe((v) => (folder = v));
  activeFilter.subscribe((v) => (filter = v));
  totalMessages.subscribe((v) => (total = v));
  unifiedMode.subscribe((v) => (isUnified = v));

  function close(): void {
    helpOpen.set(false);
  }

  function handleBackdropClick(event: MouseEvent): void {
    if (event.target === event.currentTarget) {
      close();
    }
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === '?' || event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      close();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y_interactive_supports_focus a11y_click_events_have_key_events -->
<div class="help-backdrop" on:click={handleBackdropClick} role="dialog" aria-modal="true" aria-label="Keyboard shortcuts">
  <div class="help-panel">
    <h2 class="help-title">ScouterMail Help</h2>

    <div class="state-bar">
      <span class="state-item">Account: <strong>{isUnified ? 'All (unified)' : (account ? account.name : 'none')}</strong></span>
      <span class="state-sep">|</span>
      <span class="state-item">Folder: <strong>{folder}</strong></span>
      <span class="state-sep">|</span>
      <span class="state-item">Filter: <strong>{filter}</strong></span>
      <span class="state-sep">|</span>
      <span class="state-item">Messages: <strong>{total}</strong></span>
    </div>

    <div class="tab-bar">
      <button class="tab" class:active={activeTab === 'keys'} on:click={() => (activeTab = 'keys')} type="button">Keybindings</button>
      <button class="tab" class:active={activeTab === 'commands'} on:click={() => (activeTab = 'commands')} type="button">Commands</button>
    </div>

    {#if activeTab === 'keys'}
      <div class="help-columns">
        <div class="help-column">
          <h3 class="section-title">Normal Mode</h3>
          <div class="binding-list">
            {#each normalBindings as binding}
              <div class="binding-row">
                <kbd class="binding-key">{binding.keys}</kbd>
                <span class="binding-desc">{binding.description}</span>
              </div>
            {/each}
          </div>

          <h3 class="section-title">Special Keys</h3>
          <div class="binding-list">
            {#each specialKeys as binding}
              <div class="binding-row">
                <kbd class="binding-key">{binding.keys}</kbd>
                <span class="binding-desc">{binding.description}</span>
              </div>
            {/each}
          </div>
        </div>

        <div class="help-column">
          <h3 class="section-title">Visual Mode</h3>
          <div class="binding-list">
            {#each visualBindings as binding}
              <div class="binding-row">
                <kbd class="binding-key">{binding.keys}</kbd>
                <span class="binding-desc">{binding.description}</span>
              </div>
            {/each}
          </div>

          <h3 class="section-title">Insert Mode</h3>
          <div class="binding-list">
            {#each insertBindings as binding}
              <div class="binding-row">
                <kbd class="binding-key">{binding.keys}</kbd>
                <span class="binding-desc">{binding.description}</span>
              </div>
            {/each}
          </div>

          <h3 class="section-title">Quick Reference</h3>
          <div class="binding-list">
            <div class="binding-row">
              <kbd class="binding-key">gi</kbd>
              <span class="binding-desc">Go to inbox, sync, read</span>
            </div>
            <div class="binding-row">
              <kbd class="binding-key">j/k</kbd>
              <span class="binding-desc">Browse messages</span>
            </div>
            <div class="binding-row">
              <kbd class="binding-key">Enter</kbd>
              <span class="binding-desc">Open + mark read</span>
            </div>
            <div class="binding-row">
              <kbd class="binding-key">r</kbd>
              <span class="binding-desc">Reply inline</span>
            </div>
            <div class="binding-row">
              <kbd class="binding-key">v j/k d</kbd>
              <span class="binding-desc">Visual select + delete batch</span>
            </div>
          </div>
        </div>
      </div>
    {:else}
      <div class="help-columns">
        <div class="help-column">
          <h3 class="section-title">Commands</h3>
          <div class="binding-list">
            {#each commands.slice(0, Math.ceil(commands.length / 2)) as cmd}
              <div class="binding-row">
                <kbd class="binding-key cmd-key">{cmd.cmd}</kbd>
                <span class="binding-desc">{cmd.description}</span>
              </div>
            {/each}
          </div>
        </div>
        <div class="help-column">
          <h3 class="section-title">&nbsp;</h3>
          <div class="binding-list">
            {#each commands.slice(Math.ceil(commands.length / 2)) as cmd}
              <div class="binding-row">
                <kbd class="binding-key cmd-key">{cmd.cmd}</kbd>
                <span class="binding-desc">{cmd.description}</span>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    <p class="help-footer">Press <kbd>?</kbd> or click outside to close</p>
  </div>
</div>

<style>
  .help-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .help-panel {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 24px 28px;
    max-width: 800px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
  }

  .help-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 12px;
    text-align: center;
  }

  .state-bar {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 8px 12px;
    margin-bottom: 16px;
    background: var(--bg-tertiary);
    border-radius: 6px;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .state-item strong {
    color: var(--text-primary);
  }

  .state-sep {
    color: var(--border);
  }

  .tab-bar {
    display: flex;
    gap: 4px;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0;
  }

  .tab {
    padding: 6px 16px;
    font-size: 12px;
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--text-dim);
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    margin-bottom: -1px;
  }

  .tab:hover {
    color: var(--text-secondary);
  }

  .tab.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
  }

  .help-columns {
    display: flex;
    gap: 28px;
  }

  .help-column {
    flex: 1;
  }

  .section-title {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
    margin-top: 16px;
  }

  .section-title:first-child {
    margin-top: 0;
  }

  .binding-list {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .binding-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 2px 0;
  }

  .binding-key {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 1px 6px;
    min-width: 28px;
    text-align: center;
    flex-shrink: 0;
  }

  .cmd-key {
    min-width: auto;
    text-align: left;
    white-space: nowrap;
  }

  .binding-desc {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .help-footer {
    text-align: center;
    font-size: 11px;
    color: var(--text-dim);
    margin-top: 20px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }

  .help-footer kbd {
    font-family: var(--font-mono);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 0 4px;
    font-size: 11px;
  }
</style>
