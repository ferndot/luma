<script lang="ts">
  let {
    subnets,
    onadd,
    onremove,
  }: {
    subnets: string[]
    onadd: (subnet: string) => Promise<boolean>
    onremove: (subnet: string) => void
  } = $props()

  let value = $state('')

  // Clear the field only when the network was added.
  async function submit(e: SubmitEvent) {
    e.preventDefault()
    if (await onadd(value)) value = ''
  }
</script>

<section>
  <h2 class="section-title">Networks</h2>
  <p class="muted">Also scan other networks, such as an IoT network this computer is not on.</p>

  <form class="add" onsubmit={submit}>
    <input bind:value placeholder="192.168.10" aria-label="Network to add" />
    <button type="submit" disabled={!value.trim()}>Add</button>
  </form>

  {#if subnets.length > 0}
    <ul class="list">
      {#each subnets as s (s)}
        <li>
          <code>{s}.0/24</code>
          <button class="remove" aria-label="Remove {s}.0/24" onclick={() => onremove(s)}>Remove</button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .add {
    display: flex;
    gap: 6px;
    margin: 10px 0 0;
  }
  .add input {
    flex: 1;
    min-width: 0;
    padding: 6px 8px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
  }
  .add button,
  .remove {
    padding: 6px 10px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--raised);
  }
  .add button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .list {
    list-style: none;
    margin: 10px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .list li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 4px 4px 4px 10px;
    border-radius: 6px;
    background: var(--bg);
  }
  .remove {
    padding: 2px 8px;
    font-size: 12px;
    color: var(--muted);
    background: transparent;
    border-color: transparent;
  }
  .remove:hover { color: var(--text); }
</style>
