<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';

  export let value = '';
  export let placeholder = '';
  export let id = '';
  export let disabled = false;

  interface Contact {
    email: string;
    name: string | null;
    frequency: number;
  }

  const dispatch = createEventDispatcher();

  let suggestions: Contact[] = [];
  let showSuggestions = false;
  let selectedSuggestion = 0;
  let inputEl: HTMLInputElement;

  function getLastToken(): string {
    const parts = value.split(',');
    return (parts[parts.length - 1] || '').trim();
  }

  async function handleInput(): Promise<void> {
    dispatch('input');
    const token = getLastToken();
    if (token.length < 2) {
      showSuggestions = false;
      suggestions = [];
      return;
    }
    try {
      suggestions = await invoke<Contact[]>('search_contacts', { query: token, limit: 6 });
      showSuggestions = suggestions.length > 0;
      selectedSuggestion = 0;
    } catch {
      showSuggestions = false;
    }
  }

  function selectContact(contact: Contact): void {
    const parts = value.split(',').map((s) => s.trim());
    parts.pop();
    const display = contact.name ? `${contact.name} <${contact.email}>` : contact.email;
    parts.push(display);
    value = parts.join(', ') + ', ';
    showSuggestions = false;
    suggestions = [];
    inputEl?.focus();
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (!showSuggestions) return;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedSuggestion = Math.min(selectedSuggestion + 1, suggestions.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedSuggestion = Math.max(selectedSuggestion - 1, 0);
    } else if (e.key === 'Tab' || e.key === 'Enter') {
      if (suggestions.length > 0) {
        e.preventDefault();
        selectContact(suggestions[selectedSuggestion]);
      }
    } else if (e.key === 'Escape') {
      showSuggestions = false;
    }
  }

  function handleBlur(): void {
    // Delay to allow click on suggestion
    setTimeout(() => { showSuggestions = false; }, 150);
  }
</script>

<div class="address-input-wrapper">
  <input
    bind:this={inputEl}
    bind:value
    {id}
    {placeholder}
    {disabled}
    class="field-input"
    type="text"
    on:input={handleInput}
    on:keydown={handleKeydown}
    on:blur={handleBlur}
    on:focus={handleInput}
    autocomplete="off"
  />
  {#if showSuggestions}
    <div class="suggestions">
      {#each suggestions as contact, i}
        <button
          class="suggestion"
          class:selected={i === selectedSuggestion}
          type="button"
          on:mousedown|preventDefault={() => selectContact(contact)}
        >
          <span class="suggestion-name">{contact.name || ''}</span>
          <span class="suggestion-email">{contact.email}</span>
          <span class="suggestion-freq">{contact.frequency}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .address-input-wrapper {
    position: relative;
    flex: 1;
  }

  .field-input {
    width: 100%;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 8px 10px;
    border-radius: 4px;
    font-family: inherit;
    font-size: 13px;
    outline: none;
  }

  .field-input:focus {
    border-color: var(--accent);
  }

  .field-input:disabled {
    opacity: 0.5;
  }

  .suggestions {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: var(--bg-secondary);
    border: 1px solid var(--accent);
    border-radius: 0 0 4px 4px;
    z-index: 50;
    max-height: 200px;
    overflow-y: auto;
  }

  .suggestion {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 10px;
    border: none;
    background: none;
    color: var(--text-secondary);
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
    text-align: left;
  }

  .suggestion:hover,
  .suggestion.selected {
    background: var(--accent-dim);
  }

  .suggestion-name {
    color: var(--text-primary);
    flex-shrink: 0;
  }

  .suggestion-email {
    color: var(--text-dim);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .suggestion-freq {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 10px;
    flex-shrink: 0;
  }
</style>
