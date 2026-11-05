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
  import Toolbar from './lib/components/Toolbar.svelte'
  import Statusbar from './lib/components/Statusbar.svelte'
  import ModePicker from './lib/components/ModePicker.svelte'
  import ShowStage from './lib/components/ShowStage.svelte'
  import LightList from './lib/components/LightList.svelte'
  import NetworkPanel from './lib/components/NetworkPanel.svelte'
  import Settings from './lib/components/Settings.svelte'
  import { setApiKey } from './lib/api'

  let settingsOpen = $state(false)

  async function saveKey(key: string): Promise<boolean> {
    return run(() => setApiKey(key))
  }

  // Look for lights again this often while the show is idle.
  const RESCAN_MS = 60_000

  let st = $state<AppState | null>(null)
  let error = $state('')
  let selectedIps = $derived(st ? st.lights.filter((l) => l.selected).map((l) => l.ip) : [])
  let selectedCount = $derived(selectedIps.length)
  let lightCount = $derived(st ? st.lights.length : 0)

  let statusText = $derived(
    error ||
      (!st?.running && selectedCount === 0 && lightCount > 0 ? 'Select at least one light to start.' : '') ||
      st?.status ||
      'Connecting...',
  )
  let tone = $derived<'idle' | 'busy' | 'live' | 'error'>(
    error ? 'error' : st?.running ? 'live' : st?.scanning ? 'busy' : 'idle',
  )

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

  // Runs an action, shows any error, and reports whether it worked.
  async function run(action: () => Promise<unknown>): Promise<boolean> {
    error = ''
    try {
      await action()
      return true
    } catch (e) {
      error = String(e)
      return false
    }
  }

  function pickMode(name: string) {
    run(() => setMode(name))
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

  async function addNetwork(value: string): Promise<boolean> {
    const ok = await run(() => addSubnet(value))
    if (ok) run(() => findLights())
    return ok
  }
</script>

<div class="app">
  <Toolbar
    running={st?.running ?? false}
    canStart={selectedCount > 0}
    {settingsOpen}
    ontoggle={toggleShow}
    onsettings={() => (settingsOpen = !settingsOpen)}
  />

  <aside class="sidebar">
    <ModePicker mode={st?.mode} onpick={pickMode} />
    <NetworkPanel
      subnets={st?.subnets ?? []}
      onadd={addNetwork}
      onremove={(s) => run(() => removeSubnet(s))}
    />
  </aside>

  {#if settingsOpen}
    <main class="content">
      <Settings hasKey={st?.has_key ?? false} keyStatus={st?.key_status ?? ''} onsave={saveKey} />
    </main>
  {:else}
  <main class="content">
    <ShowStage preview={st?.preview ?? []} running={st?.running ?? false} />
    <LightList
      lights={st?.lights ?? []}
      running={st?.running ?? false}
      scanning={st?.scanning ?? false}
      subnet={st?.subnet ?? ''}
      ontoggle={toggleLight}
      onselectall={selectAll}
      onrescan={() => run(() => findLights())}
    />
  </main>
  {/if}

  <Statusbar text={statusText} {tone} />
</div>

<style>
  .app {
    height: 100vh;
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr) auto;
    grid-template-areas:
      'toolbar toolbar'
      'sidebar content'
      'status status';
    overflow: hidden;
  }
  .app > :global(.toolbar) { grid-area: toolbar; }
  .app > :global(.statusbar) { grid-area: status; }
  .sidebar {
    grid-area: sidebar;
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 14px 12px;
    overflow-y: auto;
    border-right: 1px solid var(--line);
    background: var(--surface);
  }
  .content {
    grid-area: content;
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-height: 0;
    min-width: 0;
    background: var(--bg);
  }

  /* Narrow windows: the sidebar moves above the content and scrolls on its own. */
  @media (max-width: 700px) {
    .app {
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: auto auto minmax(0, 1fr) auto;
      grid-template-areas:
        'toolbar'
        'sidebar'
        'content'
        'status';
    }
    .sidebar {
      max-height: 34vh;
      border-right: none;
      border-bottom: 1px solid var(--line);
    }
  }
</style>
