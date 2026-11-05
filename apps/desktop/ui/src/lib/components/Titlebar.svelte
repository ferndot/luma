<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window'

  const letters = [
    { ch: 'l', color: '#ff4fa3' },
    { ch: 'u', color: '#ff8a2b' },
    { ch: 'm', color: '#facc15' },
    { ch: 'a', color: '#4ade80' },
  ]

  // Window controls only exist in the desktop app; in a browser tab they do nothing.
  const win = () => {
    try {
      return getCurrentWindow()
    } catch {
      return null
    }
  }
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region aria-label="luma">
    {#each letters as l, i (i)}<span style="color: {l.color}">{l.ch}</span>{/each}
  </div>

  <div class="controls">
    <button aria-label="Minimize" title="Minimize" onclick={() => win()?.minimize()}>
      <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true"><path d="M0 5h10" stroke="currentColor" stroke-width="1.2" /></svg>
    </button>
    <button aria-label="Maximize" title="Maximize" onclick={() => win()?.toggleMaximize()}>
      <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true"><rect x="0.6" y="0.6" width="8.8" height="8.8" fill="none" stroke="currentColor" stroke-width="1.2" /></svg>
    </button>
    <button class="close" aria-label="Close" title="Close" onclick={() => win()?.close()}>
      <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true"><path d="M0 0L10 10M10 0L0 10" stroke="currentColor" stroke-width="1.2" /></svg>
    </button>
  </div>
</header>

<style>
  .titlebar {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 48px;
    padding-left: 16px;
    border-bottom: 1px solid var(--line);
    background: var(--surface);
    user-select: none;
  }
  .brand {
    display: flex;
    align-items: center;
    height: 100%;
    flex: 1;
    font-size: 24px;
    font-weight: 800;
    letter-spacing: 0.06em;
  }
  .controls {
    display: flex;
    height: 100%;
  }
  .controls button {
    width: 46px;
    height: 100%;
    display: grid;
    place-items: center;
    border: none;
    background: transparent;
    color: var(--muted);
  }
  .controls button:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text);
  }
  .controls .close:hover {
    background: #c42b1c;
    color: #fff;
  }
</style>
