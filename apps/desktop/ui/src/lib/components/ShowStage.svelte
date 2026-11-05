<script lang="ts">
  import type { Rgb } from '../api'

  let {
    preview,
    positions,
    running,
  }: { preview: Rgb[]; positions: [number, number][]; running: boolean } = $props()

  // While a scene runs, each segment is drawn where it sits in the room.
  let inRoom = $derived(running && preview.length > 0 && positions.length === preview.length)
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
    min-height: 180px;
  }
  .preview {
    flex: 1;
    display: flex;
    align-items: stretch;
    gap: 4px;
    padding: 10px;
    min-height: 120px;
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
    transition: background 0.2s linear, box-shadow 0.2s linear;
  }
  .stage:not(.running) .bar {
    opacity: 0.35;
  }

  .room {
    position: relative;
    flex: 1;
    min-height: 120px;
    border-radius: 10px;
    background: var(--surface);
    border: 1px solid var(--line);
    overflow: hidden;
  }
  .dot {
    position: absolute;
    width: 22px;
    height: 22px;
    transform: translate(-50%, -50%);
    border-radius: 50%;
    background: var(--c);
    box-shadow: 0 0 14px var(--c);
    transition: background 0.2s linear, box-shadow 0.2s linear;
  }
</style>
