<script>
  import { onMount, createEventDispatcher } from 'svelte';
  import { GetAccounts, GetActiveAccount, SaveAccount, DeleteAccount, GetPassword, GetUserPreference, SetUserPreference } from '../wailsjs/go/main/App.js';
  import Notification from './Notification.svelte';
  
  const dispatch = createEventDispatcher();
  
  let accounts = [];
  let activeAccount = null;
  let showAddModal = false;
  let editingAccount = null;
  let currentTab = 'accounts';
  let notification = { show: false, message: '', type: 'success' };
  
  // Form data
  let formData = {
    name: '',
    emailServer: '',
    port: 993,
    useSSL: true,
    username: '',
    password: ''
  };
  
  // Appearance settings
  let appearanceSettings = {
    emailListWidth: 340,
    theme: 'light',
    fontSize: 'medium'
  };
  
  const tabs = [
    { id: 'accounts', label: 'Email Accounts', icon: '📧' },
    { id: 'general', label: 'General', icon: '⚙️' },
    { id: 'appearance', label: 'Appearance', icon: '🎨' },
    { id: 'advanced', label: 'Advanced', icon: '🔧' }
  ];
  
  onMount(async () => {
    await loadAccounts();
    await loadAppearanceSettings();
  });
  
  async function loadAppearanceSettings() {
    try {
      const savedWidth = await GetUserPreference('emailListWidth');
      if (savedWidth) {
        appearanceSettings.emailListWidth = parseInt(savedWidth) || 340;
      }
      
      const savedTheme = await GetUserPreference('theme');
      if (savedTheme) {
        appearanceSettings.theme = savedTheme;
      }
      
      const savedFontSize = await GetUserPreference('fontSize');
      if (savedFontSize) {
        appearanceSettings.fontSize = savedFontSize;
      }
    } catch (e) {
      console.error('Failed to load appearance settings:', e);
    }
  }
  
  async function saveAppearanceSettings() {
    try {
      await SetUserPreference('emailListWidth', appearanceSettings.emailListWidth.toString());
      await SetUserPreference('theme', appearanceSettings.theme);
      await SetUserPreference('fontSize', appearanceSettings.fontSize);
      showNotification('Appearance settings saved', 'success');
    } catch (e) {
      console.error('Failed to save appearance settings:', e);
      showNotification('Failed to save appearance settings', 'error');
    }
  }
  
  function closeSettings() {
    dispatch('close');
  }
  
  async function loadAccounts() {
    try {
      accounts = await GetAccounts();
      activeAccount = await GetActiveAccount();
    } catch (e) {
      console.error('Error loading accounts:', e);
      showNotification('Failed to load accounts', 'error');
    }
  }
  
  function showNotification(message, type = 'success') {
    notification = { show: true, message, type };
  }
  
  function openAddModal() {
    editingAccount = null;
    formData = {
      name: '',
      emailServer: '',
      port: 993,
      useSSL: true,
      username: '',
      password: ''
    };
    showAddModal = true;
  }
  
  function openEditModal(account) {
    editingAccount = account;
    formData = {
      name: account.name,
      emailServer: account.emailServer,
      port: account.port,
      useSSL: account.useSSL,
      username: account.username,
      password: ''
    };
    showAddModal = true;
  }
  
  async function saveAccount() {
    try {
      const accountData = {
        id: editingAccount?.id || 0,
        name: formData.name,
        emailServer: formData.emailServer,
        port: formData.port,
        useSSL: formData.useSSL,
        username: formData.username,
        isActive: false
      };
      
      await SaveAccount(accountData, formData.password);
      await loadAccounts();
      showAddModal = false;
      showNotification(editingAccount ? 'Account updated' : 'Account added', 'success');
    } catch (e) {
      console.error('Error saving account:', e);
      showNotification('Failed to save account', 'error');
    }
  }
  
  async function deleteAccount(account) {
    if (!confirm(`Are you sure you want to delete "${account.name}"?`)) {
      return;
    }
    
    try {
      await DeleteAccount(account.id);
      await loadAccounts();
      showNotification('Account deleted', 'success');
    } catch (e) {
      console.error('Error deleting account:', e);
      showNotification('Failed to delete account', 'error');
    }
  }
  
  async function setActiveAccount(account) {
    try {
      // Set all accounts as inactive first
      for (const acc of accounts) {
        if (acc.id !== account.id) {
          await SaveAccount({ ...acc, isActive: false }, '');
        }
      }
      
      // Set the selected account as active
      await SaveAccount({ ...account, isActive: true }, '');
      await loadAccounts();
      showNotification(`${account.name} set as active`, 'success');
    } catch (e) {
      console.error('Error setting active account:', e);
      showNotification('Failed to set active account', 'error');
    }
  }
  
  function selectTab(tabId) {
    currentTab = tabId;
  }
</script>

<div class="settings-overlay" on:click|self={closeSettings}>
  <div class="settings-modal">
    <div class="settings-header">
      <h2>Settings</h2>
      <button class="close-btn" on:click={closeSettings}>×</button>
    </div>
    
    <div class="settings-content">
      <!-- Tab Navigation -->
      <div class="tabs">
        {#each tabs as tab}
          <button 
            class="tab-btn {currentTab === tab.id ? 'active' : ''}"
            on:click={() => selectTab(tab.id)}
          >
            <span class="tab-icon">{tab.icon}</span>
            {tab.label}
          </button>
        {/each}
      </div>
      
      <!-- Tab Content -->
      <div class="tab-content">
        {#if currentTab === 'accounts'}
          <div class="accounts-tab">
            <div class="section-header">
              <h3>Email Accounts</h3>
              <button class="add-btn" on:click={openAddModal}>
                + Add Account
              </button>
            </div>
            
            <div class="accounts-list">
              {#each accounts as account}
                <div class="account-item {account.isActive ? 'active' : ''}">
                  <div class="account-info">
                    <div class="account-name">
                      {account.name}
                      {#if account.isActive}
                        <span class="active-badge">Active</span>
                      {/if}
                    </div>
                    <div class="account-details">
                      {account.username} @ {account.emailServer}
                    </div>
                  </div>
                  <div class="account-actions">
                    {#if !account.isActive}
                      <button class="action-btn set-active" on:click={() => setActiveAccount(account)}>
                        Set Active
                      </button>
                    {/if}
                    <button class="action-btn edit" on:click={() => openEditModal(account)}>
                      Edit
                    </button>
                    <button class="action-btn delete" on:click={() => deleteAccount(account)}>
                      Delete
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {:else if currentTab === 'general'}
          <div class="general-tab">
            <h3>General Settings</h3>
            <p>General settings will be implemented here.</p>
          </div>
        {:else if currentTab === 'appearance'}
          <div class="appearance-tab">
            <div class="section-header">
              <h3>Appearance Settings</h3>
              <button class="save-btn" on:click={saveAppearanceSettings}>
                Save Settings
              </button>
            </div>
            
            <div class="settings-section">
              <h4>Email List</h4>
              <div class="setting-group">
                <label for="emailListWidth">Email List Width (px)</label>
                <div class="setting-control">
                  <input 
                    id="emailListWidth"
                    type="range" 
                    min="250" 
                    max="600" 
                    step="10"
                    bind:value={appearanceSettings.emailListWidth}
                  />
                  <span class="setting-value">{appearanceSettings.emailListWidth}px</span>
                </div>
                <small>Drag the resize handle in the email list or use this slider to adjust the width</small>
              </div>
            </div>
            
            <div class="settings-section">
              <h4>Theme</h4>
              <div class="setting-group">
                <label>Theme</label>
                <div class="setting-control">
                  <select bind:value={appearanceSettings.theme}>
                    <option value="light">Light</option>
                    <option value="dark">Dark</option>
                    <option value="auto">Auto (System)</option>
                  </select>
                </div>
                <small>Choose your preferred theme</small>
              </div>
            </div>
            
            <div class="settings-section">
              <h4>Font Size</h4>
              <div class="setting-group">
                <label>Font Size</label>
                <div class="setting-control">
                  <select bind:value={appearanceSettings.fontSize}>
                    <option value="small">Small</option>
                    <option value="medium">Medium</option>
                    <option value="large">Large</option>
                  </select>
                </div>
                <small>Adjust the font size throughout the application</small>
              </div>
            </div>
          </div>
        {:else if currentTab === 'advanced'}
          <div class="advanced-tab">
            <h3>Advanced Settings</h3>
            <p>Advanced settings will be implemented here.</p>
          </div>
        {/if}
      </div>
    </div>
  </div>
  
  <!-- Add/Edit Account Modal -->
  {#if showAddModal}
    <div class="modal-overlay" on:click|self={() => showAddModal = false}>
      <div class="modal">
        <div class="modal-header">
          <h3>{editingAccount ? 'Edit Account' : 'Add Account'}</h3>
          <button class="close-btn" on:click={() => showAddModal = false}>×</button>
        </div>
        
        <form class="modal-form" on:submit|preventDefault={saveAccount}>
          <div class="form-group">
            <label for="name">Account Name</label>
            <input 
              id="name"
              type="text" 
              bind:value={formData.name} 
              placeholder="My Gmail Account"
              required
            />
          </div>
          
          <div class="form-group">
            <label for="emailServer">Email Server</label>
            <input 
              id="emailServer"
              type="text" 
              bind:value={formData.emailServer} 
              placeholder="imap.gmail.com"
              required
            />
          </div>
          
          <div class="form-row">
            <div class="form-group">
              <label for="port">Port</label>
              <input 
                id="port"
                type="number" 
                bind:value={formData.port} 
                min="1" 
                max="65535"
                required
              />
            </div>
            
            <div class="form-group checkbox-group">
              <label>
                <input type="checkbox" bind:checked={formData.useSSL} />
                Use SSL
              </label>
            </div>
          </div>
          
          <div class="form-group">
            <label for="username">Username</label>
            <input 
              id="username"
              type="text" 
              bind:value={formData.username} 
              placeholder="your.email@gmail.com"
              required
            />
          </div>
          
          <div class="form-group">
            <label for="password">Password</label>
            <input 
              id="password"
              type="password" 
              bind:value={formData.password} 
              placeholder="Your password"
              required={!editingAccount}
            />
            {#if editingAccount}
              <small>Leave blank to keep current password</small>
            {/if}
          </div>
          
          <div class="modal-actions">
            <button type="button" class="btn-secondary" on:click={() => showAddModal = false}>
              Cancel
            </button>
            <button type="submit" class="btn-primary">
              {editingAccount ? 'Update' : 'Add'} Account
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
  
  <!-- Notification -->
  <Notification 
    bind:show={notification.show}
    message={notification.message} 
    type={notification.type} 
  />
</div>

<style>
  .settings-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  
  .settings-modal {
    background: white;
    border-radius: 8px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
    width: 90%;
    max-width: 800px;
    max-height: 90vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  
  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 2rem;
    border-bottom: 1px solid #eee;
  }
  
  .settings-header h2 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
  }
  
  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: #666;
    padding: 0.5rem;
    border-radius: 4px;
    transition: background 0.12s;
  }
  
  .close-btn:hover {
    background: #f5f5f5;
  }
  
  .settings-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  
  .tabs {
    width: 200px;
    background: #f8f9fa;
    border-right: 1px solid #eee;
    padding: 1rem 0;
  }
  
  .tab-btn {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    padding: 0.75rem 1.5rem;
    background: none;
    border: none;
    text-align: left;
    cursor: pointer;
    transition: background 0.12s;
    font-size: 0.9rem;
  }
  
  .tab-btn:hover {
    background: #e9ecef;
  }
  
  .tab-btn.active {
    background: #0078d4;
    color: white;
  }
  
  .tab-icon {
    font-size: 1.1rem;
  }
  
  .tab-content {
    flex: 1;
    padding: 2rem;
    overflow-y: auto;
  }
  
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }
  
  .section-header h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }
  
  .add-btn {
    background: #0078d4;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: background 0.12s;
  }
  
  .add-btn:hover {
    background: #0056b3;
  }
  
  .accounts-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .account-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border: 1px solid #eee;
    border-radius: 6px;
    background: white;
    transition: border-color 0.12s;
  }
  
  .account-item.active {
    border-color: #0078d4;
    background: #f0f8ff;
  }
  
  .account-info {
    flex: 1;
  }
  
  .account-name {
    font-weight: 600;
    margin-bottom: 0.25rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .active-badge {
    background: #0078d4;
    color: white;
    padding: 0.25rem 0.5rem;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 500;
  }
  
  .account-details {
    color: #666;
    font-size: 0.9rem;
  }
  
  .account-actions {
    display: flex;
    gap: 0.5rem;
  }
  
  .action-btn {
    padding: 0.25rem 0.75rem;
    border: 1px solid #ddd;
    background: white;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.8rem;
    transition: all 0.12s;
  }
  
  .action-btn:hover {
    background: #f5f5f5;
  }
  
  .action-btn.set-active {
    background: #28a745;
    color: white;
    border-color: #28a745;
  }
  
  .action-btn.set-active:hover {
    background: #218838;
  }
  
  .action-btn.edit {
    background: #ffc107;
    color: #212529;
    border-color: #ffc107;
  }
  
  .action-btn.edit:hover {
    background: #e0a800;
  }
  
  .action-btn.delete {
    background: #dc3545;
    color: white;
    border-color: #dc3545;
  }
  
  .action-btn.delete:hover {
    background: #c82333;
  }
  
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1100;
  }
  
  .modal {
    background: white;
    border-radius: 8px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
    width: 90%;
    max-width: 500px;
    max-height: 90vh;
    overflow-y: auto;
  }
  
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 2rem;
    border-bottom: 1px solid #eee;
  }
  
  .modal-header h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }
  
  .modal-form {
    padding: 2rem;
  }
  
  .form-group {
    margin-bottom: 1.5rem;
  }
  
  .form-row {
    display: flex;
    gap: 1rem;
  }
  
  .form-row .form-group {
    flex: 1;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #333;
  }
  
  .form-group input[type="text"],
  .form-group input[type="number"],
  .form-group input[type="password"] {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 0.9rem;
    transition: border-color 0.12s;
  }
  
  .form-group input:focus {
    outline: none;
    border-color: #0078d4;
  }
  
  .checkbox-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .checkbox-group label {
    margin: 0;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .checkbox-group input[type="checkbox"] {
    margin: 0;
  }
  
  .form-group small {
    color: #666;
    font-size: 0.8rem;
    margin-top: 0.25rem;
    display: block;
  }
  
  .modal-actions {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
    margin-top: 2rem;
  }
  
  .btn-secondary {
    background: #6c757d;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: background 0.12s;
  }
  
  .btn-secondary:hover {
    background: #5a6268;
  }
  
  .btn-primary {
    background: #0078d4;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: background 0.12s;
  }
  
  .btn-primary:hover {
    background: #0056b3;
  }
  
  /* Appearance Settings Styles */
  .settings-section {
    margin-bottom: 2rem;
    padding: 1.5rem;
    border: 1px solid #eee;
    border-radius: 6px;
    background: #fafafa;
  }
  
  .settings-section h4 {
    margin: 0 0 1rem 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: #333;
  }
  
  .setting-group {
    margin-bottom: 1.5rem;
  }
  
  .setting-group:last-child {
    margin-bottom: 0;
  }
  
  .setting-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #333;
  }
  
  .setting-control {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.5rem;
  }
  
  .setting-control input[type="range"] {
    flex: 1;
    height: 6px;
    border-radius: 3px;
    background: #ddd;
    outline: none;
    -webkit-appearance: none;
  }
  
  .setting-control input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #0078d4;
    cursor: pointer;
  }
  
  .setting-control input[type="range"]::-moz-range-thumb {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #0078d4;
    cursor: pointer;
    border: none;
  }
  
  .setting-value {
    font-weight: 600;
    color: #0078d4;
    min-width: 60px;
    text-align: right;
  }
  
  .setting-control select {
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    background: white;
    font-size: 0.9rem;
    min-width: 150px;
  }
  
  .setting-control select:focus {
    outline: none;
    border-color: #0078d4;
  }
  
  .setting-group small {
    color: #666;
    font-size: 0.8rem;
    display: block;
  }
  
  .save-btn {
    background: #28a745;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: background 0.12s;
  }
  
  .save-btn:hover {
    background: #218838;
  }
</style> 