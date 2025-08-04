<script>
  import { GetAccounts, SaveAccount, DeleteAccount } from '../wailsjs/go/main/App.js';
  import { onMount } from 'svelte';
  
  let accounts = [];
  let selectedAccount = null;
  let showAddForm = false;
  let loading = false;
  let error = '';
  let success = '';
  
  let newAccount = {
    id: 0,
    name: '',
    emailServer: 'imap.gmail.com',
    port: 993,
    useSSL: true,
    username: '',
    isActive: false
  };
  let password = '';
  
  async function loadAccounts() {
    try {
      accounts = await GetAccounts();
    } catch (e) {
      console.error('Failed to load accounts:', e);
      error = 'Failed to load accounts';
    }
  }
  
  async function saveAccount() {
    loading = true;
    error = '';
    success = '';
    
    try {
      await SaveAccount(newAccount, password);
      success = 'Account saved successfully!';
      password = '';
      newAccount = {
        id: 0,
        name: '',
        emailServer: 'imap.gmail.com',
        port: 993,
        useSSL: true,
        username: '',
        isActive: false
      };
      showAddForm = false;
      await loadAccounts();
    } catch (e) {
      error = 'Failed to save account: ' + e.message;
    } finally {
      loading = false;
    }
  }
  
  async function deleteAccount(id) {
    if (!confirm('Are you sure you want to delete this account?')) {
      return;
    }
    
    try {
      await DeleteAccount(id);
      success = 'Account deleted successfully!';
      await loadAccounts();
    } catch (e) {
      error = 'Failed to delete account: ' + e.message;
    }
  }
  
  function editAccount(account) {
    newAccount = { ...account };
    password = '';
    showAddForm = true;
  }
  
  function addNewAccount() {
    newAccount = {
      id: 0,
      name: '',
      emailServer: 'imap.gmail.com',
      port: 993,
      useSSL: true,
      username: '',
      isActive: false
    };
    password = '';
    showAddForm = true;
  }
  
  onMount(loadAccounts);
</script>

<div class="account-manager">
  <h2>Email Accounts</h2>
  
  {#if error}
    <div class="error">{error}</div>
  {/if}
  
  {#if success}
    <div class="success">{success}</div>
  {/if}
  
  <!-- Account List -->
  <div class="account-list">
    {#each accounts as account}
      <div class="account-item">
        <div class="account-info">
          <h3>{account.name}</h3>
          <p><strong>Server:</strong> {account.emailServer}:{account.port}</p>
          <p><strong>Username:</strong> {account.username}</p>
          <p><strong>SSL:</strong> {account.useSSL ? 'Yes' : 'No'}</p>
          {#if account.isActive}
            <span class="active-badge">Active</span>
          {/if}
        </div>
        <div class="account-actions">
          <button class="edit-btn" on:click={() => editAccount(account)}>
            Edit
          </button>
          <button class="delete-btn" on:click={() => deleteAccount(account.id)}>
            Delete
          </button>
        </div>
      </div>
    {/each}
  </div>
  
  <!-- Add/Edit Form -->
  {#if showAddForm}
    <div class="form-overlay">
      <div class="form-container">
        <h3>{newAccount.id === 0 ? 'Add New Account' : 'Edit Account'}</h3>
        
        <form on:submit|preventDefault={saveAccount}>
          <div class="form-group">
            <label for="name">Account Name:</label>
            <input 
              id="name"
              type="text" 
              bind:value={newAccount.name}
              placeholder="My Gmail Account"
              required
            />
          </div>
          
          <div class="form-group">
            <label for="emailServer">Email Server:</label>
            <input 
              id="emailServer"
              type="text" 
              bind:value={newAccount.emailServer}
              placeholder="imap.gmail.com"
              required
            />
          </div>
          
          <div class="form-group">
            <label for="port">Port:</label>
            <input 
              id="port"
              type="number" 
              bind:value={newAccount.port}
              min="1"
              max="65535"
              required
            />
          </div>
          
          <div class="form-group">
            <label>
              <input 
                type="checkbox" 
                bind:checked={newAccount.useSSL}
              />
              Use SSL/TLS
            </label>
          </div>
          
          <div class="form-group">
            <label for="username">Username:</label>
            <input 
              id="username"
              type="email" 
              bind:value={newAccount.username}
              placeholder="your-email@gmail.com"
              required
            />
          </div>
          
          <div class="form-group">
            <label for="password">Password:</label>
            <input 
              id="password"
              type="password" 
              bind:value={password}
              placeholder="Enter your password"
              required
            />
            <small>Password will be stored securely in your system keyring</small>
          </div>
          
          <div class="form-group">
            <label>
              <input 
                type="checkbox" 
                bind:checked={newAccount.isActive}
              />
              Set as active account
            </label>
          </div>
          
          <div class="form-actions">
            <button type="submit" disabled={loading}>
              {loading ? 'Saving...' : 'Save Account'}
            </button>
            <button type="button" on:click={() => showAddForm = false}>
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
  
  <!-- Add Account Button -->
  <button class="add-btn" on:click={addNewAccount}>
    + Add New Account
  </button>
</div>

<style>
.account-manager {
  padding: 2rem;
  max-width: 800px;
  margin: 0 auto;
}

.account-manager h2 {
  margin-bottom: 1.5rem;
  color: #333;
}

.account-list {
  margin-bottom: 2rem;
}

.account-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  margin-bottom: 1rem;
  background: #fff;
}

.account-info h3 {
  margin: 0 0 0.5rem 0;
  color: #333;
}

.account-info p {
  margin: 0.25rem 0;
  color: #666;
  font-size: 0.9rem;
}

.active-badge {
  background: #28a745;
  color: white;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 500;
}

.account-actions {
  display: flex;
  gap: 0.5rem;
}

.edit-btn, .delete-btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
}

.edit-btn {
  background: #0078d4;
  color: white;
}

.delete-btn {
  background: #dc3545;
  color: white;
}

.add-btn {
  background: #28a745;
  color: white;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1rem;
  width: 100%;
}

.form-overlay {
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

.form-container {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  max-width: 500px;
  width: 90%;
  max-height: 90vh;
  overflow-y: auto;
}

.form-container h3 {
  margin-bottom: 1.5rem;
  color: #333;
}

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 500;
  color: #333;
}

.form-group input[type="text"],
.form-group input[type="number"],
.form-group input[type="email"],
.form-group input[type="password"] {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 1rem;
}

.form-group input[type="checkbox"] {
  margin-right: 0.5rem;
}

.form-group small {
  display: block;
  margin-top: 0.25rem;
  color: #666;
  font-size: 0.8rem;
}

.form-actions {
  display: flex;
  gap: 1rem;
  margin-top: 1.5rem;
}

.form-actions button {
  flex: 1;
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1rem;
}

.form-actions button[type="submit"] {
  background: #0078d4;
  color: white;
}

.form-actions button[type="submit"]:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.form-actions button[type="button"] {
  background: #6c757d;
  color: white;
}

.error {
  background: #f8d7da;
  color: #721c24;
  padding: 0.75rem;
  border-radius: 4px;
  margin-bottom: 1rem;
}

.success {
  background: #d4edda;
  color: #155724;
  padding: 0.75rem;
  border-radius: 4px;
  margin-bottom: 1rem;
}
</style> 