<script lang="ts">
  import { toasts } from '$lib/stores/toast';

  let toastList: import('$lib/stores/toast').Toast[] = [];
  toasts.subscribe((v) => (toastList = v));
</script>

<div class="toast-container">
  {#each toastList as toast (toast.id)}
    <div class="toast toast-{toast.type}">{toast.message}</div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 16px;
    right: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 9999;
    pointer-events: none;
  }
  .toast {
    padding: 10px 16px;
    border-radius: 6px;
    font-size: 13px;
    color: #fff;
    max-width: 360px;
    pointer-events: auto;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    animation: toast-in 0.2s ease-out;
  }
  .toast-error {
    background: #dc2626;
  }
  .toast-success {
    background: #16a34a;
  }
  .toast-info {
    background: var(--accent, #6366f1);
  }
  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
