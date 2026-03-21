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
  import FolderList from '$lib/components/FolderList.svelte';
  import SignatureEditor from '$lib/components/SignatureEditor.svelte';
  import DraftsList from '$lib/components/DraftsList.svelte';
  import { handleKeyDown, setBindings, registerHandler } from '$lib/keybindings/engine';
  import { defaultBindings } from '$lib/keybindings/bindings';
  import { searchOpen, helpOpen, mode, unifiedMode } from '$lib/stores/ui';
  import { accounts, activeAccount, activeFolder, type Account } from '$lib/stores/accounts';
  import { syncFolder, loadMessages, loadUnifiedMessages, selectedMessage, messages, visualSelection, refreshFolderCounts } from '$lib/stores/messages';

  let composing = false;
  let composeMode: 'compose' | 'reply' | 'reply-all' | 'forward' = 'compose';
  let showContacts = false;
  let showCalendar = false;
  let showFolders = false;
  let showDrafts = false;
  let showSignatureEditor = false;
  let initialDraft: any = null;
  let hasAccounts = false;
  let isSearchOpen = false;
  let isHelpOpen = false;

  let dragging = false;
  let listWidth = 33;

  function startDrag(e: MouseEvent) {
    dragging = true;
    e.preventDefault();
    const onMove = (ev: MouseEvent) => {
      const container = document.querySelector('.content') as HTMLElement;
      if (!container) return;
      const rect = container.getBoundingClientRect();
      let pct = ((ev.clientX - rect.left) / rect.width) * 100;
      pct = Math.max(15, Math.min(50, pct));
      listWidth = pct;
    };
    const onUp = () => {
      dragging = false;
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }

  searchOpen.subscribe((v) => (isSearchOpen = v));
  helpOpen.subscribe((v) => (isHelpOpen = v));

  function openCompose(mode: 'compose' | 'reply' | 'reply-all' | 'forward'): void {
    composeMode = mode;
    composing = true;
    showContacts = false;
    showCalendar = false;
    showDrafts = false;
    initialDraft = null;
  }

  async function navigateToFolder(folder: string): Promise<void> {
    unifiedMode.set(false);
    activeFolder.set(folder);
    showContacts = false;
    showCalendar = false;
    showFolders = false;
    showDrafts = false;
    await loadMessages(folder);
  }

  function handleResumeDraft(e: CustomEvent): void {
    const draft = e.detail;
    initialDraft = draft;
    composeMode = draft.reply_mode || 'compose';
    composing = true;
    showDrafts = false;
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
        await refreshFolderCounts();
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
    registerHandler('goto-drafts', () => {
      showDrafts = true;
      composing = false;
      showContacts = false;
      showCalendar = false;
      showFolders = false;
    });
    registerHandler('goto-archive', () => navigateToFolder('Archive'));

    // :folders command
    registerHandler('cmd:folders', () => {
      showFolders = !showFolders;
    });

    // :contacts command
    registerHandler('cmd:contacts', () => {
      composing = false;
      showContacts = true;
      showCalendar = false;
      showFolders = false;
    });

    // :calendar command
    registerHandler('cmd:calendar', () => {
      showCalendar = true;
      composing = false;
      showContacts = false;
      showFolders = false;
    });

    // :signature command
    registerHandler('cmd:signature', () => {
      showSignatureEditor = !showSignatureEditor;
    });

    // :unified command — show unified inbox across all accounts
    registerHandler('cmd:unified', async () => {
      unifiedMode.set(true);
      activeFolder.set('INBOX');
      showContacts = false;
      showCalendar = false;
      showFolders = false;
      showDrafts = false;
      composing = false;
      await loadUnifiedMessages('INBOX');
    });

    // :move command — move message to another IMAP folder
    registerHandler('cmd:move', async (args?: string) => {
      if (!args) return;
      const msg = get(selectedMessage);
      if (!msg) return;
      await invoke('move_message', { uid: msg.uid, fromFolder: msg.folder, toFolder: args.trim() });
      await loadMessages(get(activeFolder));
      await refreshFolderCounts();
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

    // Visual mode bulk archive
    registerHandler('visual-archive', async () => {
      const sel = get(visualSelection);
      const msgs = get(messages);
      for (const idx of sel) {
        const msg = msgs[idx];
        if (msg) await invoke('delete_message', { uid: msg.uid, folder: msg.folder });
      }
      mode.set('NORMAL');
      visualSelection.set(new Set());
      await loadMessages(get(activeFolder));
    });

    // Visual mode bulk delete
    registerHandler('visual-delete', async () => {
      const sel = get(visualSelection);
      const msgs = get(messages);
      for (const idx of sel) {
        const msg = msgs[idx];
        if (msg) await invoke('delete_message', { uid: msg.uid, folder: msg.folder });
      }
      mode.set('NORMAL');
      visualSelection.set(new Set());
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
      await loadMessages(get(activeFolder), false);
    });

    // Mark as spam
    registerHandler('mark-spam', async () => {
      const msg = get(selectedMessage);
      if (!msg) return;
      await invoke('move_message', { uid: msg.uid, fromFolder: msg.folder, toFolder: 'Junk' });
      await loadMessages(get(activeFolder));
      await refreshFolderCounts();
    });
    registerHandler('cmd:spam', async () => {
      const msg = get(selectedMessage);
      if (!msg) return;
      await invoke('move_message', { uid: msg.uid, fromFolder: msg.folder, toFolder: 'Junk' });
      await loadMessages(get(activeFolder));
      await refreshFolderCounts();
    });

    // Print email
    registerHandler('cmd:print', () => {
      const iframe = document.querySelector('.html-frame') as HTMLIFrameElement;
      if (iframe?.contentWindow) {
        iframe.contentWindow.print();
      } else {
        window.print();
      }
    });

    // Snooze message
    registerHandler('cmd:snooze', async (args?: string) => {
      const msg = get(selectedMessage);
      if (!msg || !args) return;
      const minutes = parseInt(args.trim(), 10);
      if (isNaN(minutes)) return;
      await invoke('snooze_message', { uid: msg.uid, folder: msg.folder, durationMinutes: minutes });
      await loadMessages(get(activeFolder));
    });

    // Mark unread
    registerHandler('mark-unread', async () => {
      const msg = get(selectedMessage);
      if (!msg) return;
      const currentFlags = msg.flags ?? '';
      const newFlags = currentFlags.replace('Seen', '').trim();
      await invoke('update_flags', { uid: msg.uid, folder: msg.folder, flags: newFlags });
      await loadMessages(get(activeFolder), false);
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
          await refreshFolderCounts();
        } catch (e) {
          console.error('Background sync failed:', e);
        }

        // Check for due snoozed messages
        try {
          const snoozed = await invoke<Array<[number, string]>>('check_snoozed');
          if (snoozed.length > 0) {
            for (const [uid, folder] of snoozed) {
              await invoke('unsnooze_message', { uid, folder });
            }
            await loadMessages(get(activeFolder), false);
          }
        } catch (e) {
          console.error('Snooze check failed:', e);
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
    await refreshFolderCounts();
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
      {#if showFolders}
        <FolderList
          activeFolder={get(activeFolder)}
          on:select={(e) => navigateToFolder(e.detail)}
        />
      {/if}
      <div class="message-list-pane" style="width: {listWidth}%">
        <MessageList />
      </div>
      <div
        class="pane-divider"
        class:dragging
        on:mousedown={startDrag}
        role="separator"
        aria-orientation="vertical"
        tabindex="-1"
      ></div>
      <div class="reading-pane">
        {#if composing}
          <ComposeView replyMode={composeMode} {initialDraft} on:close={() => { composing = false; initialDraft = null; }} />
        {:else if showDrafts}
          <DraftsList on:resume={handleResumeDraft} />
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
  {#if showSignatureEditor}
    <SignatureEditor on:close={() => (showSignatureEditor = false)} />
  {/if}
</div>

<style>
  .content {
    display: flex;
    flex: 1;
    overflow: hidden;
    position: relative;
  }
  .message-list-pane {
    min-width: 200px;
    max-width: 600px;
    border-right: 1px solid var(--border);
    overflow-y: auto;
  }
  .pane-divider {
    width: 4px;
    cursor: col-resize;
    background: transparent;
    transition: background 0.15s;
    flex-shrink: 0;
  }
  .pane-divider:hover,
  .pane-divider.dragging {
    background: var(--accent);
  }
  .reading-pane {
    flex: 1;
    overflow-y: auto;
  }
</style>
