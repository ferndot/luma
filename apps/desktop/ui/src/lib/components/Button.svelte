<script lang="ts">
  import type { Snippet } from 'svelte'
  import type { HTMLButtonAttributes } from 'svelte/elements'

  type Variant = 'default' | 'primary' | 'success' | 'danger' | 'ghost'
  type Size = 'sm' | 'md' | 'lg'

  let {
    variant = 'default',
    size = 'md',
    class: className = '',
    children,
    ...rest
  }: HTMLButtonAttributes & { variant?: Variant; size?: Size; children?: Snippet } = $props()
</script>

<button class="btn {variant} {size} {className}" {...rest}>
  {@render children?.()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--raised);
    color: var(--text);
    font: inherit;
    font-weight: 600;
    white-space: nowrap;
    cursor: pointer;
    transition: background 0.12s ease, border-color 0.12s ease, color 0.12s ease, opacity 0.12s ease;
  }
  .btn:hover:not(:disabled) {
    background: #342e45;
  }
  .btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .btn:disabled {
    opacity: 0.45;
    cursor: default;
  }

  /* Sizes */
  .btn.sm {
    height: 26px;
    padding: 0 8px;
    font-size: 12px;
    border-radius: 5px;
  }
  .btn.lg {
    height: 36px;
    padding: 0 18px;
    font-size: 14px;
    min-width: 150px;
  }

  /* Variants */
  .btn.primary {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }
  .btn.primary:hover:not(:disabled) {
    background: #b872ff;
  }
  .btn.success {
    background: #2f9e5c;
    border-color: #2f9e5c;
    color: #fff;
  }
  .btn.success:hover:not(:disabled) {
    background: #36b06a;
  }
  .btn.danger {
    background: rgba(200, 67, 79, 0.14);
    border-color: #5b2730;
    color: #fca5a5;
  }
  .btn.danger:hover:not(:disabled) {
    background: rgba(200, 67, 79, 0.28);
  }
  .btn.ghost {
    background: transparent;
    border-color: transparent;
    color: var(--muted);
  }
  .btn.ghost:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.06);
    color: var(--text);
  }
</style>
