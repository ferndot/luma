<script lang="ts">
  import type { Group } from '../api'
  import Button from './Button.svelte'

  export type Page = { kind: 'home' } | { kind: 'group'; id: string } | { kind: 'effects' } | { kind: 'settings' }

  let {
    groups,
    page,
    scanning,
    onnav,
    oncreate,
    onrescan,
  }: {
    groups: Group[]
    page: Page
    scanning: boolean
    onnav: (page: Page) => void
    oncreate: (name: string) => Promise<boolean>
    onrescan: () => void
  } = $props()

  let name = $state('')
  let dialog: HTMLDialogElement
  let input: HTMLInputElement

  function openDialog() {
    name = ''
    dialog.showModal()
    input.focus()
  }

  async function submit(e: SubmitEvent) {
    e.preventDefault()
    if (await oncreate(name)) dialog.close()
  }
</script>

<aside class="sidebar">
  <Button class="new-btn" onclick={openDialog}>+ New group</Button>

  <dialog bind:this={dialog} class="modal" aria-labelledby="new-group-title">
    <form method="dialog" onsubmit={submit}>
      <h2 id="new-group-title">New group</h2>
      <input bind:this={input} bind:value={name} placeholder="e.g. Desk" aria-label="Group name" maxlength="40" />
      <div class="modal-actions">
        <Button type="button" onclick={() => dialog.close()}>Cancel</Button>
        <Button type="submit" variant="primary" disabled={!name.trim()}>Create</Button>
      </div>
    </form>
  </dialog>

  <nav class="nav">
    {#if groups.length > 0}
      <h2 class="section-title">Groups</h2>
      {#each groups as g (g.id)}
        <button
          class="item"
          class:active={page.kind === 'group' && page.id === g.id}
          onclick={() => onnav({ kind: 'group', id: g.id })}
        >
          {g.name}
        </button>
      {/each}
    {:else}
      <p class="muted">No groups yet. Type a name above to make one.</p>
    {/if}

    <h2 class="section-title">Effects</h2>
    <button class="item" class:active={page.kind === 'effects'} onclick={() => onnav({ kind: 'effects' })}>
      Effect builder
    </button>
  </nav>

  <div class="bottom">
    <button class="item" onclick={onrescan} disabled={scanning}>
      {scanning ? 'Finding lights…' : 'Find lights'}
    </button>
    <button class="item" class:active={page.kind === 'settings'} onclick={() => onnav({ kind: 'settings' })}>
      Settings
    </button>
  </div>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-height: 0;
    padding: 14px 10px;
    border-right: 1px solid var(--line);
    background: var(--surface);
    overflow: hidden;
  }
  .modal {
    width: min(340px, 90vw);
    padding: 18px;
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--surface);
    color: var(--text);
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
  }
  .modal::backdrop {
    background: rgba(0, 0, 0, 0.5);
  }
  .modal form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .modal h2 {
    margin: 0;
    font-size: 16px;
  }
  .modal input {
    padding: 8px 10px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-height: 0;
    overflow-y: auto;
  }
  .nav .section-title {
    margin: 6px 8px;
  }
  .muted {
    margin: 4px 8px;
    color: var(--muted);
    font-size: 12px;
    line-height: 1.4;
  }
  .bottom {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
    border-top: 1px solid var(--line);
    padding-top: 10px;
  }
  .item {
    display: block;
    width: 100%;
    padding: 7px 10px;
    border: none;
    border-radius: 6px;
    background: transparent;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .item:hover:not(:disabled) { background: rgba(255, 255, 255, 0.05); }
  .item:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .item.active {
    background: var(--raised);
    font-weight: 600;
  }
</style>
