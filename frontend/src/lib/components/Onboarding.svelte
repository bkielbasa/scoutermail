<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  function dismiss() {
    dispatch('done');
  }

  function handleKey(e: KeyboardEvent) {
    dismiss();
  }

  onMount(() => {
    window.addEventListener('keydown', handleKey);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKey);
  });
</script>

<div class="onboarding-overlay" role="dialog" aria-label="Welcome to ScouterMail">
  <div class="onboarding-content">
    <h1>Welcome to <span class="accent">ScouterMail</span></h1>
    <p class="subtitle">A vim-powered email client built for speed.</p>

    <div class="keybindings">
      <h2>Key concepts</h2>
      <div class="binding"><kbd>j</kbd> / <kbd>k</kbd><span>Navigate messages</span></div>
      <div class="binding"><kbd>Enter</kbd><span>Open message</span></div>
      <div class="binding"><kbd>r</kbd><span>Reply</span></div>
      <div class="binding"><kbd>c</kbd><span>Compose</span></div>
      <div class="binding"><kbd>/</kbd><span>Search</span></div>
      <div class="binding"><kbd>:</kbd><span>Command mode</span></div>
      <div class="binding"><kbd>?</kbd><span>Full help</span></div>
    </div>

    <p class="continue">Press any key to continue...</p>
  </div>
</div>

<style>
  .onboarding-overlay {
    position: fixed;
    inset: 0;
    z-index: 9999;
    background: rgba(0, 0, 0, 0.92);
    display: flex;
    align-items: center;
    justify-content: center;
    animation: fadeIn 0.3s ease;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .onboarding-content {
    text-align: center;
    max-width: 500px;
    padding: 3rem;
  }

  h1 {
    font-size: 2rem;
    font-weight: 700;
    color: #e0e0e0;
    margin-bottom: 0.5rem;
  }

  .accent {
    color: var(--accent, #7aa2f7);
  }

  .subtitle {
    color: #888;
    font-size: 1.1rem;
    margin-bottom: 2.5rem;
  }

  h2 {
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #666;
    margin-bottom: 1rem;
  }

  .keybindings {
    text-align: left;
    margin-bottom: 2.5rem;
  }

  .binding {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.4rem 0;
    color: #ccc;
    font-size: 0.95rem;
  }

  kbd {
    display: inline-block;
    min-width: 2rem;
    padding: 0.15rem 0.5rem;
    background: #2a2a3e;
    border: 1px solid #444;
    border-radius: 4px;
    color: var(--accent, #7aa2f7);
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 0.85rem;
    text-align: center;
  }

  .continue {
    color: #555;
    font-size: 0.9rem;
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.5; }
    50% { opacity: 1; }
  }
</style>
