<script lang="ts">
  import type { NamedEffect, Scene, SceneStep } from '../api'
  import Button from './Button.svelte'

  let {
    scenes,
    effects,
    activeScene,
    onsave,
    ondelete,
    onplay,
  }: {
    scenes: Scene[]
    effects: NamedEffect[]
    activeScene: string | null
    onsave: (scene: Scene) => Promise<string | null>
    ondelete: (id: string) => void
    onplay: (id: string) => void
  } = $props()

  const MAX_STEPS = 12
  const blankStep = (): SceneStep => ({ effect: effects[0]?.id ?? 'rainbow', secs: 10 })

  // The scene being edited. A null id means it is not saved yet.
  let draft = $state<Scene>({ id: '', name: '', steps: [blankStep()], repeat: false })
  let editing = $state(false)
  let saving = $state(false)
  let message = $state('')

  let sequence = $derived(draft.steps.length > 1)
  let invalid = $derived(draft.name.trim() === '' || draft.steps.length === 0)

  function edit(scene: Scene) {
    draft = { ...scene, steps: scene.steps.map((s) => ({ ...s })) }
    editing = true
    message = ''
  }

  function startNew() {
    draft = { id: '', name: '', steps: [blankStep()], repeat: false }
    editing = true
    message = ''
  }

  function addStep() {
    if (draft.steps.length < MAX_STEPS) draft.steps.push(blankStep())
  }

  function removeStep(i: number) {
    draft.steps.splice(i, 1)
  }

  async function save(e: SubmitEvent) {
    e.preventDefault()
    saving = true
    // A preset holds its one effect; the timing only matters for sequences.
    const scene: Scene = { ...draft, steps: draft.steps.map((s) => ({ ...s, secs: sequence ? s.secs : 60 })) }
    const id = await onsave(scene)
    saving = false
    if (id) {
      draft = { ...scene, id }
      message = 'Saved.'
    }
  }

  function remove() {
    if (draft.id) ondelete(draft.id)
    startNew()
  }
</script>

<section class="page">
  <aside class="list" aria-label="Your scenes">
    <Button variant="primary" onclick={startNew}>+ New scene</Button>
    {#each scenes as s (s.id)}
      <button class="item" class:active={editing && draft.id === s.id} onclick={() => edit(s)}>
        <span class="name">{s.name}</span>
        <span class="muted">{s.steps.length > 1 ? `Sequence · ${s.steps.length} steps` : 'Preset'}</span>
      </button>
    {:else}
      <p class="muted">No scenes yet. Build one, or make a preset from one effect.</p>
    {/each}
  </aside>

  <div class="editor">
    {#if editing}
      <form onsubmit={save}>
        <h1>{draft.id ? draft.name || 'Untitled scene' : 'New scene'}</h1>

        <label class="field">
          <span class="muted">Name</span>
          <input bind:value={draft.name} maxlength="40" placeholder="e.g. Evening wind-down" />
        </label>

        <div class="steps">
          {#each draft.steps as step, i (i)}
            <div class="step">
              <span class="muted num">{i + 1}</span>
              <select aria-label="Effect for step {i + 1}" bind:value={step.effect}>
                {#each effects as e (e.id)}
                  <option value={e.id}>{e.name}</option>
                {/each}
              </select>
              {#if sequence}
                <label class="speed">
                  <span class="muted">Seconds</span>
                  <input type="number" min="1" max="600" step="1" bind:value={step.secs} />
                </label>
              {/if}
              <Button variant="ghost" size="sm" onclick={() => removeStep(i)} disabled={draft.steps.length === 1}>
                Remove
              </Button>
            </div>
          {/each}
        </div>

        <div class="row">
          <Button size="sm" onclick={addStep} disabled={draft.steps.length >= MAX_STEPS}>+ Add step</Button>
          {#if sequence}
            <label class="check"><input type="checkbox" bind:checked={draft.repeat} /> Repeat when finished</label>
          {:else}
            <span class="muted">One step is a preset. Add steps to make a sequence.</span>
          {/if}
        </div>

        <div class="row actions">
          <Button type="submit" variant="primary" disabled={saving || invalid}>{saving ? 'Saving…' : 'Save'}</Button>
          {#if draft.id}
            <Button onclick={() => onplay(draft.id)} disabled={activeScene === draft.id}>
              {activeScene === draft.id ? 'Playing' : 'Play'}
            </Button>
            <Button variant="danger" onclick={remove}>Delete</Button>
          {/if}
          {#if message}<span class="muted">{message}</span>{/if}
        </div>
      </form>
    {:else}
      <div class="empty">
        <h1>Scene builder</h1>
        <p class="muted">Pick a scene on the left, or start a new one. Play it from the group's Scene tab or here.</p>
      </div>
    {/if}
  </div>
</section>

<style>
  .page {
    flex: 1;
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    gap: 0;
    min-height: 0;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 20px 14px;
    border-right: 1px solid var(--line);
    background: var(--surface);
    overflow-y: auto;
  }
  .item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 10px;
    border: 1px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font: inherit;
    text-align: left;
  }
  .item:hover { background: var(--raised); }
  .item.active { border-color: var(--accent); background: var(--raised); }
  .name { font-weight: 600; }
  .editor {
    overflow-y: auto;
    padding: 20px 24px;
    min-height: 0;
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 640px;
  }
  h1 { margin: 0; font-size: 20px; }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  input[type='text'], .field input, .speed input {
    padding: 6px 9px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font: inherit;
  }
  .speed input { width: 80px; }
  .steps {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .step {
    display: flex;
    align-items: flex-end;
    gap: 10px;
    padding: 8px 10px;
    border-radius: 8px;
    background: var(--surface);
    border: 1px solid var(--line);
  }
  .step select {
    flex: 1;
    padding: 6px 9px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--raised);
    color: var(--text);
    font: inherit;
  }
  .num { width: 16px; padding-bottom: 8px; }
  .speed {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 12px;
  }
  .actions { margin-top: 4px; }
  .check {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .empty { max-width: 480px; }
  .empty p { margin: 8px 0 0; }
  .muted {
    color: var(--muted);
    font-size: 12px;
  }
</style>
