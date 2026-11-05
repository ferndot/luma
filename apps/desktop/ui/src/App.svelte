<script lang="ts">
  import { untrack } from 'svelte'
  import ModePicker from './lib/components/ModePicker.svelte'
  import {
    getState,
    playEffect,
    saveEffect,
    resetEffect,
    deleteEffect,
    type NamedEffect,
    iconEffect,
    findLights,
    addSubnet,
    removeSubnet,
    createGroup,
    renameGroup,
    setGroupMembers,
    setGroupPosition,
    lightDevice,
    releaseDevice,
    startGroup,
    deleteGroup,
    setApiKey,
    stop,
    type AppState,
  } from './lib/api'
  import Sidebar, { type Page } from './lib/components/Sidebar.svelte'
  import Titlebar from './lib/components/Titlebar.svelte'
  import GroupDetail from './lib/components/GroupDetail.svelte'
  import Settings from './lib/components/Settings.svelte'
  import EffectBuilder from './lib/components/EffectBuilder.svelte'
  import Toasts from './lib/components/Toasts.svelte'
  import { toast } from './lib/toast.svelte'

  // Look for lights again this often while no scene is running.
  const RESCAN_MS = 60_000

  let st = $state<AppState | null>(null)
  let error = $state('')
  let page = $state<Page>({ kind: 'home' })

  // The effect the user just picked. It shows in the scene at once, before the lights have started.
  let selected = $state<string | null>(null)
  // The effect shown as playing: the picked one, else the one the show is running.
  let shownId = $derived(selected ?? (st?.running ? (st?.active_effect ?? null) : null))
  // Picked, but the lights are not yet running and drawing.
  let starting = $derived(selected !== null && (!st?.running || st.preview.length === 0))
  let stageEffect = $derived.by(() => {
    const found = st?.effects.find((e) => e.id === shownId)
    return found ? iconEffect(found.effect, st?.effects ?? []) : null
  })
  // Once the lights have run for the pick, the show itself says what plays, so the pick is dropped.
  // If they never start, the pick is dropped when the show stops.
  let started = false
  $effect(() => {
    if (selected === null) {
      started = false
      return
    }
    if (st?.running) started = true
    if (st?.running && st.preview.length > 0) selected = null
    else if (started && st && !st.running) selected = null
  })

  // Play an effect on the open group's lights: show it at once, set it on the show, then start the show.
  async function playPicked(id: string) {
    if (!openGroup) {
      error = 'Open a group to play an effect on its lights.'
      return
    }
    selected = id
    const ok = await run(() => playEffect(id))
    if (!ok) {
      selected = null
      return
    }
    if (!st?.running) {
      const began = await run(() => startGroup(openGroup.id))
      if (!began) selected = null
    }
  }

  async function stopPicked() {
    selected = null
    await run(() => stop())
  }

  // The open group, or back to the list if it was deleted.
  let openGroup = $derived(
    page.kind === 'group' ? (st?.groups.find((g) => g.id === (page as { id: string }).id) ?? null) : null,
  )
  $effect(() => {
    if (page.kind === 'group' && st && !openGroup) page = { kind: 'home' }
  })

  // Poll the backend while the page is open: 5 times a second during a scene (the preview moves),
  // once a second otherwise. The next poll waits for this one, so slow calls never pile up.
  $effect(() => {
    let alive = true
    let timer: ReturnType<typeof setTimeout>
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
      if (alive) timer = setTimeout(poll, st?.running ? 200 : 1000)
    }
    poll()
    return () => {
      alive = false
      clearTimeout(timer)
    }
  })

  // Find lights on launch, then quietly again while idle, so new or rebooted lights show up.
  // Read `st` only inside the timer callback: reading it in the effect would re-run it on every poll.
  $effect(() => {
    const scan = () => {
      if (!st?.running) run(() => findLights())
    }
    untrack(scan)
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

  async function newGroup(name: string): Promise<boolean> {
    let id = ''
    const ok = await run(async () => {
      id = await createGroup(name)
    })
    if (ok) page = { kind: 'group', id }
    return ok
  }

  // Saves an effect from the builder; returns its id, or null if it was refused (the error shows at the top).
  async function saveEffectDraft(effect: NamedEffect): Promise<string | null> {
    let id: string | null = null
    const ok = await run(async () => {
      id = await saveEffect(effect)
    })
    if (ok) toast(`Saved “${effect.name.trim()}”`)
    return ok ? id : null
  }

  async function resetDraft(id: string) {
    if (await run(() => resetEffect(id))) toast('Reset to how it started')
  }

  async function deleteDraft(id: string) {
    if (await run(() => deleteEffect(id))) toast('Effect deleted')
  }

  async function saveKey(key: string): Promise<boolean> {
    const ok = await run(() => setApiKey(key))
    if (ok) toast(key.trim() ? 'API key saved' : 'API key cleared')
    return ok
  }

  async function addNetwork(value: string): Promise<boolean> {
    const ok = await run(() => addSubnet(value))
    if (ok) run(() => findLights())
    return ok
  }
</script>

<div class="app" class:with-picker={openGroup !== null}>
  <Titlebar />
  <Toasts />

  <Sidebar
    groups={st?.groups ?? []}
    {page}
    scanning={st?.scanning ?? false}
    onnav={(p) => (page = p)}
    oncreate={newGroup}
    onrescan={() => run(() => findLights())}
  />

  <main class="content">
    {#if error}
      <p class="error" role="alert">{error}</p>
    {/if}

    {#if page.kind === 'effects'}
      <EffectBuilder
        effects={st?.effects ?? []}
        activeEffect={shownId}
        running={st?.running ?? false}
        {starting}
        onsave={saveEffectDraft}
        onreset={resetDraft}
        ondelete={deleteDraft}
        onplay={playPicked}
        onstop={stopPicked}
      />
    {:else if page.kind === 'settings'}
      <Settings
        hasKey={st?.has_key ?? false}
        keyStatus={st?.key_status ?? ''}
        onsave={saveKey}
        subnets={st?.subnets ?? []}
        onadd={addNetwork}
        onremove={(s) => run(() => removeSubnet(s))}
      />
    {:else if openGroup}
      <GroupDetail
        group={openGroup}
        lights={st?.lights ?? []}
        preview={st?.preview ?? []}
        positions={st?.positions ?? []}
        running={st?.running ?? false}
        stage={stageEffect}
        {starting}
        onrename={(name) => run(() => renameGroup(openGroup.id, name))}
        onmembers={(members) => run(() => setGroupMembers(openGroup.id, members))}
        onposition={(device, x, y) => run(() => setGroupPosition(openGroup.id, device, x, y))}
        onlight={(device, color) => run(() => lightDevice(openGroup.id, device, color))}
        onrelease={(device) => run(() => releaseDevice(device))}
        ondelete={() => {
          run(() => deleteGroup(openGroup.id))
          page = { kind: 'home' }
        }}
      />
    {:else}
      <div class="welcome">
        <p>Pick a group in the sidebar, or type a name there to make one.</p>
      </div>
    {/if}
  </main>

  {#if openGroup}
    <ModePicker
      effects={st?.effects ?? []}
      activeEffect={shownId}
      running={st?.running ?? false}
      {starting}
      onplay={playPicked}
      onstop={stopPicked}
    />
  {/if}
</div>

<style>
  .app {
    height: 100vh;
    display: grid;
    grid-template-columns: 200px minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr);
    overflow: hidden;
  }
  .content {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    background: var(--bg);
  }
  /* Group pages get an effects sidebar on the right. */
  .app.with-picker {
    grid-template-columns: 200px minmax(0, 1fr) 220px;
  }
  .app :global(.picker) {
    grid-column: 3;
    grid-row: 2;
  }
  .welcome {
    padding: 24px;
    color: var(--muted);
  }
  .welcome p {
    margin: 0;
  }
  .error {
    margin: 0;
    padding: 8px 24px;
    background: rgba(239, 68, 68, 0.15);
    color: #fecaca;
    font-size: 13px;
  }

  /* Narrow windows: the sidebar becomes a strip above the page. */
  @media (max-width: 640px) {
    .app,
    .app.with-picker {
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: auto minmax(0, 1fr);
    }
    .app :global(.picker) {
      grid-column: 1;
      grid-row: 3;
    }
  }
</style>
