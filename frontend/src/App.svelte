<script lang="ts">
  import { onMount } from 'svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import HintBar from '$lib/components/HintBar.svelte';
  import MessageList from '$lib/components/MessageList.svelte';
  import ReadingPane from '$lib/components/ReadingPane.svelte';
  import ComposeView from '$lib/components/ComposeView.svelte';
  import { handleKeyDown, setBindings, registerHandler } from '$lib/keybindings/engine';
  import { defaultBindings } from '$lib/keybindings/bindings';

  let composing = false;
  let composeMode: 'compose' | 'reply' | 'reply-all' | 'forward' = 'compose';

  function openCompose(mode: 'compose' | 'reply' | 'reply-all' | 'forward'): void {
    composeMode = mode;
    composing = true;
  }

  onMount(() => {
    setBindings(defaultBindings);
    window.addEventListener('keydown', handleKeyDown);

    registerHandler('compose', () => openCompose('compose'));
    registerHandler('reply', () => openCompose('reply'));
    registerHandler('reply-all', () => openCompose('reply-all'));
    registerHandler('forward', () => openCompose('forward'));

    return () => window.removeEventListener('keydown', handleKeyDown);
  });
</script>

<div id="app">
  <StatusBar />
  <main class="content">
    <div class="message-list-pane">
      <MessageList />
    </div>
    <div class="reading-pane">
      {#if composing}
        <ComposeView replyMode={composeMode} on:close={() => (composing = false)} />
      {:else}
        <ReadingPane />
      {/if}
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
