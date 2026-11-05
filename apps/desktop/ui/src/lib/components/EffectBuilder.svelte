<script lang="ts">
  import { draggable } from '@neodrag/svelte'
  import { swatch, type Effect, type NamedEffect, type PathPoint, type Rgb } from '../api'
  import Button from './Button.svelte'

  let {
    effects,
    activeEffect,
    onsave,
    onreset,
    ondelete,
    onplay,
  }: {
    effects: NamedEffect[]
    activeEffect: string | null
    onsave: (effect: NamedEffect) => Promise<string | null>
    onreset: (id: string) => void
    ondelete: (id: string) => void
    onplay: (id: string) => void
  } = $props()

  const kinds: { kind: Effect['kind']; label: string; about: string }[] = [
    { kind: 'solid', label: 'Solid', about: 'One color everywhere.' },
    { kind: 'gradient', label: 'Gradient', about: 'Colors blend along the room and scroll. Rainbow is one.' },
    { kind: 'band', label: 'Band', about: 'A band of color sweeps across the room. Chase is one.' },
    { kind: 'pulse', label: 'Pulse', about: 'Fades between two colors and back. Breathe is one.' },
    { kind: 'path', label: 'Path', about: 'A direction through the room. Colors and brightness set along it travel that way.' },
    { kind: 'microphone', label: 'Microphone', about: 'Brightness follows the microphone.' },
    { kind: 'screen', label: 'Screen', about: 'Each spot shows the screen color of its column.' },
  ]
  const BUILT_IN = ['rainbow', 'chase', 'breathe', 'solid', 'microphone', 'screen']
  const MAX_STOPS = 8

  function defaults(kind: Effect['kind']): Effect {
    switch (kind) {
      case 'solid': return { kind, color: [255, 255, 255] }
      case 'gradient': return { kind, stops: [[255, 0, 0], [0, 0, 255]], length: 1, speed: 0.25 }
      case 'band': return { kind, color: [255, 255, 255], background: [0, 0, 0], width: 0.16, period: 4 }
      case 'pulse': return { kind, from: [0, 0, 0], to: [255, 255, 255], period: 3 }
      case 'path': return {
        kind,
        points: [
          { at: [0.1, 0.5], color: [255, 0, 0], brightness: 1 },
          { at: [0.9, 0.5], color: [0, 0, 255], brightness: 1 },
        ],
        speed: 0.25,
      }
      case 'microphone': return { kind }
      case 'screen': return { kind }
    }
  }

  let draft = $state<NamedEffect>({ id: '', name: '', effect: defaults('solid') })
  let saving = $state(false)
  let message = $state('')
  let editing = $state(false)
  let isBuiltIn = $derived(BUILT_IN.includes(draft.id))
  let invalid = $derived(draft.name.trim() === '')
  let about = $derived(kinds.find((k) => k.kind === draft.effect.kind)?.about ?? '')

  const hex = (c: Rgb) => '#' + c.map((v) => v.toString(16).padStart(2, '0')).join('')
  const rgb = (h: string): Rgb => [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16)]

  function edit(e: NamedEffect) {
    draft = structuredClone(e)
    editing = true
    message = ''
  }

  function startNew() {
    draft = { id: '', name: '', effect: defaults('solid') }
    editing = true
    message = ''
  }

  function setKind(e: Event) {
    draft.effect = defaults((e.currentTarget as HTMLSelectElement).value as Effect['kind'])
  }

  // Path editor state: the room's size on screen, the selected point, and the room-relative pointer.
  let room: HTMLDivElement
  let roomW = $state(1)
  let roomH = $state(1)
  let sel = $state<number | null>(null)

  // Room position (0 to 1 both ways) of a pointer event.
  function spot(e: { clientX: number; clientY: number }): [number, number] {
    const r = room.getBoundingClientRect()
    const clamp = (v: number) => Math.min(1, Math.max(0, v))
    return [clamp((e.clientX - r.left) / r.width), clamp((e.clientY - r.top) / r.height)]
  }

  function addPoint(e: MouseEvent) {
    if (draft.effect.kind !== 'path' || draft.effect.points.length >= 16) return
    draft.effect.points.push({ at: spot(e), color: [255, 255, 255], brightness: 1 })
    sel = draft.effect.points.length - 1
  }

  function movePoint(i: number, e: PointerEvent) {
    if (draft.effect.kind === 'path') draft.effect.points[i].at = spot(e)
  }

  function removePoint(i: number) {
    if (draft.effect.kind !== 'path' || draft.effect.points.length <= 2) return
    draft.effect.points.splice(i, 1)
    sel = null
  }

  async function save(e: SubmitEvent) {
    e.preventDefault()
    saving = true
    const id = await onsave($state.snapshot(draft))
    saving = false
    if (id) {
      draft = { ...draft, id }
      message = 'Saved.'
    }
  }

  function reset() {
    onreset(draft.id)
    const original = effects.find((x) => x.id === draft.id)
    if (original) draft = structuredClone(original)
  }

  function remove() {
    ondelete(draft.id)
    startNew()
  }
</script>

<section class="page">
  <aside class="list" aria-label="Effects">
    <Button variant="primary" onclick={startNew}>+ New effect</Button>
    {#each effects as e (e.id)}
      <button class="item" class:active={editing && draft.id === e.id} onclick={() => edit(e)}>
        <span class="swatch" style="background: {swatch(e.effect)}" aria-hidden="true"></span>
        <span class="name">{e.name}</span>
        {#if activeEffect === e.id}<span class="muted">playing</span>{/if}
      </button>
    {/each}
  </aside>

  <div class="editor">
    {#if editing}
      <form onsubmit={save}>
        <h1>{draft.id ? draft.name || 'Untitled effect' : 'New effect'}</h1>
        {#if isBuiltIn}<p class="muted">Built in. You can edit it, or reset it to how it started.</p>{/if}

        <label class="field">
          <span class="muted">Name</span>
          <input bind:value={draft.name} maxlength="40" placeholder="e.g. Ocean" />
        </label>

        <label class="field">
          <span class="muted">Kind</span>
          <select value={draft.effect.kind} onchange={setKind}>
            {#each kinds as k (k.kind)}
              <option value={k.kind}>{k.label}</option>
            {/each}
          </select>
          <span class="muted">{about}</span>
        </label>

        {#if draft.effect.kind === 'solid'}
          <label class="field"><span class="muted">Color</span>
            <input type="color" value={hex(draft.effect.color)} oninput={(e) => (draft.effect = { kind: 'solid', color: rgb(e.currentTarget.value) })} />
          </label>
        {:else if draft.effect.kind === 'gradient'}
          {@const g = draft.effect}
          <div class="field">
            <span class="muted">Colors, blended in order and wrapped around</span>
            {#each g.stops as stop, i (i)}
              <div class="row">
                <input type="color" value={hex(stop)} aria-label="Color {i + 1}" oninput={(e) => (g.stops[i] = rgb(e.currentTarget.value))} />
                <Button variant="ghost" size="sm" onclick={() => g.stops.splice(i, 1)} disabled={g.stops.length === 1}>Remove</Button>
              </div>
            {/each}
            <div class="row">
              <Button size="sm" onclick={() => g.stops.push([255, 255, 255])} disabled={g.stops.length >= MAX_STOPS}>+ Add color</Button>
            </div>
          </div>
          <label class="field num"><span class="muted">Length (room widths per cycle)</span>
            <input type="number" min="0.05" max="4" step="0.05" bind:value={g.length} />
          </label>
          <label class="field num"><span class="muted">Speed (cycles per second, negative reverses)</span>
            <input type="number" min="-4" max="4" step="0.05" bind:value={g.speed} />
          </label>
        {:else if draft.effect.kind === 'band'}
          {@const b = draft.effect}
          <label class="field"><span class="muted">Band color</span>
            <input type="color" value={hex(b.color)} oninput={(e) => (b.color = rgb(e.currentTarget.value))} />
          </label>
          <label class="field"><span class="muted">Background</span>
            <input type="color" value={hex(b.background)} oninput={(e) => (b.background = rgb(e.currentTarget.value))} />
          </label>
          <label class="field num"><span class="muted">Width (share of the room)</span>
            <input type="number" min="0.02" max="1" step="0.01" bind:value={b.width} />
          </label>
          <label class="field num"><span class="muted">Seconds per crossing</span>
            <input type="number" min="0.5" max="60" step="0.5" bind:value={b.period} />
          </label>
        {:else if draft.effect.kind === 'pulse'}
          {@const p = draft.effect}
          <label class="field"><span class="muted">From</span>
            <input type="color" value={hex(p.from)} oninput={(e) => (p.from = rgb(e.currentTarget.value))} />
          </label>
          <label class="field"><span class="muted">To</span>
            <input type="color" value={hex(p.to)} oninput={(e) => (p.to = rgb(e.currentTarget.value))} />
          </label>
          <label class="field num"><span class="muted">Seconds per fade in and out</span>
            <input type="number" min="0.5" max="60" step="0.5" bind:value={p.period} />
          </label>
        {:else if draft.effect.kind === 'path'}
          {@const pa = draft.effect}
          <div class="field">
            <span class="muted">The arrow runs from the first point to the last: that is the direction. Colors and brightness travel along it. Click to add a point, drag to move it, pick one to set its color and brightness.</span>
            <div
              class="room"
              bind:this={room}
              bind:clientWidth={roomW}
              bind:clientHeight={roomH}
              role="application"
              aria-label="Path editor"
              onclick={(e) => e.target === e.currentTarget && addPoint(e)}
            >
              {#if pa.points.length >= 2}
                {@const a = pa.points[0].at}
                {@const b = pa.points[pa.points.length - 1].at}
                <svg class="dir" viewBox="0 0 1 1" preserveAspectRatio="none" aria-hidden="true">
                  <defs>
                    <marker id="arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="5" markerHeight="5" orient="auto-start-reverse">
                      <path d="M0 0L10 5L0 10z" fill="currentColor" />
                    </marker>
                  </defs>
                  <line x1={a[0]} y1={a[1]} x2={b[0]} y2={b[1]} stroke="currentColor" stroke-width="2" vector-effect="non-scaling-stroke" marker-end="url(#arrow)" />
                </svg>
              {/if}
              {#each pa.points as p, i (i)}
                <div
                  class="pt"
                  class:sel={sel === i}
                  role="button"
                  tabindex="-1"
                  aria-label="Point {i + 1}"
                  style="background: {hex(p.color)}; opacity: {0.3 + 0.7 * p.brightness}"
                  use:draggable={{
                    position: { x: p.at[0] * roomW, y: p.at[1] * roomH },
                    bounds: 'parent',
                    onDragStart: () => (sel = i),
                    onDrag: ({ event }) => movePoint(i, event),
                  }}
                ></div>
              {/each}
            </div>
          </div>
          {#if sel !== null && pa.points[sel]}
            {@const p = pa.points[sel]}
            <div class="row">
              <span class="muted">Point {sel + 1}</span>
              <input type="color" value={hex(p.color)} aria-label="Point color" oninput={(e) => (p.color = rgb(e.currentTarget.value))} />
              <label class="row"><span class="muted">Brightness</span>
                <input type="range" min="0" max="1" step="0.05" bind:value={p.brightness} />
              </label>
              <Button variant="ghost" size="sm" onclick={() => removePoint(sel!)} disabled={pa.points.length <= 2}>Remove point</Button>
            </div>
          {/if}
          <label class="field num"><span class="muted">Speed (lengths per second, negative reverses)</span>
            <input type="number" min="-4" max="4" step="0.05" bind:value={pa.speed} />
          </label>
        {/if}

        <div class="row actions">
          <Button type="submit" variant="primary" disabled={saving || invalid}>{saving ? 'Saving…' : 'Save'}</Button>
          {#if draft.id}
            <Button onclick={() => onplay(draft.id)} disabled={activeEffect === draft.id}>
              {activeEffect === draft.id ? 'Playing' : 'Play'}
            </Button>
          {/if}
          {#if isBuiltIn}
            <Button onclick={reset}>Reset</Button>
          {:else if draft.id}
            <Button variant="danger" onclick={remove}>Delete</Button>
          {/if}
          {#if message}<span class="muted">{message}</span>{/if}
        </div>
      </form>
    {:else}
      <div class="empty">
        <h1>Effects</h1>
        <p class="muted">Pick an effect on the left to edit it, or make a new one. Scenes are built from effects.</p>
      </div>
    {/if}
  </div>
</section>

<style>
  .page {
    flex: 1;
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
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
    align-items: center;
    gap: 10px;
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
  .swatch {
    width: 14px;
    height: 14px;
    flex: none;
    border-radius: 4px;
  }
  .name { flex: 1; font-weight: 600; }
  .editor {
    overflow-y: auto;
    padding: 20px 24px;
    min-height: 0;
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 560px;
  }
  h1 { margin: 0; font-size: 20px; }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .field.num input, input[type='text'] {
    width: 120px;
  }
  input, select {
    padding: 6px 9px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font: inherit;
  }
  input[type='color'] {
    width: 56px;
    height: 32px;
    padding: 2px;
  }
  .row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
  }
  .actions { margin-top: 4px; }
  .room {
    position: relative;
    width: 100%;
    max-width: 520px;
    height: 220px;
    border: 1px dashed var(--line);
    border-radius: 10px;
    background: var(--surface);
    overflow: hidden;
    touch-action: none;
  }
  .pt {
    position: absolute;
    left: 0;
    top: 0;
    width: 18px;
    height: 18px;
    margin: -9px 0 0 -9px;
    border: 2px solid var(--text);
    border-radius: 50%;
    cursor: grab;
  }
  .pt.sel { box-shadow: 0 0 0 3px var(--accent); }
  .dir {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    color: var(--muted);
    pointer-events: none;
  }
  .empty { max-width: 480px; }
  .empty p { margin: 8px 0 0; }
  .muted {
    color: var(--muted);
    font-size: 12px;
  }
</style>
