<script>
  import { GetSettings, SaveSettings } from '../wailsjs/go/main/App.js';
  import { onMount } from 'svelte';
  
  let settings = {
    emailServer: 'imap.gmail.com',
    port: 993,
    useSSL: true,
    username: ''
  };
  let password = '';
  let loading = false;
  let error = '';
  let success = '';
  
  async function loadSettings() {
    try {
      settings = await GetSettings();
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }
  
  async function saveSettings() {
    loading = true;
    error = '';
    success = '';
    
    try {
      await SaveSettings(settings, password);
      success = 'Settings saved successfully!';
      password = ''; // Clear password field
    } catch (e) {
      error = 'Failed to save settings: ' + e.message;
    } finally {
      loading = false;
    }
  }
  
  onMount(loadSettings);
</script>

<div class="settings">
  <h2>Email Settings</h2>
  
  {#if error}
    <div class="error">{error}</div>
  {/if}
  
  {#if success}
    <div class="success">{success}</div>
  {/if}
  
  <form on:submit|preventDefault={saveSettings}>
    <div class="form-group">
      <label for="emailServer">Email Server:</label>
      <input 
        id="emailServer"
        type="text" 
        bind:value={settings.emailServer}
        placeholder="imap.gmail.com"
        required
      />
    </div>
    
    <div class="form-group">
      <label for="port">Port:</label>
      <input 
        id="port"
        type="number" 
        bind:value={settings.port}
        min="1"
        max="65535"
        required
      />
    </div>
    
    <div class="form-group">
      <label>
        <input 
          type="checkbox" 
          bind:checked={settings.useSSL}
        />
        Use SSL/TLS
      </label>
    </div>
    
    <div class="form-group">
      <label for="username">Username:</label>
      <input 
        id="username"
        type="email" 
        bind:value={settings.username}
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
    
    <button type="submit" disabled={loading}>
      {loading ? 'Saving...' : 'Save Settings'}
    </button>
  </form>
</div>

<style>
.settings {
  padding: 2rem;
  max-width: 500px;
  margin: 0 auto;
}

.settings h2 {
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

button {
  background: #0078d4;
  color: white;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1rem;
  margin-top: 1rem;
}

button:disabled {
  background: #ccc;
  cursor: not-allowed;
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