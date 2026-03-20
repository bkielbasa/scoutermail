<script lang="ts">
  import { helpOpen } from '$lib/stores/ui';
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
    <h2 class="help-title">Keyboard Shortcuts</h2>

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
      </div>
    </div>

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
    max-width: 700px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
  }

  .help-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 20px;
    text-align: center;
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
