<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  const dispatch = createEventDispatcher();

  let signature = '';
  let saving = false;
  let error = '';

  onMount(async () => {
    try {
      const val = await invoke<string | null>('get_setting', { key: 'signature' });
      if (val) {
        signature = val;
      }
    } catch {
      // No signature set yet
    }
  });

  async function handleSave(): Promise<void> {
    saving = true;
    error = '';
    try {
      await invoke('set_setting', { key: 'signature', value: signature });
      dispatch('close');
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      saving = false;
    }
  }

  function handleCancel(): void {
    dispatch('close');
  }

  function handleBackdropClick(event: MouseEvent): void {
    if (event.target === event.currentTarget) {
      handleCancel();
    }
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      handleCancel();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y_interactive_supports_focus a11y_click_events_have_key_events -->
<div class="sig-backdrop" on:click={handleBackdropClick} role="dialog" aria-modal="true" aria-label="Edit signature">
  <div class="sig-panel">
    <h2 class="sig-title">Edit Signature</h2>

    {#if error}
      <div class="error-banner">{error}</div>
    {/if}

    <textarea
      class="sig-input"
      bind:value={signature}
      placeholder="Your Name&#10;Title&#10;Company"
      rows="8"
    ></textarea>

    <div class="sig-preview">
      <span class="preview-label">Preview:</span>
      <pre class="preview-text">-- {'\n'}{signature}</pre>
    </div>

    <div class="sig-actions">
      <button class="cancel-btn" on:click={handleCancel} type="button">Cancel</button>
      <button class="save-btn" on:click={handleSave} disabled={saving} type="button">
        {#if saving}Saving...{:else}Save{/if}
      </button>
    </div>
  </div>
</div>

<style>
  .sig-backdrop {
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

  .sig-panel {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 24px 28px;
    max-width: 500px;
    width: 90%;
  }

  .sig-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 16px 0;
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #ef4444;
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 13px;
    margin-bottom: 12px;
  }

  .sig-input {
    width: 100%;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 10px 12px;
    border-radius: 4px;
    font-family: monospace;
    font-size: 13px;
    line-height: 1.5;
    resize: vertical;
    outline: none;
    box-sizing: border-box;
  }

  .sig-input:focus {
    border-color: var(--accent);
  }

  .sig-preview {
    margin-top: 12px;
    padding: 8px 12px;
    background: var(--bg-tertiary);
    border-radius: 4px;
  }

  .preview-label {
    font-size: 11px;
    color: var(--text-dim);
    font-family: monospace;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .preview-text {
    font-family: monospace;
    font-size: 12px;
    color: var(--text-secondary);
    margin: 4px 0 0 0;
    white-space: pre-wrap;
  }

  .sig-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 16px;
  }

  .cancel-btn {
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-family: inherit;
    font-size: 13px;
  }

  .cancel-btn:hover {
    color: var(--text-primary);
    border-color: var(--text-dim);
  }

  .save-btn {
    background: var(--accent);
    border: none;
    color: white;
    padding: 8px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-family: inherit;
    font-size: 13px;
    font-weight: 500;
  }

  .save-btn:hover:not(:disabled) {
    filter: brightness(1.15);
  }

  .save-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
