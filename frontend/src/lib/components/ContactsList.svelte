<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface Contact {
    email: string;
    name: string | null;
    frequency: number;
  }

  let contacts: Contact[] = [];
  let filtered: Contact[] = [];
  let search = '';
  let loading = true;

  onMount(async () => {
    try {
      contacts = await invoke<Contact[]>('get_all_contacts');
      filtered = contacts;
    } catch (e) {
      console.error('Failed to load contacts:', e);
    } finally {
      loading = false;
    }
  });

  $: {
    if (search.trim()) {
      const q = search.toLowerCase();
      filtered = contacts.filter(
        (c) =>
          c.email.toLowerCase().includes(q) ||
          (c.name && c.name.toLowerCase().includes(q))
      );
    } else {
      filtered = contacts;
    }
  }
</script>

<div class="contacts-view">
  <div class="contacts-header">
    <h2 class="contacts-title">Contacts ({contacts.length})</h2>
    <input
      class="contacts-search"
      type="text"
      bind:value={search}
      placeholder="Filter contacts..."
    />
  </div>

  {#if loading}
    <p class="contacts-empty">Loading...</p>
  {:else if filtered.length === 0}
    <p class="contacts-empty">No contacts found</p>
  {:else}
    <div class="contacts-list">
      {#each filtered as contact}
        <div class="contact-item">
          <div class="contact-avatar">
            {(contact.name || contact.email).charAt(0).toUpperCase()}
          </div>
          <div class="contact-info">
            <div class="contact-name">{contact.name || contact.email.split('@')[0]}</div>
            <div class="contact-email">{contact.email}</div>
          </div>
          <div class="contact-freq" title="Times seen">{contact.frequency}</div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .contacts-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 16px 20px;
    overflow: hidden;
  }

  .contacts-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
    gap: 12px;
  }

  .contacts-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
    white-space: nowrap;
  }

  .contacts-search {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 6px 10px;
    border-radius: 4px;
    font-size: 12px;
    outline: none;
    width: 200px;
  }

  .contacts-search:focus {
    border-color: var(--accent);
  }

  .contacts-list {
    flex: 1;
    overflow-y: auto;
  }

  .contact-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 4px;
    border-bottom: 1px solid var(--bg-tertiary);
  }

  .contact-item:hover {
    background: var(--bg-secondary);
  }

  .contact-avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--accent-dim);
    color: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    font-weight: 600;
    flex-shrink: 0;
  }

  .contact-info {
    flex: 1;
    min-width: 0;
  }

  .contact-name {
    font-size: 13px;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .contact-email {
    font-size: 11px;
    color: var(--text-dim);
    font-family: var(--font-mono);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .contact-freq {
    font-size: 11px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    flex-shrink: 0;
  }

  .contacts-empty {
    color: var(--text-dim);
    text-align: center;
    padding: 24px;
  }
</style>
