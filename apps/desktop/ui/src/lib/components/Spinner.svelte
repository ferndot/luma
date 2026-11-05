<script lang="ts">
  // A small spinning ring, shown while something is starting.
  let { size = 16, label = 'Starting' }: { size?: number; label?: string } = $props()
</script>

<span class="spinner" style="width: {size}px; height: {size}px" role="status" aria-label={label}></span>

<style>
  .spinner {
    display: inline-block;
    flex: none;
    border: 2px solid color-mix(in srgb, currentColor 25%, transparent);
    border-top-color: currentColor;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
