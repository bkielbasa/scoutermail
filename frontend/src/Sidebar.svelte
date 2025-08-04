<script>
  import { createEventDispatcher } from 'svelte';
  
  export let accounts = [];
  export let folders = [];
  export let selectedFolder = "";
  export let selectedAccount = null;
  export let selectFolder;
  export let selectAccount;
  
  const dispatch = createEventDispatcher();
  
  // Standard folders that should be shown directly
  const standardFolders = ['INBOX', 'ARCHIVE', 'DRAFTS', 'SENT', 'DELETED ITEMS', 'TRASH', 'JUNK', 'SPAM'];
  
  // Separate standard and other folders
  $: standardFoldersList = folders.filter(folder => 
    standardFolders.some(std => folder.toUpperCase().includes(std))
  );
  
  $: otherFolders = folders.filter(folder => 
    !standardFolders.some(std => folder.toUpperCase().includes(std))
  );
  
  let showMoreFolders = false;
  
  function handleAccountClick(account) {
    selectAccount(account);
  }
  
  function handleFolderClick(folder) {
    selectFolder(folder);
  }
  
  function toggleMoreFolders() {
    showMoreFolders = !showMoreFolders;
  }
</script>

<aside class="sidebar">
  
  <div class="accounts-section">
    {#each accounts as account}
      <div class="account-item" class:selected={selectedAccount && selectedAccount.id === account.id}>
        <button 
          class="account-button"
          on:click={() => handleAccountClick(account)}
          on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleAccountClick(account)}
        >
          <span class="account-name">{account.name}</span>
          <span class="account-email">{account.username}</span>
        </button>
        
        {#if selectedAccount && selectedAccount.id === account.id}
          <div class="folders-section">
            {#each standardFoldersList as folder}
              <button 
                class="folder-item"
                class:selected={folder === selectedFolder}
                on:click={() => handleFolderClick(folder)}
                on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleFolderClick(folder)}
              >
                <span class="folder-icon">
                  {#if folder.toUpperCase().includes('INBOX')}📥
                  {:else if folder.toUpperCase().includes('SENT')}📤
                  {:else if folder.toUpperCase().includes('DRAFT')}📝
                  {:else if folder.toUpperCase().includes('ARCHIVE')}📁
                  {:else if folder.toUpperCase().includes('TRASH') || folder.toUpperCase().includes('DELETED')}🗑️
                  {:else if folder.toUpperCase().includes('JUNK') || folder.toUpperCase().includes('SPAM')}🚫
                  {:else}📂
                  {/if}
                </span>
                <span class="folder-name">{folder}</span>
              </button>
            {/each}
            
            {#if otherFolders.length > 0}
              <div class="more-section">
                <button 
                  class="more-toggle"
                  on:click={toggleMoreFolders}
                  on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleMoreFolders()}
                >
                  <span class="more-icon">{showMoreFolders ? '▼' : '▶'}</span>
                  <span class="more-text">More</span>
                </button>
                
                {#if showMoreFolders}
                  <div class="other-folders">
                    {#each otherFolders as folder}
                      <button 
                        class="folder-item other-folder"
                        class:selected={folder === selectedFolder}
                        on:click={() => handleFolderClick(folder)}
                        on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleFolderClick(folder)}
                      >
                        <span class="folder-icon">📂</span>
                        <span class="folder-name">{folder}</span>
                      </button>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  </div>
</aside>

<style>
  .sidebar {
    width: 280px;
    background: #f3f6f8;
    border-right: 1px solid #e0e0e0;
    padding: 0;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    flex-shrink: 0; /* Prevent shrinking */
  }
  
  .sidebar-header {
    padding: 1.5rem 1rem 1rem 1.5rem;
    border-bottom: 1px solid #e0e0e0;
    background: #f8f9fa;
  }
  
  .sidebar-header h2 {
    font-size: 1.1rem;
    margin: 0;
    color: #444;
    font-weight: 600;
  }
  
  .accounts-section {
    flex: 1;
    padding: 0.5rem 0;
  }
  
  .account-item {
    margin-bottom: 0.5rem;
  }
  
  .account-item.selected {
    background: #e6f0fa;
  }
  
  .account-button {
    width: 100%;
    padding: 0.8rem 1.5rem;
    border: none;
    background: none;
    text-align: left;
    cursor: pointer;
    transition: background 0.13s;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.2rem;
  }
  
  .account-button:hover {
    background: #e6f0fa;
  }
  
  .account-name {
    font-weight: 600;
    color: #333;
    font-size: 0.95rem;
  }
  
  .account-email {
    font-size: 0.8rem;
    color: #666;
    font-weight: 400;
  }
  
  .folders-section {
    padding-left: 1rem;
    margin-top: 0.5rem;
  }
  
  .folder-item {
    width: 100%;
    padding: 0.5rem 1rem;
    border: none;
    background: none;
    text-align: left;
    cursor: pointer;
    transition: background 0.13s;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    color: #555;
  }
  
  .folder-item:hover {
    background: #e6f0fa;
    color: #1976d2;
  }
  
  .folder-item.selected {
    background: #d1e7ff;
    color: #1976d2;
    font-weight: 500;
  }
  
  .folder-icon {
    font-size: 1rem;
    flex-shrink: 0;
  }
  
  .folder-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  .more-section {
    margin-top: 0.5rem;
  }
  
  .more-toggle {
    width: 100%;
    padding: 0.5rem 1rem;
    border: none;
    background: none;
    text-align: left;
    cursor: pointer;
    transition: background 0.13s;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    color: #666;
  }
  
  .more-toggle:hover {
    background: #e6f0fa;
    color: #1976d2;
  }
  
  .more-icon {
    font-size: 0.8rem;
    flex-shrink: 0;
  }
  
  .more-text {
    flex: 1;
  }
  
  .other-folders {
    padding-left: 1rem;
  }
  
  .other-folder {
    font-size: 0.85rem;
    color: #666;
  }
  
  .other-folder:hover {
    color: #1976d2;
  }
  
  .other-folder.selected {
    color: #1976d2;
    font-weight: 500;
  }
</style>