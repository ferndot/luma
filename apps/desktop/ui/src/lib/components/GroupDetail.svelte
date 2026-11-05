<script lang="ts">
  import type { Group, Light, Rgb } from '../api'
  import ShowStage from './ShowStage.svelte'
  import Arranger from './Arranger.svelte'
  import Button from './Button.svelte'

  let {
    group,
    lights,
    preview,
    positions,
    running,
    onrename,
    onmembers,
    onposition,
    onlight,
    onrelease,
    onstart,
    onstop,
    ondelete,
  }: {
    group: Group
    lights: Light[]
    preview: Rgb[]
    positions: [number, number][]
    running: boolean
    onrename: (name: string) => Promise<boolean>
    onmembers: (members: string[]) => void
    onposition: (device: string, x: number, y: number) => void
    onlight: (device: string, color: [number, number, number]) => void
    onrelease: (device: string) => void
    onstart: () => void
    onstop: () => void
    ondelete: () => void
  } = $props()

  type Tab = 'scene' | 'layout' | 'devices' | 'settings'
  let tab = $state<Tab>('scene')
  const tabs: { id: Tab; label: string }[] = [
    { id: 'scene', label: 'Scene' },
    { id: 'layout', label: 'Layout' },
    { id: 'devices', label: 'Devices' },
    { id: 'settings', label: 'Settings' },
  ]

  let members = $derived(lights.filter((l) => group.members.includes(l.id)))
  let available = $derived(lights.filter((l) => !group.members.includes(l.id)))

  function addMember(id: string) {
    onmembers([...group.members, id])
  }

  let name = $state('')
  $effect(() => {
    name = group.name
  })

  async function saveName(e: SubmitEvent) {
    e.preventDefault()
    await onrename(name.trim())
  }

</script>

<section class="page">
  <header class="head">
    <h1>{group.name}</h1>
    <div class="start">
      {#if !running && members.length === 0}
        <span class="muted">Add devices first.</span>
      {/if}
      <Button
        size="lg"
        variant={running ? 'danger' : 'success'}
        onclick={() => (running ? onstop() : onstart())}
        disabled={!running && members.length === 0}
      >
        {running ? 'Stop scene' : 'Start scene'}
      </Button>
    </div>
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
      <ShowStage {preview} {positions} {running} />
    {:else if tab === 'layout'}
      <p class="muted hint">Press a device to light it. Drag to match where it is in the room. Scenes run left to right.</p>
      {#if members.length === 0}
        <p class="muted">Add devices in the Devices tab first.</p>
      {/if}
      <Arranger {members} positions={group.positions} disabled={running} onmove={onposition} {onlight} {onrelease} />
    {:else if tab === 'devices'}
      <div class="devices-cols">
        <section>
          <h2>In this group <span class="count">{members.length}</span></h2>
          {#if members.length === 0}
            <p class="muted">No devices yet. Add some from the list on the right.</p>
          {:else}
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
        </section>

        <section>
          <h2>Other devices <span class="count">{available.length}</span></h2>
          {#if available.length === 0}
            <p class="muted">Every known device is in this group.</p>
          {:else}
            <ul class="devices">
              {#each available as l (l.id)}
                <li>
                  <span class="name">{l.name}</span>
                  <span class="muted">{l.sku}</span>
                  <Button variant="ghost" size="sm" onclick={() => addMember(l.id)} disabled={running}>Add</Button>
                </li>
              {/each}
            </ul>
          {/if}
        </section>
      </div>
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
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-height: 0;
    overflow-y: auto;
    padding: 20px 24px;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  h1 {
    margin: 0;
    font-size: 20px;
  }
  .start {
    display: flex;
    align-items: center;
    gap: 12px;
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
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 0;
  }

  .hint {
    margin: 0;
  }
  .devices-cols {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 20px;
    align-items: start;
  }
  .devices-cols h2 {
    margin: 0 0 8px;
    font-size: 14px;
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
