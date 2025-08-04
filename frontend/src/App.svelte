<script>
  import { onMount } from 'svelte';
  import { GetEmails, GetFolders, RefreshEmails, GetActiveAccount, GetAccounts, SetActiveAccount, GetUserPreference, SetUserPreference, ClearEmailContentCache, ForceRefreshEmailContent } from '../wailsjs/go/main/App.js';
  import EmailList from './EmailList.svelte';
  import EmailView from './EmailView.svelte';
  import Sidebar from './Sidebar.svelte';
  import Settings from './Settings.svelte';
  import Notification from './Notification.svelte';
  
  let emails = [];
  let folders = [];
  let accounts = [];
  let selectedEmail = null;
  let currentPage = 1;
  let totalPages = 1;
  let loading = false;
  let error = '';
  let showSettings = false;
  let activeAccount = null;
  let selectedAccount = null;
  let notification = { show: false, message: '', type: 'success' };
  let refreshing = false;
  let selectedFolder = "INBOX";
  const pageSize = 20;
  let totalCount = 0;
  let emailListWidth = 340; // Default width
  
  $: totalPages = Math.max(1, Math.ceil(totalCount / pageSize));

  async function loadUserPreferences() {
    try {
      const savedWidth = await GetUserPreference('emailListWidth');
      if (savedWidth) {
        emailListWidth = parseInt(savedWidth) || 340;
      }
    } catch (e) {
      console.error('Failed to load user preferences:', e);
    }
  }

  async function saveUserPreferences() {
    try {
      await SetUserPreference('emailListWidth', emailListWidth.toString());
    } catch (e) {
      console.error('Failed to save user preferences:', e);
    }
  }

  async function loadAccounts() {
    try {
      accounts = await GetAccounts();
      if (accounts.length > 0) {
        // Set the first account as selected if none is selected
        if (!selectedAccount) {
          selectedAccount = accounts[0];
        }
      }
    } catch (e) {
      console.error('Failed to load accounts:', e);
      accounts = [];
    }
  }

  async function loadActiveAccount() {
    try {
      activeAccount = await GetActiveAccount();
      // If we have a selected account, use it as active
      if (selectedAccount) {
        activeAccount = selectedAccount;
      }
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

  async function refreshEmails() {
    if (refreshing) return;
    
    refreshing = true;
    try {
      await RefreshEmails();
      await loadEmails();
      showNotification('Emails refreshed successfully', 'success');
    } catch (e) {
      console.error('Error refreshing emails:', e);
      const errorMessage = e?.message || e?.toString() || 'Unknown error occurred';
      showNotification('Failed to refresh emails: ' + errorMessage, 'error');
    } finally {
      refreshing = false;
    }
  }

  async function clearEmailContentCache() {
    try {
      await ClearEmailContentCache();
      showNotification('Email content cache cleared', 'success');
    } catch (e) {
      console.error('Error clearing email content cache:', e);
      const errorMessage = e?.message || e?.toString() || 'Unknown error occurred';
      showNotification('Failed to clear email content cache: ' + errorMessage, 'error');
    }
  }

  async function forceRefreshEmailContent(emailId) {
    try {
      await ForceRefreshEmailContent(emailId);
      showNotification('Email content refreshed', 'success');
      // Reload the email if it's currently selected
      if (selectedEmail && selectedEmail.id === emailId) {
        // Trigger a reload of the email content
        selectedEmail = { ...selectedEmail }; // Force re-render
      }
    } catch (e) {
      console.error('Error refreshing email content:', e);
      const errorMessage = e?.message || e?.toString() || 'Unknown error occurred';
      showNotification('Failed to refresh email content: ' + errorMessage, 'error');
    }
  }

  function showNotification(message, type = 'success') {
    notification = { show: true, message, type };
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

  async function selectAccount(account) {
    try {
      // Set the account as active in the backend
      await SetActiveAccount(account.id);
      
      selectedAccount = account;
      activeAccount = account;
      selectedFolder = "INBOX";
      currentPage = 1;
      
      // Reload folders and emails for the new account
      await loadFolders();
      await loadEmails(currentPage, selectedFolder);
      
      showNotification(`Switched to ${account.name}`, 'success');
    } catch (e) {
      console.error('Error switching account:', e);
      showNotification('Failed to switch account', 'error');
    }
  }

  function handleEmailListWidthChange(newWidth) {
    emailListWidth = newWidth;
    // Save the preference after a short delay to avoid too many saves
    setTimeout(() => {
      saveUserPreferences();
    }, 500);
  }

  function openEmail(email) {
    selectedEmail = email;
  }
  
  function closeEmail() {
    selectedEmail = null;
  }
  
  function handleEmailRead(emailId) {
    // Update the email's read status in the emails array
    emails = emails.map(email => {
      if (email.id === emailId) {
        return { ...email, read: true };
      }
      return email;
    });
  }

  function toggleSettings() {
    showSettings = !showSettings;
  }

  onMount(async () => {
    await loadUserPreferences();
    await loadAccounts();
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
  <Sidebar 
    {folders} 
    {selectedFolder} 
    {selectFolder} 
    {accounts} 
    {selectedAccount} 
    {selectAccount} 
  />
  <div class="main-pane">
    <header class="header">
      <div class="header-left">
        <h1>ScouterMail</h1>
        {#if activeAccount}
          <div class="account-info">
            <span class="account-name">{activeAccount.name}</span>
          </div>
        {/if}
      </div>
              <button class="settings-btn" on:click={toggleSettings}>
          ⚙️ Settings
        </button>
      <button class="refresh-btn" on:click={refreshEmails} disabled={refreshing} title="Refresh emails">
        {refreshing ? '⟳' : '↻'}
      </button>
      <button class="clear-cache-btn" on:click={clearEmailContentCache} title="Clear email content cache">
        🗑️
      </button>
    </header>
    
          {#if showSettings}
        <Settings on:close={() => showSettings = false} />
      {:else}
      <div class="content-row" style="--email-list-width: {emailListWidth}px;">
        <div class="email-list-pane" style="width: {emailListWidth}px;">
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
              width={emailListWidth}
              onWidthChange={handleEmailListWidthChange}
              on:nextPage={nextPage}
              on:prevPage={prevPage}
            />
          {/if}
        </div>
        <div class="email-view-pane">
          {#if selectedEmail}
            <EmailView email={selectedEmail} onBack={closeEmail} onEmailRead={handleEmailRead} />
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
  
  <!-- Notification -->
  <Notification 
    bind:show={notification.show}
    message={notification.message} 
    type={notification.type} 
  />
</main>

<style>
.outlook-layout {
  display: flex;
  min-height: 100vh;
  background: #f7f9fa;
  font-family: system-ui, sans-serif;
  width: 100%;
}

.main-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: #fff;
  min-width: 0; /* Allow flex item to shrink below content size */
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

.refresh-btn {
  background: transparent;
  color: #666;
  border: 1px solid #ddd;
  padding: 0.5rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1.2rem;
  transition: all 0.12s;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.refresh-btn:hover {
  background: #f5f5f5;
  color: #333;
  border-color: #ccc;
}

.refresh-btn:disabled {
  background: #f9f9f9;
  color: #ccc;
  cursor: not-allowed;
  border-color: #eee;
}

.refresh-btn:disabled:hover {
  background: #f9f9f9;
  color: #ccc;
}

.clear-cache-btn {
  background: transparent;
  color: #666;
  border: 1px solid #ddd;
  padding: 0.5rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1.2rem;
  transition: all 0.12s;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.clear-cache-btn:hover {
  background: #f5f5f5;
  color: #333;
  border-color: #ccc;
}

.clear-cache-btn:disabled {
  background: #f9f9f9;
  color: #ccc;
  cursor: not-allowed;
  border-color: #eee;
}

.clear-cache-btn:disabled:hover {
  background: #f9f9f9;
  color: #ccc;
}

.content-row {
  display: flex;
  flex: 1;
  min-height: 0;
  width: 100%;
}

.email-list-pane {
  border-right: 1px solid #e0e0e0;
  overflow-y: auto;
  background: #fafdff;
  flex-shrink: 0; /* Prevent shrinking */
}

.email-view-pane {
  flex: 1;
  overflow-y: auto;
  background: #fff;
  padding: 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
  width: calc(100% - var(--email-list-width, 340px)); /* Use CSS variable for dynamic width */
  max-width: calc(100% - var(--email-list-width, 340px)); /* Ensure it doesn't exceed available space */
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
