<script lang="ts">
  import type { Group, Light, Rgb } from '../api'
  import ModePicker from './ModePicker.svelte'
  import ShowStage from './ShowStage.svelte'
  import Arranger from './Arranger.svelte'
  import Button from './Button.svelte'

  let {
    group,
    lights,
    mode,
    preview,
    running,
    onrename,
    onmembers,
    onposition,
    onlight,
    onrelease,
    onstart,
    onstop,
    onpick,
    ondelete,
  }: {
    group: Group
    lights: Light[]
    mode: string
    preview: Rgb[]
    running: boolean
    onrename: (name: string) => Promise<boolean>
    onmembers: (members: string[]) => void
    onposition: (device: string, x: number, y: number) => void
    onlight: (device: string, color: [number, number, number]) => void
    onrelease: (device: string) => void
    onstart: () => void
    onstop: () => void
    onpick: (name: string) => void
    ondelete: () => void
  } = $props()

  type Tab = 'scene' | 'devices' | 'settings'
  let tab = $state<Tab>('scene')
  const tabs: { id: Tab; label: string }[] = [
    { id: 'scene', label: 'Scene' },
    { id: 'devices', label: 'Devices' },
    { id: 'settings', label: 'Settings' },
  ]

  let members = $derived(lights.filter((l) => group.members.includes(l.id)))
  let available = $derived(lights.filter((l) => !group.members.includes(l.id)))

  let name = $state('')
  $effect(() => {
    name = group.name
  })

  async function saveName(e: SubmitEvent) {
    e.preventDefault()
    await onrename(name.trim())
  }

  function addDevice(e: Event) {
    const select = e.currentTarget as HTMLSelectElement
    if (select.value) onmembers([...group.members, select.value])
    select.value = ''
  }
</script>

<section class="page">
  <header class="head">
    <h1>{group.name}</h1>
  </header>

  <div class="tabs" role="tablist">
    {#each tabs as t (t.id)}
      <button
        role="tab"
        id="tab-{t.id}"
        aria-selected={tab === t.id}
        aria-controls="panel"
        class:active={tab === t.id}
        onclick={() => (tab = t.id)}
      >
        {t.label}
        {#if t.id === 'devices'}<span class="count">{members.length}</span>{/if}
      </button>
    {/each}
  </div>

  <div id="panel" role="tabpanel" aria-labelledby="tab-{tab}" class="panel">
    {#if tab === 'scene'}
      <ModePicker {mode} {onpick} />
      <ShowStage {preview} {running} />
      <div class="actions">
        <Button
          size="lg"
          variant={running ? 'danger' : 'success'}
          onclick={() => (running ? onstop() : onstart())}
          disabled={!running && members.length === 0}
        >
          {running ? 'Stop scene' : 'Start scene'}
        </Button>
        {#if !running && members.length === 0}
          <span class="muted">Add devices first.</span>
        {/if}
      </div>
    {:else if tab === 'devices'}
      <div class="row-head">
        <span class="muted">Click a light to see it. Drag to match where it is in the room. Scenes run left to right.</span>
        {#if available.length > 0}
          <select aria-label="Add a device" onchange={addDevice} disabled={running}>
            <option value="">Add device…</option>
            {#each available as l (l.id)}
              <option value={l.id}>{l.name}</option>
            {/each}
          </select>
        {/if}
      </div>

      <Arranger {members} positions={group.positions} disabled={running} onmove={onposition} {onlight} {onrelease} />

      {#if members.length > 0}
        <ul class="devices">
          {#each members as l (l.id)}
            <li>
              <span class="name">{l.name}</span>
              <span class="muted">{l.sku}</span>
              <Button
                variant="ghost"
                size="sm"
                onclick={() => onmembers(group.members.filter((m) => m !== l.id))}
                disabled={running}
              >
                Remove
              </Button>
            </li>
          {/each}
        </ul>
      {/if}
    {:else}
      <form class="settings" onsubmit={saveName}>
        <label for="group-name" class="muted">Name</label>
        <div class="row-head">
          <input id="group-name" bind:value={name} maxlength="40" />
          <Button type="submit" variant="primary" disabled={!name.trim() || name.trim() === group.name}>Save</Button>
        </div>
      </form>

      <div class="danger-zone">
        <div>
          <strong>Delete group</strong>
          <p class="muted">Removes the group. Its devices stay known and can go in other groups.</p>
        </div>
        <Button variant="danger" onclick={ondelete} disabled={running}>Delete group</Button>
      </div>
    {/if}
  </div>
</section>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-height: 0;
    overflow-y: auto;
    padding: 20px 24px;
  }
  h1 {
    margin: 0;
    font-size: 20px;
  }

  .tabs {
    display: flex;
    gap: 4px;
    border-bottom: 1px solid var(--line);
  }
  .tabs button {
    padding: 8px 14px;
    margin-bottom: -1px;
    border: none;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--muted);
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }
  .tabs button:hover { color: var(--text); }
  .tabs button.active {
    color: var(--text);
    border-bottom-color: var(--accent);
  }
  .count {
    font-weight: 500;
    color: var(--muted);
    margin-left: 4px;
  }

  .panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .row-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  select {
    padding: 5px 10px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--raised);
    color: var(--text);
    font: inherit;
  }
  .settings {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-width: 420px;
  }
  .settings .row-head {
    justify-content: flex-start;
  }
  .settings input {
    flex: 1;
    min-width: 0;
    padding: 6px 9px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font: inherit;
  }
  .danger-zone {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-top: 12px;
    padding: 12px 14px;
    border: 1px solid #5b2730;
    border-radius: 8px;
    background: var(--surface);
  }
  .danger-zone p {
    margin: 4px 0 0;
  }
  .devices {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .devices li {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 10px;
    border-radius: 6px;
    background: var(--surface);
  }
  .devices .name {
    flex: 1;
    font-weight: 600;
  }
  .muted {
    color: var(--muted);
    font-size: 12px;
  }
</style>
