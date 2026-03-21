<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import HintBar from '$lib/components/HintBar.svelte';
  import MessageList from '$lib/components/MessageList.svelte';
  import ReadingPane from '$lib/components/ReadingPane.svelte';
  import ComposeView from '$lib/components/ComposeView.svelte';
  import AccountSetup from '$lib/components/AccountSetup.svelte';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import HelpOverlay from '$lib/components/HelpOverlay.svelte';
  import ContactsList from '$lib/components/ContactsList.svelte';
  import CalendarView from '$lib/components/CalendarView.svelte';
  import { handleKeyDown, setBindings, registerHandler } from '$lib/keybindings/engine';
  import { defaultBindings } from '$lib/keybindings/bindings';
  import { searchOpen, helpOpen } from '$lib/stores/ui';
  import { accounts, activeAccount, activeFolder, type Account } from '$lib/stores/accounts';
  import { syncFolder, loadMessages, selectedMessage } from '$lib/stores/messages';

  let composing = false;
  let composeMode: 'compose' | 'reply' | 'reply-all' | 'forward' = 'compose';
  let showContacts = false;
  let showCalendar = false;
  let hasAccounts = false;
  let isSearchOpen = false;
  let isHelpOpen = false;

  searchOpen.subscribe((v) => (isSearchOpen = v));
  helpOpen.subscribe((v) => (isHelpOpen = v));

  function openCompose(mode: 'compose' | 'reply' | 'reply-all' | 'forward'): void {
    composeMode = mode;
    composing = true;
    showContacts = false;
    showCalendar = false;
  }

  async function navigateToFolder(folder: string): Promise<void> {
    activeFolder.set(folder);
    showContacts = false;
    showCalendar = false;
    await loadMessages(folder);
  }

  async function initAccounts(): Promise<void> {
    try {
      const accountList = await invoke<Account[]>('list_accounts');
      accounts.set(accountList);
      if (accountList.length > 0) {
        hasAccounts = true;
        activeAccount.set(accountList[0]);
        await invoke('set_active_account', { id: accountList[0].id });
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

    // :contacts command
    registerHandler('cmd:contacts', () => {
      composing = false;
      showContacts = true;
      showCalendar = false;
    });

    // :calendar command
    registerHandler('cmd:calendar', () => {
      showCalendar = true;
      composing = false;
      showContacts = false;
    });

    // Archive: delete from current folder (simplified for v1)
    registerHandler('archive', async () => {
      const msg = get(selectedMessage);
      if (!msg) return;
      await invoke('delete_message', { uid: msg.uid, folder: msg.folder });
      await loadMessages(get(activeFolder));
    });

    // Delete: same as archive for v1
    registerHandler('delete', async () => {
      const msg = get(selectedMessage);
      if (!msg) return;
      await invoke('delete_message', { uid: msg.uid, folder: msg.folder });
      await loadMessages(get(activeFolder));
    });

    // Star/flag toggle
    registerHandler('star', async () => {
      const msg = get(selectedMessage);
      if (!msg) return;
      const currentFlags = msg.flags ?? '';
      const newFlags = currentFlags.includes('Flagged')
        ? currentFlags.replace('Flagged', '').trim()
        : `${currentFlags} Flagged`.trim();
      await invoke('update_flags', { uid: msg.uid, folder: msg.folder, flags: newFlags });
      await loadMessages(get(activeFolder));
    });

    // Mark unread
    registerHandler('mark-unread', async () => {
      const msg = get(selectedMessage);
      if (!msg) return;
      const currentFlags = msg.flags ?? '';
      const newFlags = currentFlags.replace('Seen', '').trim();
      await invoke('update_flags', { uid: msg.uid, folder: msg.folder, flags: newFlags });
      await loadMessages(get(activeFolder));
    });

    // Account switching (1-9)
    for (let i = 1; i <= 9; i++) {
      registerHandler(`switch-account-${i}`, async () => {
        const accts = get(accounts);
        if (i <= accts.length) {
          activeAccount.set(accts[i - 1]);
          await invoke('set_active_account', { id: accts[i - 1].id });
          activeFolder.set('INBOX');
          await syncFolder('INBOX');
        }
      });
    }

    initAccounts();

    // Background sync every 5 minutes
    const syncInterval = setInterval(async () => {
      if (hasAccounts) {
        try {
          await syncFolder(get(activeFolder));
        } catch (e) {
          console.error('Background sync failed:', e);
        }
      }
    }, 5 * 60 * 1000);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      clearInterval(syncInterval);
    };
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
        {:else if showCalendar}
          <CalendarView />
        {:else if showContacts}
          <ContactsList />
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
