<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { accounts, activeAccount } from '$lib/stores/accounts';

  const dispatch = createEventDispatcher();

  let name = '';
  let email = '';
  let password = '';
  let imapHost = '';
  let imapPort = 993;
  let smtpHost = '';
  let smtpPort = 587;
  let username = '';

  let testing = false;
  let saving = false;
  let error = '';
  let success = '';

  async function selectProvider(provider: string): Promise<void> {
    error = '';
    success = '';
    try {
      // Returns [imap_host, imap_port, smtp_host, smtp_port] or null
      const defaults = await invoke<[string, number, string, number] | null>('get_provider_defaults', { provider });
      if (defaults) {
        imapHost = defaults[0];
        imapPort = defaults[1];
        smtpHost = defaults[2];
        smtpPort = defaults[3];
      }
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  async function testConnection(): Promise<void> {
    error = '';
    success = '';
    testing = true;
    try {
      await invoke('test_imap_connection', {
        host: imapHost,
        port: imapPort,
        username: username || email,
        password,
      });
      success = 'Connection successful!';
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      testing = false;
    }
  }

  async function saveAccount(): Promise<void> {
    error = '';
    success = '';
    saving = true;
    try {
      // add_account returns just the account ID string
      const id = await invoke<string>('add_account', {
        req: {
          name,
          email,
          password,
          imap_host: imapHost,
          imap_port: imapPort,
          smtp_host: smtpHost,
          smtp_port: smtpPort,
          username: username || email,
        },
      });

      // Set active account on the Rust side
      await invoke('set_active_account', { id });

      const account = {
        id,
        name,
        email,
        imap_host: imapHost,
        imap_port: imapPort,
        smtp_host: smtpHost,
        smtp_port: smtpPort,
        username: username || email,
      };
      accounts.update((list) => [...list, account]);
      activeAccount.set(account);
      dispatch('done');
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      saving = false;
    }
  }
</script>

<div class="setup-backdrop">
  <div class="setup-panel">
    <h1 class="setup-title">Welcome to ScouterMail</h1>
    <p class="setup-subtitle">Add your first email account to get started.</p>

    <div class="provider-buttons">
      <button class="provider-btn" on:click={() => selectProvider('gmail')} type="button">Gmail</button>
      <button class="provider-btn" on:click={() => selectProvider('outlook')} type="button">Outlook</button>
      <button class="provider-btn" on:click={() => selectProvider('yahoo')} type="button">Yahoo</button>
    </div>

    {#if error}
      <div class="error-banner">{error}</div>
    {/if}
    {#if success}
      <div class="success-banner">{success}</div>
    {/if}

    <div class="form-fields">
      <div class="field-row">
        <label class="field-label" for="setup-name">Name</label>
        <input id="setup-name" class="field-input" type="text" bind:value={name} placeholder="Display Name" />
      </div>
      <div class="field-row">
        <label class="field-label" for="setup-email">Email</label>
        <input id="setup-email" class="field-input" type="email" bind:value={email} placeholder="you@example.com" />
      </div>
      <div class="field-row">
        <label class="field-label" for="setup-password">Password</label>
        <input id="setup-password" class="field-input" type="password" bind:value={password} placeholder="App password" />
      </div>

      <div class="field-group">
        <div class="field-row">
          <label class="field-label" for="setup-imap-host">IMAP Host</label>
          <input id="setup-imap-host" class="field-input" type="text" bind:value={imapHost} placeholder="imap.example.com" />
        </div>
        <div class="field-row short">
          <label class="field-label" for="setup-imap-port">Port</label>
          <input id="setup-imap-port" class="field-input" type="number" bind:value={imapPort} />
        </div>
      </div>

      <div class="field-group">
        <div class="field-row">
          <label class="field-label" for="setup-smtp-host">SMTP Host</label>
          <input id="setup-smtp-host" class="field-input" type="text" bind:value={smtpHost} placeholder="smtp.example.com" />
        </div>
        <div class="field-row short">
          <label class="field-label" for="setup-smtp-port">Port</label>
          <input id="setup-smtp-port" class="field-input" type="number" bind:value={smtpPort} />
        </div>
      </div>

      <div class="field-row">
        <label class="field-label" for="setup-username">Username</label>
        <input id="setup-username" class="field-input" type="text" bind:value={username} placeholder="Optional (defaults to email)" />
      </div>
    </div>

    <div class="form-actions">
      <button class="action-btn test-btn" on:click={testConnection} disabled={testing || !imapHost || !password} type="button">
        {#if testing}Testing...{:else}Test Connection{/if}
      </button>
      <button class="action-btn save-btn" on:click={saveAccount} disabled={saving || !name || !email || !password || !imapHost || !smtpHost} type="button">
        {#if saving}Saving...{:else}Save Account{/if}
      </button>
    </div>
  </div>
</div>

<style>
  .setup-backdrop {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    background: var(--bg-primary);
  }

  .setup-panel {
    width: 100%;
    max-width: 520px;
    padding: 32px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .setup-title {
    font-size: 22px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 6px;
  }

  .setup-subtitle {
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: 20px;
  }

  .provider-buttons {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }

  .provider-btn {
    flex: 1;
    padding: 8px 12px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    border-radius: 4px;
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 12px;
    transition: border-color 0.15s;
  }

  .provider-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
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

  .success-banner {
    background: rgba(34, 197, 94, 0.15);
    border: 1px solid rgba(34, 197, 94, 0.4);
    color: #22c55e;
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 13px;
    margin-bottom: 12px;
  }

  .form-fields {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-bottom: 20px;
  }

  .field-group {
    display: flex;
    gap: 8px;
  }

  .field-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
  }

  .field-row.short {
    flex: 0 0 120px;
  }

  .field-label {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-dim);
    width: 80px;
    flex-shrink: 0;
    text-align: right;
  }

  .short .field-label {
    width: 36px;
  }

  .field-input {
    flex: 1;
    background: var(--bg-primary);
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

  .field-input[type='number'] {
    width: 70px;
  }

  .form-actions {
    display: flex;
    gap: 10px;
    justify-content: flex-end;
  }

  .action-btn {
    padding: 8px 18px;
    border-radius: 4px;
    cursor: pointer;
    font-family: inherit;
    font-size: 13px;
    font-weight: 500;
    border: none;
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .test-btn {
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    color: var(--text-primary);
  }

  .test-btn:hover:not(:disabled) {
    border-color: var(--text-dim);
  }

  .save-btn {
    background: var(--accent);
    color: white;
  }

  .save-btn:hover:not(:disabled) {
    filter: brightness(1.15);
  }
</style>
