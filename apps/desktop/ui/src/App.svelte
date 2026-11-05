<script lang="ts">
  import { getState, setMode, findLights, setSelected, start, stop, type AppState } from './lib/api'

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

  let st = $state<AppState | null>(null)
  let error = $state('')
  let subnet = $state('192.168.1')
  let selectedIps = $derived(st ? st.lights.filter((l) => l.selected).map((l) => l.ip) : [])

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

  function toggleShow() {
    run(() => (st?.running ? stop() : start()))
  }

  function search(e: SubmitEvent) {
    e.preventDefault()
    run(() => findLights(subnet))
  }
</script>

<main>
  <h1 class="title" aria-label="luma">
    {#each letters as l, i (i)}
      <span style="color: {l.color}; animation-delay: {i * 0.15}s">{l.ch}</span>
    {/each}
  </h1>

  <section class="card">
    <h2>Mode</h2>
    <div class="modes">
      {#each modes as m (m.name)}
        <button
          class="mode"
          class:active={st?.mode === m.name}
          style="--c: {m.color}"
          onclick={() => run(() => setMode(m.name))}
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
      disabled={!st?.running && selectedIps.length === 0}
    >
      {st?.running ? 'Stop' : 'Start the show'}
    </button>

    <div class="preview" aria-label="Live preview">
      {#each st?.preview ?? [] as c, i (i)}
        <span class="swatch" style="--c: rgb({c[0]}, {c[1]}, {c[2]})"></span>
      {/each}
    </div>
  </section>

  <section class="card">
    <h2>Lights</h2>
    <form class="find" onsubmit={search}>
      <input bind:value={subnet} placeholder="192.168.1" aria-label="Subnet" />
      <button type="submit">Find lights</button>
    </form>

    {#if st && st.lights.length > 0}
      <ul class="lights">
        {#each st.lights as l (l.ip)}
          <li>
            <label>
              <input type="checkbox" checked={l.selected} onchange={() => toggleLight(l.ip)} />
              <span class="name">{l.sku}</span>
              <span class="ip">{l.ip}</span>
              {#if l.razer}<span class="tag">{l.segments} zones</span>{/if}
            </label>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="muted">No lights yet. Enter your subnet and search.</p>
    {/if}
  </section>

  <p class="status" role="status" aria-live="polite">{error || st?.status || 'Connecting...'}</p>
</main>

<style>
  main {
    max-width: 760px;
    margin: 0 auto;
    padding: 24px 16px 40px;
    display: grid;
    gap: 16px;
  }

  .title {
    margin: 8px 0 0;
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

  .card {
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 22px;
    padding: 18px;
    backdrop-filter: blur(10px);
  }
  h2 {
    margin: 0 0 12px;
    font-size: 1rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: #d8c8ff;
  }

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
    gap: 18px;
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

  .find {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin-bottom: 14px;
  }
  .find input {
    flex: 1 1 160px;
    min-width: 0;
  }
  input,
  .find button {
    font: inherit;
    border-radius: 14px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    padding: 12px 14px;
    background: rgba(10, 4, 22, 0.55);
    color: inherit;
  }
  .find button {
    background: #7c3aed;
    border-color: transparent;
    font-weight: 700;
    cursor: pointer;
    padding: 12px 18px;
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
    gap: 10px;
    padding: 12px 14px;
    border-radius: 14px;
    background: rgba(10, 4, 22, 0.35);
    cursor: pointer;
  }
  .lights input[type='checkbox'] {
    width: 20px;
    height: 20px;
    accent-color: #a855f7;
  }
  .name { font-weight: 700; }
  .ip { opacity: 0.7; font-variant-numeric: tabular-nums; }
  .tag {
    margin-left: auto;
    font-size: 0.8rem;
    padding: 3px 9px;
    border-radius: 999px;
    background: rgba(20, 184, 166, 0.25);
    color: #9ff5ea;
  }
  .muted { opacity: 0.65; margin: 0; }

  .status {
    text-align: center;
    margin: 0;
    min-height: 1.4em;
    color: #e9ddff;
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
