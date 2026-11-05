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

<div class="modes" role="group" aria-label="Pattern">
  {#each modes as m (m.name)}
    <button
      class="mode"
      class:active={mode === m.name}
      style="--c: {m.color}"
      aria-pressed={mode === m.name}
      onclick={() => onpick(m.name)}
    >
      <span class="dot" aria-hidden="true"></span>
      {m.label}
    </button>
  {/each}
</div>

<style>
  .modes {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .mode {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 16px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--surface);
    font-size: 14px;
    font-weight: 500;
  }
  .mode:hover { background: var(--raised); }
  .mode.active {
    border-color: var(--c);
    background: color-mix(in srgb, var(--c) 18%, var(--surface));
    font-weight: 600;
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--c);
    opacity: 0.55;
  }
  .mode.active .dot {
    opacity: 1;
    box-shadow: 0 0 8px var(--c);
  }
</style>
