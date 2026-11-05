<script lang="ts">
  import { swatch, type NamedEffect, type Scene } from '../api'

  let {
    effects,
    scenes,
    activeEffect,
    activeScene,
    onplayeffect,
    onplayscene,
  }: {
    effects: NamedEffect[]
    scenes: Scene[]
    activeEffect: string | null
    activeScene: string | null
    onplayeffect: (id: string) => void
    onplayscene: (id: string) => void
  } = $props()

  let query = $state('')
  let needle = $derived(query.trim().toLowerCase())
  let shownEffects = $derived(effects.filter((e) => e.name.toLowerCase().includes(needle)))
  let shownScenes = $derived(scenes.filter((s) => s.name.toLowerCase().includes(needle)))
</script>

<aside class="picker" aria-label="Scenes">
  <h2 class="title">Effects</h2>
  <input type="search" placeholder="Search effects and scenes" aria-label="Search effects and scenes" bind:value={query} />
  <div class="list" role="group" aria-label="Effects">
    {#each shownEffects as e (e.id)}
      <button
        class="mode"
        class:active={!activeScene && activeEffect === e.id}
        style="--c: {swatch(e.effect)}"
        aria-pressed={!activeScene && activeEffect === e.id}
        onclick={() => onplayeffect(e.id)}
      >
        <span class="dot" aria-hidden="true"></span>
        {e.name}
      </button>
    {/each}
  </div>

  {#if shownScenes.length > 0}
    <h2 class="title">Scenes</h2>
    <div class="list" role="group" aria-label="Scenes">
      {#each shownScenes as s (s.id)}
        <button
          class="mode"
          class:active={activeScene === s.id}
          style="--c: #e879f9"
          aria-pressed={activeScene === s.id}
          onclick={() => onplayscene(s.id)}
        >
          <span class="dot" aria-hidden="true"></span>
          {s.name}
        </button>
      {/each}
    </div>
  {/if}

  {#if shownEffects.length === 0 && shownScenes.length === 0}
    <p class="none">Nothing matches “{query}”.</p>
  {/if}
</aside>

<style>
  .picker {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-height: 0;
    padding: 14px 10px;
    border-left: 1px solid var(--line);
    background: var(--surface);
    overflow: hidden;
  }
  .title {
    margin: 0 2px;
    font-size: 14px;
  }
  input[type='search'] {
    padding: 6px 9px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font: inherit;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow-y: auto;
    min-height: 0;
  }
  .mode {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 12px;
    border: 1px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: 14px;
    font-weight: 500;
    text-align: left;
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
    flex: none;
    border-radius: 50%;
    background: var(--c);
    opacity: 0.55;
  }
  .mode.active .dot {
    opacity: 1;
    box-shadow: 0 0 8px var(--c);
  }
  .none {
    margin: 4px 2px;
    color: var(--muted);
    font-size: 12px;
  }
</style>
