<script lang="ts">
  import { mode } from '$lib/stores/ui';
  import { activeAccount, activeFolder, unreadCount } from '$lib/stores/accounts';

  const modeColors: Record<string, string> = {
    NORMAL: 'var(--text-dim)',
    INSERT: '#22c55e',
    VISUAL: '#f59e0b',
    COMMAND: 'var(--accent)',
  };

  let currentMode = $state('NORMAL');
  let account = $state<{ name: string; email: string } | null>(null);
  let folder = $state('INBOX');
  let unread = $state(0);

  mode.subscribe((v) => (currentMode = v));
  activeAccount.subscribe((v) => (account = v));
  activeFolder.subscribe((v) => (folder = v));
  unreadCount.subscribe((v) => (unread = v));
</script>

<div class="status-bar">
  <span class="left">
    {account ? account.name : 'no account'}:{folder}{#if unread > 0}({unread}){/if}
  </span>
  <span class="center" style="color: {modeColors[currentMode]}">
    {currentMode}
  </span>
  <span class="right">? help</span>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 28px;
    padding: 0 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: 12px;
    flex-shrink: 0;
  }
  .left {
    color: var(--text-secondary);
  }
  .center {
    font-weight: 600;
  }
  .right {
    color: var(--text-dim);
  }
</style>
