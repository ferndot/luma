<script lang="ts">
  import { iconEffect, swatch, type NamedEffect } from '../api'
  import EffectIcon from './EffectIcon.svelte'
  import PlayIcon from './PlayIcon.svelte'
  import Spinner from './Spinner.svelte'

  let {
    effects,
    activeEffect,
    running,
    starting,
    onplay,
    onstop,
  }: {
    effects: NamedEffect[]
    activeEffect: string | null
    running: boolean
    starting: boolean
    onplay: (id: string) => void
    onstop: () => void
  } = $props()

  let query = $state('')
  let needle = $derived(query.trim().toLowerCase())
  let shown = $derived(effects.filter((e) => e.name.toLowerCase().includes(needle)))
</script>

<aside class="picker" aria-label="Effects">
  <h2 class="title">Effects</h2>
  <input type="search" placeholder="Search effects" aria-label="Search effects" bind:value={query} />
  <div class="list" role="group" aria-label="Effects">
    {#each shown as e (e.id)}
      {@const current = activeEffect === e.id}
      <div class="mode" class:active={current} style="--c: {swatch(e.effect)}">
        <button class="pick" onclick={() => onplay(e.id)} aria-pressed={current}>
          <EffectIcon spec={iconEffect(e.effect, effects)} width={56} height={34} />
          <span class="name">{e.name}</span>
        </button>
        {#if current && starting}
          <span class="control"><Spinner size={14} label="Starting {e.name}" /></span>
        {:else if current && running}
          <button class="control" onclick={onstop} aria-label="Stop {e.name}"><PlayIcon kind="stop" size={14} /></button>
        {:else}
          <button class="control" onclick={() => onplay(e.id)} aria-label="Play {e.name}"><PlayIcon kind="play" size={14} /></button>
        {/if}
      </div>
    {/each}
  </div>
  {#if shown.length === 0}
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
    gap: 6px;
    padding: 4px 6px 4px 4px;
    border: 1px solid transparent;
    border-radius: 8px;
    color: var(--text);
  }
  .mode:hover { background: var(--raised); }
  .mode.active {
    border-color: var(--c);
    background: color-mix(in srgb, var(--c) 18%, var(--surface));
  }
  .pick {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px;
    border: none;
    background: transparent;
    color: inherit;
    font: inherit;
    font-size: 14px;
    font-weight: 500;
    text-align: left;
    cursor: pointer;
  }
  .mode.active .pick { font-weight: 600; }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .control {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    flex: none;
    border: 1px solid var(--line);
    border-radius: 50%;
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
  }
  span.control { cursor: default; border-color: transparent; background: transparent; }
  .none {
    margin: 4px 2px;
    color: var(--muted);
    font-size: 12px;
  }
</style>
