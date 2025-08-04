<script>
  export let folders = [];
  export let selectedFolder = "";
  export let selectFolder;
</script>

<aside class="sidebar">
  <h2>Folders</h2>
  <ul>
    {#each folders as folder}
      <li class:selected={folder === selectedFolder} on:click={() => selectFolder(folder)}>
        {folder}
      </li>
    {/each}
  </ul>
</aside>

<style>
  .sidebar {
    width: 220px;
    background: #f3f6f8;
    border-right: 1px solid #e0e0e0;
    padding: 1.5rem 1rem 1rem 1.5rem;
  }
  .sidebar h2 {
    font-size: 1.1rem;
    margin-bottom: 1rem;
    color: #444;
  }
  .sidebar ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .sidebar li {
    padding: 0.5rem 0.8rem;
    margin-bottom: 0.2rem;
    border-radius: 4px;
    cursor: pointer;
    color: #222;
    transition: background 0.13s;
  }
  .sidebar li.selected,
  .sidebar li:hover {
    background: #e6f0fa;
    color: #1976d2;
  }
</style>