<script>
  export let message = '';
  export let type = 'success'; // 'success' | 'error' | 'info'
  export let duration = 3000;
  export let show = false;
  
  let timeout;
  
  $: if (show && message) {
    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(() => {
      show = false;
    }, duration);
  }
</script>

{#if show && message}
  <div class="notification notification-{type}" role="alert">
    <span class="icon">
      {#if type === 'success'}✓{/if}
      {#if type === 'error'}✕{/if}
      {#if type === 'info'}ℹ{/if}
    </span>
    <span class="message">{message}</span>
  </div>
{/if}

<style>
  .notification {
    position: fixed;
    top: 16px;
    right: 16px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    color: white;
    z-index: 1000;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
    animation: slideIn 0.2s ease-out;
    max-width: 300px;
  }
  
  .notification-success {
    background: #059669;
  }
  
  .notification-error {
    background: #dc2626;
  }
  
  .notification-info {
    background: #2563eb;
  }
  
  .icon {
    font-size: 16px;
    font-weight: bold;
  }
  
  .message {
    flex: 1;
    line-height: 1.4;
  }
  
  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
</style> 