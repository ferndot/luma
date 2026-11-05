<script lang="ts">
  import type { Light } from '../api'

  let {
    lights,
    running,
    scanning,
    subnet,
    ontoggle,
    onselectall,
    onrescan,
  }: {
    lights: Light[]
    running: boolean
    scanning: boolean
    subnet: string
    ontoggle: (ip: string) => void
    onselectall: (on: boolean) => void
    onrescan: () => void
  } = $props()

  let selectedCount = $derived(lights.filter((l) => l.selected).length)
</script>

<section class="lights-pane">
  <div class="toolbar">
    <h2 class="section-title">
      Lights <span class="count">{selectedCount} of {lights.length} selected</span>
    </h2>
    <div class="actions">
      {#if lights.length > 0}
        <button onclick={() => onselectall(selectedCount < lights.length)} disabled={running}>
          {selectedCount < lights.length ? 'Select all' : 'Clear'}
        </button>
      {/if}
      <button onclick={onrescan} disabled={scanning}>{scanning ? 'Scanning…' : 'Rescan'}</button>
    </div>
  </div>

  {#if subnet}
    <p class="muted scan">Scanning {subnet}.0/24</p>
  {/if}

  {#if lights.length > 0}
    <ul class="rows">
      {#each lights as l (l.ip)}
        <li>
          <label class:locked={running}>
            <input
              class="sr"
              type="checkbox"
              checked={l.selected}
              disabled={running}
              onchange={() => ontoggle(l.ip)}
            />
            <span class="switch" class:on={l.selected} aria-hidden="true"></span>
            <span class="name">{l.name}</span>
            <span class="ip">{l.sku} · {l.ip}</span>
            {#if l.razer}<span class="zones">{l.segments} zones</span>{/if}
          </label>
        </li>
      {/each}
    </ul>
  {:else if scanning}
    <div class="empty">
      <span class="spinner" aria-hidden="true"></span>
      <p class="muted">Looking for lights…</p>
    </div>
  {:else}
    <div class="empty">
      <p>No lights found.</p>
      <p class="muted">Make sure they are on, on the same Wi-Fi as this computer, and LAN Control is enabled in the Govee app.</p>
    </div>
  {/if}
</section>

<style>
  .lights-pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 14px 16px 0;
  }
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin-bottom: 8px;
  }
  .toolbar .section-title {
    margin: 0;
  }
  .actions {
    display: flex;
    gap: 6px;
  }
  .actions button {
    padding: 5px 12px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--raised);
  }
  .actions button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .scan {
    margin: 0 0 8px;
  }

  .rows {
    list-style: none;
    margin: 0;
    padding: 0 0 14px;
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }
  .rows li + li {
    border-top: 1px solid var(--line);
  }
  .rows label {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 6px;
    border-radius: 4px;
  }
  .rows label:hover { background: rgba(255, 255, 255, 0.03); }
  .rows label.locked { cursor: default; }
  .rows label:focus-within {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  /* Visually hidden native checkbox; keeps keyboard and screen reader support. */
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    margin: 0;
    overflow: hidden;
    clip: rect(0 0 0 0);
    opacity: 0;
  }

  .switch {
    position: relative;
    flex: none;
    width: 34px;
    height: 18px;
    border-radius: 999px;
    background: #3a3450;
    transition: background 0.15s ease;
  }
  .switch::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #e8e3f2;
    transition: transform 0.15s ease;
  }
  .switch.on { background: var(--accent); }
  .switch.on::after { transform: translateX(16px); }
  .rows label.locked .switch { opacity: 0.5; }

  .name {
    font-weight: 600;
  }
  .ip {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .zones {
    margin-left: auto;
    color: var(--muted);
    font-size: 12px;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 32px 12px;
    text-align: center;
  }
  .empty p { margin: 0; }

  .spinner {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 2px solid var(--line);
    border-top-color: var(--accent);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
