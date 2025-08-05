<script>
  import { GetEmailContent, SaveAttachment, MarkEmailAsRead, ForceRefreshEmailContent } from '../wailsjs/go/main/App.js';
  import { onMount } from 'svelte';
  import Notification from './Notification.svelte';
  
  export let email;
  export let onBack;
  export let onEmailRead;
  
  let emailContent = null;
  let loading = false;
  let error = '';
  let notification = { show: false, message: '', type: 'success' };
  let retryCount = 0;
  const maxRetries = 2;

  async function loadEmailContent() {
    if (!email || !email.id) {
      return;
    }
    
    loading = true;
    error = '';
    
    try {
      emailContent = await GetEmailContent(email.id);
      
      // Check if we got meaningful content
      if (!emailContent || (emailContent.textBody === '' && emailContent.htmlBody === '')) {
        if (retryCount < maxRetries) {
          retryCount++;
          console.log(`No content found, retrying... (${retryCount}/${maxRetries})`);
          // Force refresh the content
          await ForceRefreshEmailContent(email.id);
          emailContent = await GetEmailContent(email.id);
        } else {
          error = 'No content available. The email might be empty or corrupted.';
        }
      } else {
        retryCount = 0; // Reset retry count on success
      }
      
      // Mark email as read if it's not already read
      if (!email.read) {
        try {
          await MarkEmailAsRead(email.id);
          // Notify parent component that email was marked as read
          if (onEmailRead) {
            onEmailRead(email.id);
          }
        } catch (e) {
          console.error('Error marking email as read:', e);
        }
      }
    } catch (e) {
      error = 'Failed to load email content';
      console.error('Error loading email content:', e);
    } finally {
      loading = false;
    }
  }

  async function retryLoadContent() {
    retryCount = 0;
    await loadEmailContent();
  }

  function showNotification(message, type = 'success') {
    notification = { show: true, message, type };
  }

  async function downloadAttachment(filename) {
    if (!email || !email.id) return;
    
    try {
      // Use the new SaveAttachment function
      const filePath = await SaveAttachment(email.id, filename);
      
      if (filePath) {
        // Extract just the filename from the full path for display
        const fileName = filePath.split('/').pop();
        showNotification(`"${fileName}" downloaded`, 'success');
      } else {
        showNotification('Failed to save attachment', 'error');
      }
    } catch (e) {
      console.error('Error downloading attachment:', e);
      showNotification(`Download failed: ${e.message}`, 'error');
    }
  }
  
  $: if (email && email.id) {
    loadEmailContent();
  }
</script>

<div class="email-view">
  {#if email}
    <div class="email-header">
      <h2>{email.subject || '(No subject)'}</h2>
      <div class="email-meta">
        <div class="meta-row">
          <span class="meta-label">From:</span>
          <span class="meta-value">{email.from || '(Unknown sender)'}</span>
        </div>
        <div class="meta-row">
          <span class="meta-label">Date:</span>
          <span class="meta-value">{email.date || ''}</span>
        </div>
        {#if emailContent && emailContent.headers}
          {#if emailContent.headers['to']}
            <div class="meta-row">
              <span class="meta-label">To:</span>
              <span class="meta-value">{emailContent.headers['to']}</span>
            </div>
          {/if}
          {#if emailContent.headers['cc']}
            <div class="meta-row">
              <span class="meta-label">Cc:</span>
              <span class="meta-value">{emailContent.headers['cc']}</span>
            </div>
          {/if}
          {#if emailContent.headers['reply-to']}
            <div class="meta-row">
              <span class="meta-label">Reply-To:</span>
              <span class="meta-value">{emailContent.headers['reply-to']}</span>
            </div>
          {/if}
        {/if}
      </div>
    </div>
    
    {#if loading}
      <div class="loading">Loading content...</div>
    {:else if error}
      <div class="error">
        <div>{error}</div>
        <button class="retry-btn" on:click={retryLoadContent}>
          🔄 Retry Loading
        </button>
      </div>
    {:else if emailContent}
      <!-- Email Body - Show HTML if available, otherwise show text -->
      {#if emailContent.htmlBody}
        <div class="email-body">
          <div class="html-content">{@html emailContent.htmlBody}</div>
        </div>
      {:else if emailContent.textBody}
        <div class="email-body">
          <div class="text-content">{emailContent.textBody}</div>
        </div>
      {:else}
        <div class="no-content">No content available</div>
      {/if}
      
      <!-- Attachments -->
      {#if emailContent.attachments && emailContent.attachments.length > 0}
        <div class="email-attachments">
          <h3>Attachments ({emailContent.attachments.length})</h3>
          {#each emailContent.attachments as attachment}
            <div class="attachment">
              <div class="attachment-info">
                <span class="attachment-name">{attachment.filename}</span>
                <span class="attachment-type">{attachment.contentType}</span>
                <span class="attachment-size">{attachment.size} bytes</span>
              </div>
              <button 
                class="download-btn" 
                on:click={() => {
                  downloadAttachment(attachment.filename);
                }}
              >
                Download
              </button>
            </div>
          {/each}
        </div>
      {/if}
    {:else}
      <div class="no-content">No content available</div>
    {/if}
  {:else}
    <div>No email selected.</div>
  {/if}
</div>

<!-- Notification -->
<Notification 
  bind:show={notification.show}
  message={notification.message} 
  type={notification.type} 
/>

<style>
.email-view {
  color: #222;
  background: #fff;
  padding: 2rem;
  height: 100%;
  overflow-y: auto;
}

.email-header {
  margin-bottom: 2rem;
  padding-bottom: 1.5rem;
  border-bottom: 1px solid #e0e0e0;
}

.email-header h2 {
  color: #222;
  margin: 0 0 1rem 0;
  font-size: 1.5rem;
  font-weight: 600;
  line-height: 1.3;
}

.email-meta {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.meta-row {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  font-size: 0.9rem;
  line-height: 1.4;
}

.meta-label {
  font-weight: 500;
  color: #666;
  min-width: 60px;
  flex-shrink: 0;
}

.meta-value {
  color: #333;
  word-break: break-word;
}

.loading {
  color: #666;
  font-style: italic;
  text-align: center;
  padding: 2rem;
}

.error {
  color: #d9534f;
  padding: 1rem;
  text-align: center;
  background: #f8d7da;
  border: 1px solid #f5c6cb;
  border-radius: 4px;
  margin: 1rem 0;
}

.retry-btn {
  background: #0078d4;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  margin-top: 0.5rem;
  transition: background 0.12s;
}

.retry-btn:hover {
  background: #0056b3;
}

.no-content {
  color: #888;
  font-style: italic;
  text-align: center;
  padding: 2rem;
}

.email-body {
  margin: 1.5rem 0;
}

.text-content {
  line-height: 1.6;
  color: #333;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 14px;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.html-content {
  line-height: 1.6;
  color: #333;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 14px;
  max-width: 100%;
  overflow-wrap: break-word;
  word-wrap: break-word;
}

.html-content img {
  max-width: 100%;
  height: auto;
}

.html-content table {
  max-width: 100%;
  border-collapse: collapse;
}

.email-attachments {
  margin: 2rem 0;
  padding: 1.5rem;
  background: #f8f9fa;
  border-radius: 8px;
}

.email-attachments h3 {
  margin: 0 0 1rem 0;
  font-size: 1rem;
  color: #555;
  font-weight: 600;
}

.attachment {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.75rem;
  background: #fff;
  border-radius: 6px;
  margin-bottom: 0.75rem;
  border: 1px solid #e0e0e0;
}

.attachment-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  flex: 1;
}

.attachment-name {
  font-weight: 500;
  color: #1976d2;
  font-size: 0.9rem;
}

.attachment-type {
  color: #666;
  font-size: 0.8rem;
}

.attachment-size {
  color: #888;
  font-size: 0.75rem;
}

.download-btn {
  background: #007bff;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8rem;
  transition: background 0.12s;
  white-space: nowrap;
}

.download-btn:hover {
  background: #0056b3;
}
</style>