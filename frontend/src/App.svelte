<script lang="ts">
  import { onMount } from 'svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import HintBar from '$lib/components/HintBar.svelte';
  import { handleKeyDown, setBindings } from '$lib/keybindings/engine';
  import { defaultBindings } from '$lib/keybindings/bindings';

  onMount(() => {
    setBindings(defaultBindings);
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  });
</script>

<div id="app">
  <StatusBar />
  <main class="content">
    <div class="message-list-pane">
      <p style="color: var(--text-dim); padding: 12px;">Message list</p>
    </div>
    <div class="reading-pane">
      <p style="color: var(--text-dim); padding: 12px;">Reading pane</p>
    </div>
  </main>
  <HintBar />
</div>

<style>
  .content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .message-list-pane {
    width: 33%;
    min-width: 250px;
    max-width: 500px;
    border-right: 1px solid var(--border);
    overflow-y: auto;
  }
  .reading-pane {
    flex: 1;
    overflow-y: auto;
  }
</style>
