<script lang="ts">
  // Play, stop, or a spinner while starting. Drawn as SVG so it stays crisp at any size.
  let { kind, size = 14 }: { kind: 'play' | 'stop'; size?: number } = $props()
</script>

<svg width={size} height={size} viewBox="0 0 16 16" aria-hidden="true" fill="currentColor">
  {#if kind === 'play'}
    <path d="M4 2.5v11l9-5.5z" />
  {:else}
    <rect x="3.5" y="3.5" width="9" height="9" rx="1.5" />
  {/if}
</svg>
