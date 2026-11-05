<script lang="ts">
  let { mode, onpick }: { mode?: string; onpick: (name: string) => void } = $props()

  const modes = [
    { name: 'microphone', label: 'Microphone', color: '#ff4fa3' },
    { name: 'screen', label: 'Screen', color: '#3d8bff' },
    { name: 'rainbow', label: 'Rainbow', color: '#a855f7' },
    { name: 'chase', label: 'Chase', color: '#ff8a2b' },
    { name: 'breathe', label: 'Breathe', color: '#14b8a6' },
    { name: 'solid', label: 'Solid', color: '#facc15' },
  ]
</script>

<section>
  <h2 class="section-title">Mode</h2>
  <div class="list" role="group" aria-label="Mode">
    {#each modes as m (m.name)}
      <button
        class="item"
        class:active={mode === m.name}
        style="--c: {m.color}"
        aria-pressed={mode === m.name}
        onclick={() => onpick(m.name)}
      >
        <span class="swatch" aria-hidden="true"></span>
        {m.label}
      </button>
    {/each}
  </div>
</section>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 7px 10px;
    border: none;
    border-radius: 6px;
    background: transparent;
    text-align: left;
    font-size: 13px;
  }
  .item:hover { background: rgba(255, 255, 255, 0.05); }
  .item.active {
    background: var(--raised);
    font-weight: 600;
  }
  .swatch {
    width: 10px;
    height: 10px;
    border-radius: 3px;
    background: var(--c);
    flex: none;
    opacity: 0.6;
  }
  .item.active .swatch {
    opacity: 1;
    box-shadow: 0 0 8px var(--c);
  }
</style>
