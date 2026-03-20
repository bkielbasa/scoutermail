<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import HintBar from '$lib/components/HintBar.svelte';
  import MessageList from '$lib/components/MessageList.svelte';
  import ReadingPane from '$lib/components/ReadingPane.svelte';
  import ComposeView from '$lib/components/ComposeView.svelte';
  import AccountSetup from '$lib/components/AccountSetup.svelte';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import HelpOverlay from '$lib/components/HelpOverlay.svelte';
  import { handleKeyDown, setBindings, registerHandler } from '$lib/keybindings/engine';
  import { defaultBindings } from '$lib/keybindings/bindings';
  import { searchOpen, helpOpen } from '$lib/stores/ui';
  import { accounts, activeAccount, activeFolder, type Account } from '$lib/stores/accounts';
  import { syncFolder, loadMessages } from '$lib/stores/messages';

  let composing = false;
  let composeMode: 'compose' | 'reply' | 'reply-all' | 'forward' = 'compose';
  let hasAccounts = false;
  let isSearchOpen = false;
  let isHelpOpen = false;

  searchOpen.subscribe((v) => (isSearchOpen = v));
  helpOpen.subscribe((v) => (isHelpOpen = v));

  function openCompose(mode: 'compose' | 'reply' | 'reply-all' | 'forward'): void {
    composeMode = mode;
    composing = true;
  }

  async function navigateToFolder(folder: string): Promise<void> {
    activeFolder.set(folder);
    await loadMessages(folder);
  }

  async function initAccounts(): Promise<void> {
    try {
      const accountList = await invoke<Account[]>('list_accounts');
      accounts.set(accountList);
      if (accountList.length > 0) {
        hasAccounts = true;
        activeAccount.set(accountList[0]);
        try {
          await syncFolder('INBOX');
        } catch {
          // Sync may fail if offline; messages may still be cached
          await loadMessages('INBOX');
        }
      }
    } catch {
      // No accounts yet
    }
  }

  onMount(() => {
    setBindings(defaultBindings);
    window.addEventListener('keydown', handleKeyDown);

    registerHandler('compose', () => openCompose('compose'));
    registerHandler('reply', () => openCompose('reply'));
    registerHandler('reply-all', () => openCompose('reply-all'));
    registerHandler('forward', () => openCompose('forward'));

    registerHandler('goto-inbox', () => navigateToFolder('INBOX'));
    registerHandler('goto-sent', () => navigateToFolder('Sent'));
    registerHandler('goto-drafts', () => navigateToFolder('Drafts'));
    registerHandler('goto-archive', () => navigateToFolder('Archive'));

    initAccounts();

    return () => window.removeEventListener('keydown', handleKeyDown);
  });

  async function handleSetupDone(): Promise<void> {
    hasAccounts = true;
    try {
      await syncFolder('INBOX');
    } catch {
      await loadMessages('INBOX');
    }
  }
</script>

<div id="app">
  {#if !hasAccounts}
    <AccountSetup on:done={handleSetupDone} />
  {:else}
    <StatusBar />
    {#if isSearchOpen}
      <SearchBar />
    {/if}
    <main class="content">
      <div class="message-list-pane">
        <MessageList />
      </div>
      <div class="reading-pane">
        {#if composing}
          <ComposeView replyMode={composeMode} on:close={() => (composing = false)} />
        {:else}
          <ReadingPane />
        {/if}
      </div>
    </main>
    <HintBar />
  {/if}
  {#if isHelpOpen}
    <HelpOverlay />
  {/if}
</div>

<style>
  .content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .message-list-pane {
    width: 33%;
    min-width: 250px;
    max-width: 500px;
    border-right: 1px solid var(--border);
    overflow-y: auto;
  }
  .reading-pane {
    flex: 1;
    overflow-y: auto;
  }
</style>
