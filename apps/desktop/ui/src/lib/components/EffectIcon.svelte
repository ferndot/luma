<script lang="ts">
  import { watch } from '../preview'
  import type { Effect, Rgb } from '../api'

  // A small animated picture of an effect: the room squeezed to a few spots, redrawn as it plays.
  // The prop is not called `effect`: a local with that name turns `$effect` into a store lookup.
  let { spec, width = 64, height = 40 }: { spec: Effect; width?: number; height?: number } = $props()

  const COLS = 8
  const ROWS = 4
  let cells = $state<Rgb[]>([])

  $effect(() => watch(spec, COLS, ROWS, (c) => (cells = c)))
</script>

<span class="icon" style="width: {width}px; height: {height}px; grid-template-columns: repeat({COLS}, 1fr)" aria-hidden="true">
  {#each cells as c, i (i)}
    <span style="background: rgb({c[0]}, {c[1]}, {c[2]})"></span>
  {/each}
</span>

<style>
  .icon {
    display: grid;
    flex: none;
    overflow: hidden;
    border-radius: 7px;
    background: #050505;
  }
  .icon span {
    display: block;
  }
</style>
