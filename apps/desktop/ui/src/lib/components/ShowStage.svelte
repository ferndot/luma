<script lang="ts">
  import type { Rgb } from '../api'

  let { preview, running }: { preview: Rgb[]; running: boolean } = $props()
</script>

<section class="stage" class:running>
  <div class="preview" aria-label="Live preview">
    {#each preview as c, i (i)}
      <span class="bar" style="--c: rgb({c[0]}, {c[1]}, {c[2]})"></span>
    {/each}
  </div>
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
</style>
