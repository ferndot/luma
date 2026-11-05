<script lang="ts">
  import {
    getState,
    setMode,
    findLights,
    addSubnet,
    removeSubnet,
    setSelected,
    start,
    stop,
    type AppState,
  } from './lib/api'

  const modes = [
    { name: 'microphone', label: 'Microphone', color: '#ff4fa3' },
    { name: 'screen', label: 'Screen', color: '#3d8bff' },
    { name: 'rainbow', label: 'Rainbow', color: '#a855f7' },
    { name: 'chase', label: 'Chase', color: '#ff8a2b' },
    { name: 'breathe', label: 'Breathe', color: '#14b8a6' },
    { name: 'solid', label: 'Solid', color: '#facc15' },
  ]
  const letters = [
    { ch: 'l', color: '#ff4fa3' },
    { ch: 'u', color: '#ff8a2b' },
    { ch: 'm', color: '#facc15' },
    { ch: 'a', color: '#4ade80' },
  ]
  // Look for lights again this often while the show is idle.
  const RESCAN_MS = 60_000

  let st = $state<AppState | null>(null)
  let error = $state('')
  let selectedIps = $derived(st ? st.lights.filter((l) => l.selected).map((l) => l.ip) : [])
  let selectedCount = $derived(selectedIps.length)
  let lightCount = $derived(st ? st.lights.length : 0)

  // Poll the backend about 5 times a second while the page is open.
  $effect(() => {
    let alive = true
    const poll = async () => {
      try {
        const next = await getState()
        if (alive) {
          st = next
          error = ''
        }
      } catch (e) {
        if (alive) error = String(e)
      }
    }
    poll()
    const id = setInterval(poll, 200)
    return () => {
      alive = false
      clearInterval(id)
    }
  })

  // Find lights on launch, then quietly again while idle, so new or rebooted lights show up.
  $effect(() => {
    const scan = () => {
      if (!st?.running) run(() => findLights())
    }
    scan()
    const id = setInterval(scan, RESCAN_MS)
    return () => clearInterval(id)
  })

  async function run(action: () => Promise<unknown>) {
    error = ''
    try {
      await action()
    } catch (e) {
      error = String(e)
    }
  }

  function toggleLight(ip: string) {
    const next = new Set(selectedIps)
    if (next.has(ip)) next.delete(ip)
    else next.add(ip)
    run(() => setSelected([...next]))
  }

  function selectAll(on: boolean) {
    run(() => setSelected(on && st ? st.lights.map((l) => l.ip) : []))
  }

  function toggleShow() {
    run(() => (st?.running ? stop() : start()))
  }

  let newSubnet = $state('')

  function submitSubnet(e: SubmitEvent) {
    e.preventDefault()
    const value = newSubnet
    run(async () => {
      await addSubnet(value)
      newSubnet = ''
      await findLights()
    })
  }

  let statusText = $derived(error || st?.status || 'Connecting...')
</script>

<main>
  <header class="top">
    <h1 class="title" aria-label="luma">
      {#each letters as l, i (i)}
        <span style="color: {l.color}; animation-delay: {i * 0.15}s">{l.ch}</span>
      {/each}
    </h1>
    <p class="tagline">Sync your Govee lights to music, your screen, or a pattern</p>
  </header>

  <section class="card">
    <div class="card-head">
      <h2>Mode</h2>
    </div>
    <div class="modes">
      {#each modes as m (m.name)}
        <button
          class="mode"
          class:active={st?.mode === m.name}
          style="--c: {m.color}"
          onclick={() => run(() => setMode(m.name))}
          aria-pressed={st?.mode === m.name}
        >
          {m.label}
        </button>
      {/each}
    </div>
  </section>

  <section class="card show">
    <button
      class="power"
      class:stop={st?.running}
      onclick={toggleShow}
      disabled={!st?.running && selectedCount === 0}
    >
      {st?.running ? 'Stop' : 'Start the show'}
    </button>
    {#if !st?.running && selectedCount === 0 && lightCount > 0}
      <p class="hint">Select at least one light below to start.</p>
    {/if}

    <div class="preview" aria-label="Live preview">
      {#each st?.preview ?? [] as c, i (i)}
        <span class="swatch" style="--c: rgb({c[0]}, {c[1]}, {c[2]})"></span>
      {/each}
    </div>
  </section>

  <section class="card">
    <div class="card-head">
      <h2>Lights</h2>
      <div class="head-actions">
        {#if lightCount > 0}
          <button class="ghost" onclick={() => selectAll(selectedCount < lightCount)} disabled={st?.running}>
            {selectedCount < lightCount ? 'Select all' : 'Clear'}
          </button>
        {/if}
        <button class="ghost" onclick={() => run(() => findLights())} disabled={st?.scanning}>
          {st?.scanning ? 'Scanning…' : 'Rescan'}
        </button>
      </div>
    </div>
    {#if st?.subnet}
      <p class="subnet">Scanning <code>{st.subnet}</code></p>
    {/if}

    <form class="add-subnet" onsubmit={submitSubnet}>
      <input bind:value={newSubnet} placeholder="Add a subnet, e.g. 192.168.10" aria-label="Subnet to add" />
      <button type="submit" disabled={!newSubnet.trim()}>Add</button>
    </form>
    {#if st && st.subnets.length > 0}
      <ul class="chips">
        {#each st.subnets as s (s)}
          <li>
            <code>{s}.0/24</code>
            <button class="chip-remove" aria-label="Remove {s}.0/24" onclick={() => run(() => removeSubnet(s))}>×</button>
          </li>
        {/each}
      </ul>
    {/if}

    {#if lightCount > 0}
      <ul class="lights">
        {#each st?.lights ?? [] as l (l.ip)}
          <li>
            <label class:on={l.selected}>
              <input
                type="checkbox"
                checked={l.selected}
                disabled={st?.running}
                onchange={() => toggleLight(l.ip)}
              />
              <span class="meta">
                <span class="name">{l.sku}</span>
                <span class="ip">{l.ip}</span>
              </span>
              {#if l.razer}<span class="tag">{l.segments} zones</span>{/if}
            </label>
          </li>
        {/each}
      </ul>
    {:else if st?.scanning}
      <div class="empty">
        <span class="spinner" aria-hidden="true"></span>
        <p>Looking for lights…</p>
      </div>
    {:else}
      <div class="empty">
        <p>No lights found yet.</p>
        <p class="muted">Make sure they are on, on the same Wi-Fi as this computer, and LAN Control is enabled in the Govee app.</p>
      </div>
    {/if}
  </section>

  <p class="status" role="status" aria-live="polite">{statusText}</p>
</main>

<style>
  main {
    max-width: 760px;
    margin: 0 auto;
    padding: 24px 16px 40px;
    display: grid;
    gap: 16px;
  }

  .top {
    display: grid;
    justify-items: center;
    gap: 6px;
    margin-top: 8px;
  }
  .title {
    margin: 0;
    text-align: center;
    font-size: clamp(56px, 16vw, 112px);
    font-weight: 900;
    letter-spacing: 0.02em;
    line-height: 1;
    text-shadow: 0 0 28px rgba(168, 85, 247, 0.55);
  }
  .title span {
    display: inline-block;
    animation: bob 2.4s ease-in-out infinite;
  }
  .tagline {
    margin: 0;
    font-size: 0.95rem;
    opacity: 0.7;
  }

  .card {
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 22px;
    padding: 18px;
    backdrop-filter: blur(10px);
  }
  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
  }
  h2 {
    margin: 0;
    font-size: 1rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: #d8c8ff;
  }
  .head-actions {
    display: flex;
    gap: 8px;
  }
  .ghost {
    font: inherit;
    font-size: 0.85rem;
    font-weight: 600;
    padding: 7px 12px;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    background: transparent;
    color: #e9ddff;
    cursor: pointer;
  }
  .ghost:hover:not(:disabled) { background: rgba(255, 255, 255, 0.08); }
  .ghost:disabled { opacity: 0.5; cursor: default; }

  .modes {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: 12px;
  }
  .mode {
    min-height: 68px;
    border: none;
    border-radius: 20px;
    background: var(--c);
    color: #1a0b2e;
    font-size: 1.05rem;
    font-weight: 800;
    cursor: pointer;
    box-shadow: 0 6px 18px color-mix(in srgb, var(--c) 45%, transparent);
    transition: transform 0.15s ease, outline-color 0.15s ease;
    outline: 3px solid transparent;
  }
  .mode:hover { transform: translateY(-2px); }
  .mode.active { outline-color: #ffffff; transform: scale(1.04); }

  .show {
    display: grid;
    justify-items: center;
    gap: 14px;
  }
  .power {
    min-width: min(100%, 300px);
    min-height: 70px;
    padding: 0 28px;
    border: none;
    border-radius: 999px;
    background: #22c55e;
    color: #04240f;
    font-size: 1.25rem;
    font-weight: 800;
    cursor: pointer;
    animation: pulse-green 1.8s ease-out infinite;
  }
  .power.stop {
    background: #ef4444;
    color: #fff;
    animation: pulse-red 1.8s ease-out infinite;
  }
  .power:disabled {
    opacity: 0.45;
    animation: none;
    cursor: not-allowed;
  }
  .hint {
    margin: 0;
    font-size: 0.9rem;
    opacity: 0.7;
  }

  .preview {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
    min-height: 44px;
  }
  .swatch {
    width: 40px;
    height: 40px;
    border-radius: 14px;
    background: var(--c);
    box-shadow: 0 0 16px var(--c), 0 0 4px var(--c);
    transition: background 0.2s linear, box-shadow 0.2s linear;
  }

  .subnet {
    margin: -4px 0 12px;
    font-size: 0.85rem;
    opacity: 0.7;
  }
  .subnet code {
    font-variant-numeric: tabular-nums;
  }
  .add-subnet {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin-bottom: 10px;
  }
  .add-subnet input {
    flex: 1 1 200px;
    min-width: 0;
    font: inherit;
    border-radius: 14px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    padding: 10px 14px;
    background: rgba(10, 4, 22, 0.55);
    color: inherit;
  }
  .add-subnet button {
    font: inherit;
    font-weight: 700;
    border: none;
    border-radius: 14px;
    padding: 10px 18px;
    background: #7c3aed;
    color: #fff;
    cursor: pointer;
  }
  .add-subnet button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .chips {
    list-style: none;
    margin: 0 0 14px;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .chips li {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 6px 4px 12px;
    border-radius: 999px;
    background: rgba(88, 28, 135, 0.4);
    font-size: 0.85rem;
  }
  .chip-remove {
    border: none;
    background: transparent;
    color: #e9ddff;
    font-size: 1rem;
    cursor: pointer;
    padding: 0 4px;
  }

  .lights {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 8px;
  }
  .lights label {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: 14px;
    background: rgba(10, 4, 22, 0.35);
    border: 1px solid transparent;
    cursor: pointer;
    transition: border-color 0.15s ease, background 0.15s ease;
  }
  .lights label.on {
    border-color: rgba(168, 85, 247, 0.6);
    background: rgba(88, 28, 135, 0.35);
  }
  .lights input[type='checkbox'] {
    width: 20px;
    height: 20px;
    accent-color: #a855f7;
  }
  .lights input:disabled + .meta { opacity: 0.6; }
  .meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .name { font-weight: 700; }
  .ip { font-size: 0.85rem; opacity: 0.7; font-variant-numeric: tabular-nums; }
  .tag {
    margin-left: auto;
    font-size: 0.8rem;
    padding: 3px 9px;
    border-radius: 999px;
    background: rgba(20, 184, 166, 0.25);
    color: #9ff5ea;
  }

  .empty {
    display: grid;
    justify-items: center;
    gap: 8px;
    padding: 18px 8px;
    text-align: center;
  }
  .empty p { margin: 0; }
  .muted { opacity: 0.65; font-size: 0.9rem; }
  .spinner {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 3px solid rgba(255, 255, 255, 0.2);
    border-top-color: #a855f7;
    animation: spin 0.9s linear infinite;
  }

  .status {
    text-align: center;
    margin: 0;
    min-height: 1.4em;
    color: #e9ddff;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  @keyframes bob {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-8px); }
  }
  @keyframes pulse-green {
    0% { box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.6); }
    70% { box-shadow: 0 0 0 22px rgba(34, 197, 94, 0); }
    100% { box-shadow: 0 0 0 0 rgba(34, 197, 94, 0); }
  }
  @keyframes pulse-red {
    0% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.6); }
    70% { box-shadow: 0 0 0 22px rgba(239, 68, 68, 0); }
    100% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0); }
  }
  @media (prefers-reduced-motion: reduce) {
    *, *::before, *::after { animation: none !important; transition: none !important; }
  }
</style>
