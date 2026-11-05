<script lang="ts">
  import { toasts } from '../toast.svelte'
</script>

<div class="toasts" role="status" aria-live="polite">
  {#each toasts as t (t.id)}
    <p class="toast">{t.text}</p>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: 20px;
    bottom: 20px;
    z-index: 10;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 8px;
    pointer-events: none;
  }
  .toast {
    margin: 0;
    padding: 9px 14px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--raised);
    color: var(--text);
    font-size: 13px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    animation: rise 0.18s ease-out;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
</style>
