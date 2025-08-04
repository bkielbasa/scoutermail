<script>
  import { createEventDispatcher } from 'svelte';
  export let emails = [];
  export let openEmail;
  export let currentPage = 1;
  export let totalPages = 1;
  export let pageSize = 20;

  const dispatch = createEventDispatcher();

  function nextPage() {
    if (currentPage < totalPages) {
      dispatch('nextPage');
    }
  }

  function prevPage() {
    if (currentPage > 1) {
      dispatch('prevPage');
    }
  }

  // Reset to first page when emails or folder changes
  $: if (emails && currentPage > totalPages) {
    currentPage = 1;
  }
</script>

<ul class="email-list">
  {#each emails as email}
    <li class="email-item-container">
      <button
        class="email-item"
        on:click={() => {
          console.log('Email clicked:', email);
          openEmail(email);
        }}
        on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && openEmail(email)}
      >
        <span class="email-from">{email.from}</span>
        <span class="email-subject">{email.subject}</span>
        <span class="email-date">{email.date}</span>
      </button>
    </li>
  {/each}
</ul>

<style>
  .email-list {
    list-style: none;
    margin: 0;
    padding: 0;
    width: 100%;
  }
  .email-item-container {
    list-style: none;
  }
  .email-item {
    display: grid;
    grid-template-columns: 1.5fr 2fr 1fr;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1.5rem;
    border: none;
    background: none;
    width: 100%;
    text-align: left;
    font-size: 0.97rem;
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
  .email-from {
    font-weight: 500;
    color: #2a2a2a;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .email-subject {
    font-weight: 600;
    color: #222;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .email-date {
    color: #888;
    font-size: 0.93em;
    text-align: right;
    white-space: nowrap;
  }
</style>