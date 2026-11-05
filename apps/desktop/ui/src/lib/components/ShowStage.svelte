<script lang="ts">
  import { watch } from '../preview'
  import type { Effect, Rgb } from '../api'
  import Spinner from './Spinner.svelte'

  let {
    preview,
    positions,
    running,
    shown,
    starting,
  }: {
    preview: Rgb[]
    positions: [number, number][]
    running: boolean
    // The effect the user picked: drawn at once, before the lights have started.
    // Named `shown`, not `effect`: a prop called effect would shadow the $effect rune.
    shown: Effect | null
    starting: boolean
  } = $props()

  // While a scene runs, each segment is drawn where it sits in the room.
  let inRoom = $derived(running && preview.length > 0 && positions.length === preview.length)

  // Until then, the picked effect is drawn on the room grid.
  const COLS = 24
  const ROWS = 14
  let grid = $state<Rgb[]>([])
  $effect(() => {
    const e = shown
    if (!e || e.kind === 'sequence' || inRoom) {
      grid = []
      return
    }
    return watch(e, COLS, ROWS, (c) => (grid = c))
  })
</script>

<section class="stage" class:running>
  {#if inRoom}
    <div class="room" aria-label="Live preview in the room">
      {#each preview as c, i (i)}
        <span
          class="dot"
          style="left: {positions[i][0] * 100}%; top: {positions[i][1] * 100}%; --c: rgb({c[0]}, {c[1]}, {c[2]})"
        ></span>
      {/each}
    </div>
  {:else if grid.length > 0}
    <div class="room" aria-label="Effect preview">
      <div class="grid" style="grid-template-columns: repeat({COLS}, 1fr)">
        {#each grid as c, i (i)}
          <span style="background: rgb({c[0]}, {c[1]}, {c[2]})"></span>
        {/each}
      </div>
      {#if starting}
        <div class="starting"><Spinner size={22} label="Starting the lights" /></div>
      {/if}
    </div>
  {:else}
    <div class="preview" aria-label="Live preview">
      {#each preview as c, i (i)}
        <span class="bar" style="--c: rgb({c[0]}, {c[1]}, {c[2]})"></span>
      {/each}
    </div>
  {/if}
</section>

<style>
  .stage {
    flex: 1;
    display: flex;
    min-height: 240px;
  }
  .preview {
    flex: 1;
    display: flex;
    align-items: stretch;
    gap: 4px;
    padding: 10px;
    min-height: 240px;
    border-radius: 10px;
    background: var(--surface);
    border: 1px solid var(--line);
  }
  .bar {
    flex: 1 1 0;
    max-width: 56px;
    border-radius: 6px;
    background: var(--c);
    box-shadow: 0 0 18px color-mix(in srgb, var(--c) 60%, transparent);
  }
  .stage:not(.running) .bar {
    opacity: 0.35;
  }

  .room {
    position: relative;
    flex: 1;
    min-height: 240px;
    border-radius: 10px;
    background: #050505;
    border: 1px solid var(--line);
    overflow: hidden;
  }
  .grid {
    position: absolute;
    inset: 0;
    display: grid;
  }
  .grid span {
    display: block;
  }
  .starting {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.35);
    color: #fff;
  }
  .dot {
    position: absolute;
    width: 28px;
    height: 28px;
    transform: translate(-50%, -50%);
    border-radius: 50%;
    background: var(--c);
    box-shadow: 0 0 14px var(--c);
  }
</style>
