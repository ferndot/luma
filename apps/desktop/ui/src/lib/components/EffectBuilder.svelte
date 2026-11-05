<script lang="ts">
  import { draggable } from '@neodrag/svelte'
  import { iconEffect, swatch, type Effect, type NamedEffect, type Rgb } from '../api'
  import { watch } from '../preview'
  import Button from './Button.svelte'
  import EffectIcon from './EffectIcon.svelte'
  import PlayIcon from './PlayIcon.svelte'
  import Spinner from './Spinner.svelte'

  let {
    effects,
    activeEffect,
    running,
    starting,
    onsave,
    onreset,
    ondelete,
    onplay,
    onstop,
  }: {
    effects: NamedEffect[]
    activeEffect: string | null
    running: boolean
    starting: boolean
    onsave: (effect: NamedEffect) => Promise<string | null>
    onreset: (id: string) => void
    ondelete: (id: string) => void
    onplay: (id: string) => void
    onstop: () => void
  } = $props()

  const kinds: { kind: Effect['kind']; label: string }[] = [
    { kind: 'solid', label: 'Solid' },
    { kind: 'gradient', label: 'Gradient' },
    { kind: 'band', label: 'Band' },
    { kind: 'pulse', label: 'Pulse' },
    { kind: 'path', label: 'Path' },
    { kind: 'sequence', label: 'Timeline' },
    { kind: 'microphone', label: 'Microphone' },
    { kind: 'screen', label: 'Screen' },
  ]
  const BUILT_IN = ['rainbow', 'chase', 'breathe', 'solid', 'microphone', 'screen']
  const MAX_STOPS = 8
  const MAX_CUES = 12
  const COLS = 24
  const ROWS = 14

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
        stops: [{ color: [255, 0, 0], brightness: 1 }, { color: [0, 0, 255], brightness: 1 }],
        speed: 0.25,
        orbit: 'line',
      }
      case 'sequence': return { kind, steps: [{ effect: 'rainbow', secs: 10 }], repeat: true }
      case 'microphone': return { kind }
      case 'screen': return { kind }
    }
  }

  let draft = $state<NamedEffect>({ id: '', name: '', effect: defaults('solid') })
  let saving = $state(false)
  let editing = $state(false)
  let cue = $state(0)
  let isBuiltIn = $derived(BUILT_IN.includes(draft.id))
  let invalid = $derived(draft.name.trim() === '')
  // A timeline can use any effect in the library except timelines, and itself.
  let playable = $derived(effects.filter((e) => e.effect.kind !== 'sequence' && e.id !== draft.id))

  const hex = (c: Rgb) => '#' + c.map((v) => v.toString(16).padStart(2, '0')).join('')
  const rgb = (h: string): Rgb => [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16)]
  const find = (id: string) => effects.find((e) => e.id === id)
  const colorOf = (id: string) => (find(id) ? swatch(find(id)!.effect) : '#9ca3af')
  const nameOf = (id: string) => find(id)?.name ?? 'Missing'

  function edit(e: NamedEffect) {
    // $state.snapshot: the list item is a reactive proxy, which structuredClone cannot copy.
    draft = $state.snapshot(e)
    // Older paths have no palette: their point colors become the palette.
    if (draft.effect.kind === 'path' && !draft.effect.stops) {
      draft.effect.stops = draft.effect.points.map((p) => ({ color: p.color, brightness: p.brightness }))
    }
    editing = true
    cue = 0
  }

  function startNew() {
    draft = { id: '', name: '', effect: defaults('solid') }
    editing = true
  }

  function setKind(kind: Effect['kind']) {
    draft.effect = defaults(kind)
    cue = 0
    sel = null
  }

  // The room preview: a grid of spots colored by the effect, redrawn about ten times a second.
  // A timeline previews its first step; live sources show black (they only play on the lights).
  let frame = $state<Rgb[]>([])
  let shownEffect = $derived.by(() => {
    const e = draft.effect
    return e.kind === 'sequence' ? (find(e.steps[0]?.effect ?? '')?.effect ?? null) : e
  })
  $effect(() => {
    const e = shownEffect
    if (!editing || !e || e.kind === 'sequence') {
      frame = []
      return
    }
    return watch(e, COLS, ROWS, (c) => (frame = c))
  })

  // Path editor: points are dragged on the room itself. The room's size on screen maps pointer to room position.
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

  // The palette of a path: its colors, spread evenly along the route.
  let stopSel = $state(0)

  function addStop() {
    if (draft.effect.kind !== 'path' || draft.effect.stops.length >= MAX_STOPS) return
    draft.effect.stops.push({ color: [255, 255, 255], brightness: 1 })
    stopSel = draft.effect.stops.length - 1
  }

  function removeStop(i: number) {
    if (draft.effect.kind !== 'path' || draft.effect.stops.length <= 1) return
    draft.effect.stops.splice(i, 1)
    stopSel = Math.min(stopSel, draft.effect.stops.length - 1)
  }

  function removePoint(i: number) {
    if (draft.effect.kind !== 'path' || draft.effect.points.length <= 2) return
    draft.effect.points.splice(i, 1)
    sel = null
  }

  function addCue() {
    if (draft.effect.kind !== 'sequence' || draft.effect.steps.length >= MAX_CUES) return
    draft.effect.steps.push({ effect: playable[0]?.id ?? 'solid', secs: 10 })
    cue = draft.effect.steps.length - 1
  }

  function removeCue(i: number) {
    if (draft.effect.kind !== 'sequence' || draft.effect.steps.length <= 1) return
    draft.effect.steps.splice(i, 1)
    cue = Math.min(cue, draft.effect.steps.length - 1)
  }

  async function save(e: SubmitEvent) {
    e.preventDefault()
    saving = true
    const id = await onsave($state.snapshot(draft))
    saving = false
    if (id) draft = { ...draft, id }
  }

  function reset() {
    onreset(draft.id)
    const original = effects.find((x) => x.id === draft.id)
    if (original) draft = $state.snapshot(original)
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
        <EffectIcon spec={iconEffect(e.effect, effects)} />

        <span class="name">{e.name}</span>
        {#if activeEffect === e.id}<span class="muted">playing</span>{/if}
      </button>
    {/each}
  </aside>

  <div class="editor">
    {#if editing}
      <form onsubmit={save}>
        <div class="header">
          <input class="title-input" bind:value={draft.name} maxlength="40" placeholder="Name this effect" aria-label="Effect name" />
          <div class="row actions">
            <Button type="submit" variant="primary" disabled={saving || invalid}>{saving ? 'Saving…' : 'Save'}</Button>
            {#if draft.id && activeEffect === draft.id && starting}
              <Button disabled><Spinner size={12} /> Starting</Button>
            {:else if draft.id && activeEffect === draft.id && running}
              <Button onclick={onstop}><PlayIcon kind="stop" size={12} /> Stop</Button>
            {:else if draft.id}
              <Button onclick={() => onplay(draft.id)}><PlayIcon kind="play" size={12} /> Play on lights</Button>
            {/if}
            {#if isBuiltIn}
              <Button onclick={reset}>Reset</Button>
            {:else if draft.id}
              <Button variant="danger" onclick={remove}>Delete</Button>
            {/if}
          </div>
        </div>
        {#if isBuiltIn}<p class="muted">Built in. Edit it freely, or reset it to how it started.</p>{/if}

        <div class="kinds" role="tablist" aria-label="Kind">
          {#each kinds as k (k.kind)}
            <button
              type="button"
              role="tab"
              class="kind"
              class:active={draft.effect.kind === k.kind}
              aria-selected={draft.effect.kind === k.kind}
              onclick={() => setKind(k.kind)}
            >{k.label}</button>
          {/each}
        </div>

        <div
          class="room"
          bind:this={room}
          bind:clientWidth={roomW}
          bind:clientHeight={roomH}
          role="application"
          aria-label="Room preview"
          onclick={(e) => draft.effect.kind === 'path' && e.target === e.currentTarget && addPoint(e)}
        >
          <div class="cells" aria-hidden="true" style="grid-template-columns: repeat({COLS}, 1fr)">
            {#each frame as c, i (i)}
              <div style="background: rgb({c[0]}, {c[1]}, {c[2]})"></div>
            {/each}
          </div>
          {#if draft.effect.kind === 'microphone' || draft.effect.kind === 'screen'}
            <p class="live">Shows on the lights when played.</p>
          {/if}
          {#if draft.effect.kind === 'path'}
            {@const pa = draft.effect}
            {#if pa.orbit !== 'line'}
              <svg class="dir" viewBox="0 0 1 1" preserveAspectRatio="none" aria-hidden="true">
                <ellipse cx="0.5" cy="0.5" rx="0.42" ry="0.42" fill="none" stroke="currentColor" stroke-width="2" stroke-dasharray="6 6" vector-effect="non-scaling-stroke" />
              </svg>
            {:else if pa.points.length >= 2}
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
                use:draggable={{
                  position: { x: p.at[0] * roomW, y: p.at[1] * roomH },
                  bounds: 'parent',
                  onDragStart: () => (sel = i),
                  onDrag: ({ event }) => movePoint(i, event),
                }}
              ></div>
            {/each}
          {/if}
        </div>

        {#if draft.effect.kind === 'solid'}
          {@const s = draft.effect}
          <div class="controls">
            <label class="inline"><span>Color</span>
              <input type="color" value={hex(s.color)} oninput={(e) => (s.color = rgb(e.currentTarget.value))} />
            </label>
          </div>
        {:else if draft.effect.kind === 'gradient'}
          {@const g = draft.effect}
          <div class="controls">
            <div class="palette" role="group" aria-label="Colors">
              {#each g.stops as stop, i (i)}
                <span class="chip">
                  <input type="color" value={hex(stop)} aria-label="Color {i + 1}" oninput={(e) => (g.stops[i] = rgb(e.currentTarget.value))} />
                  <button type="button" class="x" aria-label="Remove color {i + 1}" onclick={() => g.stops.splice(i, 1)} disabled={g.stops.length === 1}>×</button>
                </span>
              {/each}
              <button type="button" class="add" aria-label="Add color" onclick={() => g.stops.push([255, 255, 255])} disabled={g.stops.length >= MAX_STOPS}>+</button>
            </div>
            <label class="slider"><span>Length</span><input type="range" min="0.05" max="4" step="0.05" bind:value={g.length} /><output>{g.length.toFixed(2)}</output></label>
            <label class="slider"><span>Speed</span><input type="range" min="-4" max="4" step="0.05" bind:value={g.speed} /><output>{g.speed.toFixed(2)}</output></label>
          </div>
        {:else if draft.effect.kind === 'band'}
          {@const b = draft.effect}
          <div class="controls">
            <label class="inline"><span>Band</span><input type="color" value={hex(b.color)} oninput={(e) => (b.color = rgb(e.currentTarget.value))} /></label>
            <label class="inline"><span>Background</span><input type="color" value={hex(b.background)} oninput={(e) => (b.background = rgb(e.currentTarget.value))} /></label>
            <label class="slider"><span>Width</span><input type="range" min="0.02" max="1" step="0.01" bind:value={b.width} /><output>{Math.round(b.width * 100)}%</output></label>
            <label class="slider"><span>Seconds per crossing</span><input type="range" min="0.5" max="60" step="0.5" bind:value={b.period} /><output>{b.period}s</output></label>
          </div>
        {:else if draft.effect.kind === 'pulse'}
          {@const p = draft.effect}
          <div class="controls">
            <label class="inline"><span>From</span><input type="color" value={hex(p.from)} oninput={(e) => (p.from = rgb(e.currentTarget.value))} /></label>
            <label class="inline"><span>To</span><input type="color" value={hex(p.to)} oninput={(e) => (p.to = rgb(e.currentTarget.value))} /></label>
            <label class="slider"><span>Seconds per fade</span><input type="range" min="0.5" max="60" step="0.5" bind:value={p.period} /><output>{p.period}s</output></label>
          </div>
        {:else if draft.effect.kind === 'path'}
          {@const pa = draft.effect}
          <div class="controls">
            <div class="row">
              <label class="inline"><span>Shape</span>
                <select bind:value={pa.orbit} aria-label="Shape">
                  <option value="line">Line across the room</option>
                  <option value="clockwise">Ring, clockwise</option>
                  <option value="counterclockwise">Ring, counter-clockwise</option>
                </select>
              </label>
            </div>
            <p class="muted">Drag the points to shape the route. Colors flow along it, evenly spaced, so they move with the route. Click the room to add a point.</p>
            <div class="palette" role="group" aria-label="Colors">
              {#each pa.stops as stop, i (i)}
                <span class="chip" class:sel={stopSel === i}>
                  <input type="color" value={hex(stop.color)} aria-label="Color {i + 1}" onfocus={() => (stopSel = i)} oninput={(e) => (pa.stops[i].color = rgb(e.currentTarget.value))} />
                  <button type="button" class="x" aria-label="Remove color {i + 1}" onclick={() => removeStop(i)} disabled={pa.stops.length <= 1}>×</button>
                </span>
              {/each}
              <button type="button" class="add" aria-label="Add color" onclick={addStop} disabled={pa.stops.length >= MAX_STOPS}>+</button>
            </div>
            {#if pa.stops[stopSel]}
              {@const s = pa.stops[stopSel]}
              <label class="slider"><span>Brightness of color {stopSel + 1}</span><input type="range" min="0" max="1" step="0.05" bind:value={s.brightness} /><output>{Math.round(s.brightness * 100)}%</output></label>
            {/if}
            {#if sel !== null && pa.points[sel]}
              <div class="row">
                <span>Point {sel + 1}</span>
                <Button variant="ghost" size="sm" onclick={() => removePoint(sel!)} disabled={pa.points.length <= 2}>Remove point</Button>
              </div>
            {/if}
            <label class="slider"><span>Speed</span><input type="range" min="-4" max="4" step="0.05" bind:value={pa.speed} /><output>{pa.speed.toFixed(2)}</output></label>
          </div>
        {:else if draft.effect.kind === 'sequence'}
          {@const sq = draft.effect}
          {@const current = sq.steps[cue]}
          <div class="controls">
            <p class="muted">Each block plays its effect for its length, one after another.</p>
            <div class="timeline" role="listbox" aria-label="Timeline">
              {#each sq.steps as s, i (i)}
                <button
                  type="button"
                  role="option"
                  aria-selected={cue === i}
                  class="cue"
                  class:sel={cue === i}
                  style="flex: {Math.max(s.secs, 1)} 1 0; background: {colorOf(s.effect)}"
                  onclick={() => (cue = i)}
                >
                  <span>{nameOf(s.effect)}</span><small>{s.secs}s</small>
                </button>
              {/each}
            </div>
            {#if current}
              <div class="row">
                <select value={current.effect} onchange={(e) => (current.effect = e.currentTarget.value)} aria-label="Cue effect">
                  {#each playable as p (p.id)}
                    <option value={p.id}>{p.name}</option>
                  {/each}
                </select>
                <label class="inline"><span>Seconds</span><input type="number" min="1" max="600" step="0.5" bind:value={current.secs} /></label>
                <Button variant="ghost" size="sm" onclick={() => removeCue(cue)} disabled={sq.steps.length <= 1}>Remove</Button>
              </div>
            {/if}
            <div class="row">
              <Button size="sm" onclick={addCue} disabled={sq.steps.length >= MAX_CUES}>+ Add block</Button>
              <label class="inline"><input type="checkbox" bind:checked={sq.repeat} /><span>Repeat</span></label>
            </div>
          </div>
        {/if}

      </form>
    {:else}
      <div class="empty">
        <h1>Effects</h1>
        <p class="muted">Pick an effect on the left to edit it, or make a new one. A timeline plays other effects one after another.</p>
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
    gap: 14px;
    max-width: 720px;
  }
  .header {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
    position: sticky;
    top: -20px;
    z-index: 1;
    margin: -20px 0 0;
    padding: 20px 0 10px;
    background: var(--bg);
  }
  .header .actions {
    margin: 0;
  }
  .title-input {
    flex: 1;
    min-width: 200px;
    padding: 4px 2px;
    border: none;
    border-bottom: 1px solid var(--line);
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: 22px;
    font-weight: 700;
  }
  .kinds {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .kind {
    padding: 5px 11px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: transparent;
    color: var(--muted);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .kind.active {
    border-color: var(--accent);
    background: var(--raised);
    color: var(--text);
  }
  .room {
    position: relative;
    width: 100%;
    height: 240px;
    border-radius: 10px;
    background: #050505;
    overflow: hidden;
    touch-action: none;
  }
  .cells {
    position: absolute;
    inset: 0;
    display: grid;
    pointer-events: none;
  }
  .live, .dir {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }
  .live {
    margin: 0;
    display: grid;
    place-items: center;
    color: var(--muted);
    font-size: 13px;
  }
  .dir {
    width: 100%;
    height: 100%;
    color: rgba(255, 255, 255, 0.7);
  }
  .pt {
    position: absolute;
    left: 0;
    top: 0;
    width: 18px;
    height: 18px;
    margin: -9px 0 0 -9px;
    border: 2px solid #fff;
    border-radius: 50%;
    cursor: grab;
  }
  .pt.sel { box-shadow: 0 0 0 3px var(--accent); }
  .controls {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .controls p { margin: 0; }
  .palette, .row, .actions {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }
  .chip .x, .add {
    border: 1px solid var(--line);
    border-radius: 6px;
    background: transparent;
    color: var(--muted);
    font: inherit;
    line-height: 1;
    padding: 3px 7px;
    cursor: pointer;
  }
  .add {
    width: 32px;
    height: 32px;
    font-size: 18px;
  }
  .slider {
    display: grid;
    grid-template-columns: 150px minmax(0, 1fr) 70px;
    align-items: center;
    gap: 10px;
    font-size: 13px;
  }
  .slider output {
    color: var(--muted);
    text-align: right;
  }
  .inline {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }
  input[type='color'] {
    width: 48px;
    height: 30px;
    padding: 2px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--surface);
  }
  input[type='number'], select {
    padding: 5px 8px;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font: inherit;
  }
  input[type='number'] { width: 90px; }
  .timeline {
    display: flex;
    gap: 3px;
    height: 56px;
  }
  .cue {
    min-width: 40px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: flex-start;
    gap: 2px;
    padding: 6px 8px;
    border: 2px solid transparent;
    border-radius: 6px;
    color: #111;
    font: inherit;
    font-size: 12px;
    font-weight: 600;
    text-align: left;
    overflow: hidden;
    cursor: pointer;
  }
  .cue small { font-weight: 500; opacity: 0.8; }
  .cue.sel { border-color: var(--text); }
  .actions { margin-top: 4px; }
  .empty { max-width: 480px; }
  .empty h1 { margin: 0; font-size: 20px; }
  .empty p { margin: 8px 0 0; }
  h1 { margin: 0; font-size: 20px; }
  .muted {
    color: var(--muted);
    font-size: 12px;
  }
</style>
