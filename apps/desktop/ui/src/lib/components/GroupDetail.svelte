<script lang="ts">
  import type { Group, Light, Rgb } from '../api'
  import ModePicker from './ModePicker.svelte'
  import ShowStage from './ShowStage.svelte'
  import Arranger from './Arranger.svelte'

  let {
    group,
    lights,
    mode,
    preview,
    running,
    onrename,
    onmembers,
    onposition,
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
    onrename: (name: string) => void
    onmembers: (members: string[]) => void
    onposition: (device: string, x: number, y: number) => void
    onstart: () => void
    onstop: () => void
    onpick: (name: string) => void
    ondelete: () => void
  } = $props()

  let members = $derived(lights.filter((l) => group.members.includes(l.id)))
  let available = $derived(lights.filter((l) => !group.members.includes(l.id)))

  function addDevice(e: Event) {
    const select = e.currentTarget as HTMLSelectElement
    if (select.value) onmembers([...group.members, select.value])
    select.value = ''
  }
</script>

<section class="page">
  <header class="head">
    <input class="title" value={group.name} aria-label="Group name" maxlength="40" onchange={(e) => onrename(e.currentTarget.value)} />
    <button class="danger" onclick={ondelete}>Delete group</button>
  </header>

  <div class="block">
    <h2 class="section-title">Scene</h2>
    <ModePicker {mode} {onpick} />
    <ShowStage {preview} {running} />
    <div class="actions">
      <button class="scene" class:stop={running} onclick={() => (running ? onstop() : onstart())} disabled={!running && members.length === 0}>
        {running ? 'Stop scene' : 'Start scene'}
      </button>
      {#if !running && members.length === 0}
        <span class="muted">Add devices to start this scene.</span>
      {/if}
    </div>
  </div>

  <div class="block">
    <div class="row-head">
      <h2 class="section-title">Devices <span class="count">{members.length}</span></h2>
      {#if available.length > 0}
        <select aria-label="Add a device" onchange={addDevice} disabled={running}>
          <option value="">Add device…</option>
          {#each available as l (l.id)}
            <option value={l.id}>{l.name}</option>
          {/each}
        </select>
      {/if}
    </div>

    <Arranger {members} positions={group.positions} onmove={onposition} />

    {#if members.length > 0}
      <ul class="devices">
        {#each members as l (l.id)}
          <li>
            <span class="name">{l.name}</span>
            <span class="muted">{l.sku}</span>
            <button class="remove" onclick={() => onmembers(group.members.filter((m) => m !== l.id))} disabled={running}>
              Remove
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</section>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 18px;
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
  .title {
    flex: 1;
    min-width: 0;
    padding: 4px 6px;
    border: 1px solid transparent;
    border-radius: 6px;
    background: transparent;
    color: var(--text);
    font-size: 20px;
    font-weight: 700;
  }
  .title:hover,
  .title:focus {
    border-color: var(--line);
    background: var(--surface);
  }
  .block {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .row-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .row-head .section-title,
  .block > .section-title {
    margin: 0;
  }
  .count {
    font-weight: 500;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .scene {
    min-width: 150px;
    height: 36px;
    padding: 0 18px;
    border: none;
    border-radius: 6px;
    background: #2f9e5c;
    color: #fff;
    font-weight: 600;
  }
  .scene.stop { background: #c8434f; }
  .scene:disabled {
    background: var(--raised);
    color: var(--muted);
    cursor: default;
  }
  select,
  .danger,
  .remove {
    padding: 5px 10px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--raised);
  }
  .danger {
    color: #fca5a5;
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
  .remove {
    padding: 2px 8px;
    font-size: 12px;
    color: var(--muted);
    background: transparent;
  }
  .muted {
    color: var(--muted);
    font-size: 12px;
  }
</style>
