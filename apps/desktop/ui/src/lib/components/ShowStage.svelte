<script lang="ts">
  import type { Rgb } from '../api'

  let { preview, running }: { preview: Rgb[]; running: boolean } = $props()
</script>

<section class="stage">
  <h2 class="section-title">Live preview</h2>
  <div class="preview" class:running aria-label="Live preview">
    {#if preview.length > 0}
      {#each preview as c, i (i)}
        <span class="bar" style="--c: rgb({c[0]}, {c[1]}, {c[2]})"></span>
      {/each}
    {:else}
      <p class="muted">Press Start show to light up your lights.</p>
    {/if}
  </div>
</section>

<style>
  .stage {
    padding: 14px 16px 0;
  }
  .section-title {
    margin-bottom: 8px;
  }
  .preview {
    display: flex;
    align-items: stretch;
    gap: 3px;
    height: 48px;
    padding: 4px;
    border-radius: 6px;
    background: var(--bg);
    border: 1px solid var(--line);
  }
  .preview .muted {
    margin: auto;
  }
  .bar {
    flex: 1 1 0;
    max-width: 40px;
    border-radius: 3px;
    background: var(--c);
    transition: background 0.2s linear;
  }
</style>
