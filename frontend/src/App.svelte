<script>
  import { GetEmails, GetFolders, GetActiveAccount } from '../wailsjs/go/main/App.js';
  import Sidebar from './Sidebar.svelte';
  import EmailList from './EmailList.svelte';
  import EmailView from './EmailView.svelte';
  import AccountManager from './AccountManager.svelte';
  import { onMount } from 'svelte';

  let folders = [];
  let emails = [];
  let totalCount = 0;
  let currentPage = 1;
  const pageSize = 20;
  $: totalPages = Math.max(1, Math.ceil(totalCount / pageSize));
  let loading = false;
  let error = "";
  let selectedFolder = "INBOX";
  let selectedEmail = null;
  let showAccountManager = false;
  let activeAccount = null;

  async function loadActiveAccount() {
    try {
      activeAccount = await GetActiveAccount();
    } catch (e) {
      console.error('Failed to load active account:', e);
    }
  }

  async function loadFolders() {
    try {
      folders = await GetFolders();
    } catch (e) {
      folders = [];
    }
  }

  async function loadEmails(page = 1, folder = "INBOX") {
    loading = true;
    try {
      const result = await GetEmails(page, pageSize);
      emails = result.emails;
      totalCount = result.totalCount;
      error = "";
    } catch (e) {
      emails = [];
      totalCount = 0;
      error = "Failed to fetch emails";
    }
    loading = false;
  }

  function nextPage() {
    if (currentPage < totalPages) {
      currentPage += 1;
      loadEmails(currentPage, selectedFolder);
    }
  }

  function prevPage() {
    if (currentPage > 1) {
      currentPage -= 1;
      loadEmails(currentPage, selectedFolder);
    }
  }

  function selectFolder(folder) {
    selectedFolder = folder;
    currentPage = 1;
    loadEmails(currentPage, folder);
  }

  function openEmail(email) {
    selectedEmail = email;
  }

  function closeEmail() {
    selectedEmail = null;
  }

  function toggleAccountManager() {
    showAccountManager = !showAccountManager;
  }

  onMount(async () => {
    await loadActiveAccount();
    await loadFolders();
    await loadEmails(currentPage, selectedFolder);
  });

  // Reset to first page when emails change
  $: if (emails && emails.length && currentPage > totalPages) {
    currentPage = 1;
  }
</script>

<main class="outlook-layout">
  <Sidebar {folders} {selectedFolder} {selectFolder} />
  <div class="main-pane">
    <header>
      <div class="header-left">
        <h1>{selectedFolder}</h1>
        {#if activeAccount}
          <span class="account-info">({activeAccount.name})</span>
        {/if}
      </div>
      <button class="settings-btn" on:click={toggleAccountManager}>
        ⚙️ Accounts
      </button>
    </header>
    
    {#if showAccountManager}
      <AccountManager />
    {:else}
      <div class="content-row">
        <div class="email-list-pane">
          {#if loading}
            <div class="loading">Loading...</div>
          {:else if error}
            <div class="error">{error}</div>
          {:else}
            <EmailList
              {emails}
              {openEmail}
              {currentPage}
              {totalPages}
              {pageSize}
              on:nextPage={nextPage}
              on:prevPage={prevPage}
            />
          {/if}
        </div>
        <div class="email-view-pane">
          {#if selectedEmail}
            <EmailView email={selectedEmail} onBack={closeEmail} />
          {:else}
            <div class="placeholder">Select an email to view its content</div>
          {/if}
        </div>
      </div>
      
      <!-- Pagination at bottom of page -->
      {#if totalPages > 1}
        <div class="pagination">
          <button on:click={prevPage} disabled={currentPage === 1}>
            « Previous
          </button>
          <span>Page {currentPage} of {totalPages}</span>
          <button on:click={nextPage} disabled={currentPage === totalPages}>
            Next »
          </button>
        </div>
      {/if}
    {/if}
  </div>
</main>

<style>
.outlook-layout {
  display: flex;
  min-height: 100vh;
  background: #f7f9fa;
  font-family: system-ui, sans-serif;
}

.main-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: #fff;
}

header {
  padding: 0.7rem 1.2rem;
  border-bottom: 1px solid #e0e0e0;
  background: #f5f7fa;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
}

h1 {
  font-size: 1.1rem;
  margin: 0;
  color: #333;
  font-weight: 600;
  letter-spacing: 0.5px;
}

.account-info {
  font-size: 0.8rem;
  color: #666;
  margin-left: 0.5rem;
}

.settings-btn {
  background: #0078d4;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background 0.12s;
  white-space: nowrap; /* Prevent text wrapping */
}

.settings-btn:hover {
  background: #0056b3;
}

.content-row {
  display: flex;
  flex: 1;
  min-height: 0;
}

.email-list-pane {
  width: 340px;
  border-right: 1px solid #e0e0e0;
  overflow-y: auto;
  background: #fafdff;
}

.email-view-pane {
  flex: 1;
  overflow-y: auto;
  background: #fff;
  padding: 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.placeholder {
  color: #888;
  padding: 2rem;
  font-size: 1.1rem;
}

.loading, .error {
  padding: 1rem 1.5rem;
  text-align: left;
  color: #666;
}

.error {
  color: #d9534f;
}

.pagination {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 1rem;
  border-top: 1px solid #e0e0e0;
  background: #f5f7fa;
  gap: 1rem;
}

.pagination button {
  background: #0078d4;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background 0.12s;
}

.pagination button:hover:not(:disabled) {
  background: #0056b3;
}

.pagination button:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.pagination span {
  font-weight: 500;
  color: #333;
  font-size: 0.9rem;
}
</style>
