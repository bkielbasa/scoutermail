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
  let nextPage = 1;
  let allLoaded = false;

  $: totalPages = Math.max(1, Math.ceil(totalCount / pageSize));

  async function loadEmails(page, folder) {
    if (loading) return;
    
    console.log('Loading emails:', { page, folder, loading, allLoaded });
    loading = true;
    error = null;
    
    try {
      const result = await GetEmails(page, pageSize);
      console.log('GetEmails result:', result);
      
      if (result && result.emails) {
        if (page === 1) {
          // First page - replace the list
          emails = result.emails;
          nextPage = 2;
          allLoaded = false;
        } else {
          // Subsequent pages - append to existing list
          emails = [...emails, ...result.emails];
          nextPage = page + 1;
        }
        
        totalCount = result.totalCount || 0;
        
        // Check if we've loaded all emails
        if (result.emails.length < pageSize) {
          allLoaded = true;
        }
        
        console.log('Emails loaded:', { 
          emailsLength: emails.length, 
          nextPage, 
          allLoaded, 
          totalCount 
        });
      }
    } catch (e) {
      console.error('Error loading emails:', e);
      error = e?.message || e?.toString() || 'Failed to load emails';
    } finally {
      loading = false;
      console.log('Loading set to false');
    }
  }

  async function loadMoreEmails(e) {
    e.preventDefault();
    e.stopPropagation();
    
    if (loading || allLoaded) {
      console.log('loadMoreEmails blocked:', { loading, allLoaded });
      return;
    }
    
    console.log('Loading more emails, page:', nextPage);
    await loadEmails(nextPage, selectedFolder);
  }

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

  function selectFolder(folder) {
    selectedFolder = folder;
    nextPage = 1;
    allLoaded = false;
    loadEmails(1, folder); // Load first page of the selected folder
  }

  async function selectAccount(account) {
    selectedAccount = account;
    activeAccount = account;
    selectedFolder = "INBOX";
    nextPage = 1;
    allLoaded = false;
    
    // Reload folders and emails for the new account
    await loadFolders();
    await loadEmails(1, selectedFolder);
    
    showNotification(`Switched to ${account.name}`, 'success');
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
    await loadEmails(1, selectedFolder);
  });

  // Reset to first page when emails change
  $: if (emails && emails.length && totalPages > 0 && totalPages < 1) {
    // This condition will never be true if totalPages is 0 or less,
    // but the original code had it. Keeping it as is.
    // If totalPages is 0, it means no emails were loaded, so we should
    // ideally load the first page. However, the loadEmails function
    // already handles loading the first page if page is 1.
    // So, this reset logic might be redundant or need adjustment
    // depending on how totalPages is calculated.
    // For now, keeping it as is, but it might not have an effect
    // if totalPages is 0 or less.
  }
</script>

<main class="outlook-layout">
  <div class="main-content">
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
          <!-- App name moved to sidebar -->
        </div>
        <div class="header-right">
          <button class="settings-btn" on:click={toggleSettings} title="Settings">
            ⚙️
          </button>
          <button class="refresh-btn" on:click={refreshEmails} disabled={refreshing} title="Refresh emails">
            {refreshing ? '⟳' : '↻'}
          </button>
          <button class="clear-cache-btn" on:click={clearEmailContentCache} title="Clear email content cache">
            🗑️
          </button>
        </div>
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
                width={emailListWidth}
                onWidthChange={handleEmailListWidthChange}
                {loading}
                {allLoaded}
              />
            {/if}
            
            <!-- Load More button below email list -->
            {#if !loading && !error && !allLoaded && emails.length > 0}
              <div class="load-more-container">
                <button class="load-more-btn" on:click|preventDefault={loadMoreEmails}>
                  Load More Emails
                </button>
              </div>
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
        
        <!-- Remove pagination bar at the bottom -->
      {/if}
    </div>
  </div>
  
  <!-- Footer -->
  <footer class="app-footer">
    <div class="footer-content">
      <span class="app-name">ScouterMail</span>
      <span class="footer-divider">•</span>
      <span class="footer-text">Email client for modern productivity</span>
    </div>
  </footer>
  
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
  flex-direction: column;
  min-height: 100vh;
  background: #f7f9fa;
  font-family: system-ui, sans-serif;
  width: 100%;
}

.main-content {
  display: flex;
  flex: 1;
  min-height: 0;
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

.header-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
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

.settings-btn:hover {
  background: #f5f5f5;
  color: #333;
  border-color: #ccc;
}

.settings-btn:active {
  background: #e6f3ff;
  color: #0078d4;
  border-color: #0078d4;
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

.app-footer {
  background: #f5f7fa;
  border-top: 1px solid #e0e0e0;
  padding: 0.75rem 1.5rem;
  display: flex;
  justify-content: center;
  align-items: center;
  flex-shrink: 0;
}

.footer-content {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-size: 0.85rem;
  color: #666;
}

.app-name {
  font-weight: 600;
  color: #1976d2;
}

.footer-divider {
  color: #ccc;
}

.footer-text {
  color: #888;
}

.load-more-container {
  padding: 1rem 1.5rem;
  text-align: center;
  background: #f5f7fa;
  border-top: 1px solid #e0e0e0;
}

.load-more-btn {
  background: #1976d2;
  color: white;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  font-size: 1rem;
  font-weight: 600;
  transition: background-color 0.2s ease;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.load-more-btn:hover {
  background: #1565c0;
}

.load-more-btn:active {
  background: #0d47a1;
}

.load-more-btn:disabled {
  background: #ccc;
  color: #666;
  cursor: not-allowed;
  box-shadow: none;
}
</style>
