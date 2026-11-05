<script lang="ts">
  let {
    running,
    canStart,
    settingsOpen,
    ontoggle,
    onsettings,
  }: {
    running: boolean
    canStart: boolean
    settingsOpen: boolean
    ontoggle: () => void
    onsettings: () => void
  } = $props()

  const letters = [
    { ch: 'l', color: '#ff4fa3' },
    { ch: 'u', color: '#ff8a2b' },
    { ch: 'm', color: '#facc15' },
    { ch: 'a', color: '#4ade80' },
  ]
</script>

<header class="toolbar">
  <h1 aria-label="luma">
    {#each letters as l, i (i)}<span style="color: {l.color}">{l.ch}</span>{/each}
  </h1>

  <div class="right">
    <button class="power" class:stop={running} onclick={ontoggle} disabled={!running && !canStart}>
      {running ? 'Stop show' : 'Start show'}
    </button>
    <button class="icon" class:active={settingsOpen} onclick={onsettings} aria-label="Settings" aria-pressed={settingsOpen} title="Settings">
      ⚙
    </button>
  </div>
</header>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    height: 48px;
    padding: 0 14px;
    border-bottom: 1px solid var(--line);
    background: var(--surface);
  }
  h1 {
    margin: 0;
    font-size: 16px;
    font-weight: 800;
    letter-spacing: 0.06em;
  }

  .right {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .icon {
    width: 30px;
    height: 30px;
    padding: 0;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: transparent;
    color: var(--muted);
    font-size: 15px;
  }
  .icon:hover { color: var(--text); }
  .icon.active {
    background: var(--raised);
    color: var(--text);
  }
  .power {
    min-width: 110px;
    height: 30px;
    padding: 0 14px;
    border: none;
    border-radius: 6px;
    background: #2f9e5c;
    color: #fff;
    font-weight: 600;
  }
  .power:hover:not(:disabled) { background: #36b06a; }
  .power.stop { background: #c8434f; }
  .power.stop:hover:not(:disabled) { background: #d95460; }
  .power:disabled {
    background: var(--raised);
    color: var(--muted);
    cursor: default;
  }
</style>
