<script>
  import { createEventDispatcher } from 'svelte';
  export let emails = [];
  export let openEmail;
  export let width = 340; // Default width
  export let onWidthChange;
  export let onLoadMore;

  const dispatch = createEventDispatcher();

  let isResizing = false;
  let startX = 0;
  let startWidth = 0;

  function handleScroll(e) {
    const el = e.target;
    if (el.scrollTop + el.clientHeight >= el.scrollHeight - 100) {
      if (onLoadMore) onLoadMore();
    }
  }

  // Debug: Log email data to see attachment counts
  $: if (emails && emails.length > 0) {
    console.log('Full email objects:', emails);
    console.log('Emails with attachment counts:', emails.map(e => ({
      subject: e.subject,
      attachmentCount: e.attachmentCount,
      hasAttachmentCount: 'attachmentCount' in e,
      type: typeof e.attachmentCount
    })));
    
    // Log emails that show attachments
    const emailsWithAttachments = emails.filter(e => e.attachmentCount > 0);
    if (emailsWithAttachments.length > 0) {
      console.log('Emails showing attachment icons:', emailsWithAttachments.map(e => ({
        subject: e.subject,
        attachmentCount: e.attachmentCount
      })));
    }
  }

  function startResize(e) {
    isResizing = true;
    startX = e.clientX;
    startWidth = width;
    
    document.addEventListener('mousemove', handleResize);
    document.addEventListener('mouseup', stopResize);
    
    // Prevent text selection during resize
    e.preventDefault();
  }

  function handleResize(e) {
    if (!isResizing) return;
    
    const deltaX = e.clientX - startX;
    const newWidth = Math.max(250, Math.min(600, startWidth + deltaX));
    
    if (newWidth !== width) {
      width = newWidth;
      if (onWidthChange) {
        onWidthChange(newWidth);
      }
    }
  }

  function stopResize() {
    isResizing = false;
    document.removeEventListener('mousemove', handleResize);
    document.removeEventListener('mouseup', stopResize);
  }

  // Generate avatar initials from email address
  function getAvatarInitials(email) {
    const from = email.from || '';
    const name = from.split('@')[0];
    return name.substring(0, 2).toUpperCase();
  }

  // Generate avatar color based on email address
  function getAvatarColor(email) {
    const from = email.from || '';
    const hash = from.split('').reduce((a, b) => {
      a = ((a << 5) - a) + b.charCodeAt(0);
      return a & a;
    }, 0);
    const colors = [
      '#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', 
      '#FFEAA7', '#DDA0DD', '#98D8C8', '#F7DC6F',
      '#BB8FCE', '#85C1E9', '#F8C471', '#82E0AA'
    ];
    return colors[Math.abs(hash) % colors.length];
  }

  // Truncate text for snippet
  function truncateText(text, maxLength = 80) {
    if (!text) return '';
    return text.length > maxLength ? text.substring(0, maxLength) + '...' : text;
  }
</script>

<div class="email-list-container" style="width: {width}px;" on:scroll={handleScroll}>
  <ul class="email-list">
    {#each emails as email}
      <li class="email-item-container">
        <button
          class="email-item {!email.read ? 'unread' : ''}"
          on:click={() => {
            openEmail(email);
          }}
          on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && openEmail(email)}
        >
          <div class="email-avatar">
            <div 
              class="avatar-circle"
              style="background-color: {getAvatarColor(email)}"
            >
              {getAvatarInitials(email)}
            </div>
          </div>
          
          <div class="email-content">
            <div class="email-header">
              <span class="email-author">{email.from}</span>
              <span class="email-date">{email.date}</span>
            </div>
            
            <div class="email-subject-line">
              <span class="email-subject {!email.read ? 'unread' : ''}">
                {email.subject}
              </span>
              {#if email.attachmentCount > 0}
                <span class="attachment-icon" title="{email.attachmentCount} attachment(s)">📎</span>
              {/if}
            </div>
            
            <div class="email-snippet">
              {#if email.snippet && email.snippet !== 'No preview available'}
                {truncateText(email.snippet, 80)}
              {:else}
                <span class="no-snippet">Click to view email content</span>
              {/if}
            </div>
          </div>
        </button>
      </li>
    {/each}
  </ul>
  
  <!-- Resize handle -->
  <div 
    class="resize-handle"
    on:mousedown={startResize}
    title="Drag to resize email list"
  ></div>
</div>

<style>
  .email-list-container {
    position: relative;
    border-right: 1px solid #e0e0e0;
    overflow-y: auto;
    background: #fafdff;
    display: flex;
    flex-direction: column;
  }
  
  .email-list {
    list-style: none;
    margin: 0;
    padding: 0;
    width: 100%;
    flex: 1;
  }
  
  .email-item-container {
    list-style: none;
  }
  
  .email-item {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 1rem 1.25rem;
    border: none;
    background: none;
    width: 100%;
    text-align: left;
    cursor: pointer;
    transition: background 0.12s;
    border-bottom: 1px solid #f0f0f0;
  }
  
  .email-item:last-child {
    border-bottom: none;
  }
  
  .email-item:hover {
    background: #f5faff;
  }
  
  .email-avatar {
    flex-shrink: 0;
  }
  
  .avatar-circle {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-weight: 600;
    font-size: 0.9rem;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  }
  
  .email-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .email-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
  }
  
  .email-author {
    font-weight: 600;
    color: #2a2a2a;
    font-size: 0.9rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  .email-date {
    font-size: 0.75rem;
    color: #666;
    flex-shrink: 0;
  }
  
  .email-subject-line {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .email-subject {
    font-weight: 600;
    color: #222;
    font-size: 0.95rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }
  
  .email-subject.unread {
    font-weight: 700;
    color: #000;
  }
  
  .attachment-icon {
    font-size: 0.8rem;
    color: #666;
    flex-shrink: 0;
    opacity: 0.8;
  }
  
  .email-snippet {
    color: #666;
    font-size: 0.85rem;
    line-height: 1.3;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    max-height: 2.6em;
    line-clamp: 2;
  }

  .no-snippet {
    color: #999;
    font-style: italic;
    font-size: 0.8rem;
  }
  
  .email-item.unread {
    background: #f0f8ff;
    border-left: 4px solid #0078d4;
  }
  
  .email-item.unread:hover {
    background: #e6f3ff;
  }
  
  .email-item.unread .email-author {
    font-weight: 700;
    color: #000;
  }
  
  .resize-handle {
    position: absolute;
    right: -3px;
    top: 0;
    bottom: 0;
    width: 6px;
    cursor: col-resize;
    background: transparent;
    transition: background 0.12s;
    z-index: 10;
  }
  
  .resize-handle:hover {
    background: rgba(0, 120, 212, 0.2);
  }
  
  .resize-handle:active {
    background: rgba(0, 120, 212, 0.4);
  }
</style>