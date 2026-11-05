<script lang="ts">
  import type { Group } from '../api'

  let {
    groups,
    scanning,
    oncreate,
    onopen,
    onrescan,
  }: {
    groups: Group[]
    scanning: boolean
    oncreate: (name: string) => Promise<boolean>
    onopen: (id: string) => void
    onrescan: () => void
  } = $props()

  let name = $state('')

  async function submit(e: SubmitEvent) {
    e.preventDefault()
    if (await oncreate(name)) name = ''
  }
</script>

<section class="page">
  <header class="head">
    <h1>Groups</h1>
    <button onclick={onrescan} disabled={scanning}>{scanning ? 'Finding lights…' : 'Find lights'}</button>
  </header>

  <form class="new" onsubmit={submit}>
    <input bind:value={name} placeholder="New group, e.g. Desk" aria-label="New group name" maxlength="40" />
    <button type="submit" disabled={!name.trim()}>New group</button>
  </form>

  {#if groups.length > 0}
    <ul class="cards">
      {#each groups as g (g.id)}
        <li>
          <button class="card" onclick={() => onopen(g.id)}>
            <span class="name">{g.name}</span>
            <span class="meta">{g.members.length} {g.members.length === 1 ? 'device' : 'devices'}</span>
          </button>
        </li>
      {/each}
    </ul>
  {:else}
    <div class="empty">
      <p>No groups yet.</p>
      <p class="muted">Make one for a room or a strip, add its lights, then start a scene on it.</p>
    </div>
  {/if}
</section>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-height: 0;
    overflow-y: auto;
    padding: 20px 24px;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  h1 {
    margin: 0;
    font-size: 20px;
  }
  .new {
    display: flex;
    gap: 8px;
    max-width: 420px;
  }
  .new input {
    flex: 1;
    min-width: 0;
    padding: 7px 9px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
  }
  .new button,
  .head button {
    padding: 6px 12px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--raised);
  }
  .new button:disabled,
  .head button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .cards {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 10px;
  }
  .card {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
    padding: 14px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--surface);
    text-align: left;
  }
  .card:hover { border-color: var(--accent); }
  .name {
    font-weight: 600;
    font-size: 15px;
  }
  .meta,
  .muted {
    color: var(--muted);
    font-size: 12px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 24px 0;
  }
  .empty p { margin: 0; }
</style>
