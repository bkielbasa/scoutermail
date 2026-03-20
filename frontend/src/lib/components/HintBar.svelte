<script lang="ts">
  import { mode, commandInput } from '$lib/stores/ui';

  const hints: Record<string, string> = {
    NORMAL: 'j/k navigate · enter open · r reply · a archive · d delete · / search · ? help',
    VISUAL: 'j/k extend · a archive · d delete · esc cancel',
    INSERT: 'esc normal · ctrl+enter send',
  };

  let currentMode = $state('NORMAL');
  let cmdInput = $state('');

  mode.subscribe((v) => (currentMode = v));
  commandInput.subscribe((v) => (cmdInput = v));
</script>

<div class="hint-bar">
  {#if currentMode === 'COMMAND'}
    <span class="command-text">:{cmdInput}</span>
  {:else}
    <span class="hints">{hints[currentMode] ?? ''}</span>
  {/if}
</div>

<style>
  .hint-bar {
    display: flex;
    align-items: center;
    height: 28px;
    padding: 0 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-dim);
    flex-shrink: 0;
  }
  .command-text {
    color: var(--accent);
  }
</style>
