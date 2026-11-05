<script lang="ts">
  import type { Group } from '../api'

  export type Page = { kind: 'home' } | { kind: 'group'; id: string } | { kind: 'settings' }

  let {
    groups,
    page,
    onnav,
  }: {
    groups: Group[]
    page: Page
    onnav: (page: Page) => void
  } = $props()
</script>

<aside class="sidebar">
  <div class="brand" aria-label="luma">
    {#each [['l', '#ff4fa3'], ['u', '#ff8a2b'], ['m', '#facc15'], ['a', '#4ade80']] as [ch, color]}
      <span style="color: {color}">{ch}</span>
    {/each}
  </div>

  <nav class="nav">
    <button class="item" class:active={page.kind === 'home'} onclick={() => onnav({ kind: 'home' })}>
      All groups
    </button>

    {#if groups.length > 0}
      <h2 class="section-title">Groups</h2>
      {#each groups as g (g.id)}
        <button
          class="item"
          class:active={page.kind === 'group' && page.id === g.id}
          onclick={() => onnav({ kind: 'group', id: g.id })}
        >
          {g.name}
        </button>
      {/each}
    {/if}
  </nav>

  <div class="bottom">
    <button class="item" class:active={page.kind === 'settings'} onclick={() => onnav({ kind: 'settings' })}>
      Settings
    </button>
  </div>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-height: 0;
    padding: 14px 10px;
    border-right: 1px solid var(--line);
    background: var(--surface);
    overflow: hidden;
  }
  .brand {
    padding: 0 8px;
    font-size: 20px;
    font-weight: 800;
    letter-spacing: 0.06em;
    user-select: none;
  }
  .nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-height: 0;
    overflow-y: auto;
  }
  .nav .section-title {
    margin: 14px 8px 6px;
  }
  .bottom {
    margin-top: auto;
    border-top: 1px solid var(--line);
    padding-top: 10px;
  }
  .item {
    display: block;
    width: 100%;
    padding: 7px 10px;
    border: none;
    border-radius: 6px;
    background: transparent;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .item:hover { background: rgba(255, 255, 255, 0.05); }
  .item.active {
    background: var(--raised);
    font-weight: 600;
  }
</style>
