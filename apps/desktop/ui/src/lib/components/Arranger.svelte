<script lang="ts">
  import type { Light } from '../api'

  let {
    members,
    positions,
    onmove,
  }: {
    members: Light[]
    positions: Record<string, [number, number]>
    onmove: (id: string, x: number, y: number) => void
  } = $props()

  let box: HTMLDivElement
  let dragId = $state('')
  // Where a device is while it is being dragged, before the saved position catches up.
  let live = $state<Record<string, [number, number]>>({})

  // Devices without a saved spot are lined up along the top until they are placed.
  function spot(l: Light, i: number): [number, number] {
    return live[l.id] ?? positions[l.id] ?? [0.15 + (i % 4) * 0.23, 0.2 + Math.floor(i / 4) * 0.3]
  }

  const clamp = (v: number) => Math.min(1, Math.max(0, v))

  function down(e: PointerEvent, id: string) {
    dragId = id
    ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  }

  function move(e: PointerEvent) {
    if (!dragId) return
    const r = box.getBoundingClientRect()
    live = { ...live, [dragId]: [clamp((e.clientX - r.left) / r.width), clamp((e.clientY - r.top) / r.height)] }
  }

  function up() {
    if (!dragId) return
    const id = dragId
    dragId = ''
    const p = live[id]
    if (p) onmove(id, p[0], p[1])
  }
</script>

<div
  class="room"
  bind:this={box}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={up}
  role="group"
  aria-label="Room layout. Drag devices to match where they are."
>
  {#if members.length === 0}
    <p class="hint">Add devices to this group, then drag them to where they are in the room.</p>
  {/if}
  {#each members as l, i (l.id)}
    {@const p = spot(l, i)}
    <div
      class="dev"
      class:dragging={dragId === l.id}
      style="left: {p[0] * 100}%; top: {p[1] * 100}%"
      onpointerdown={(e) => down(e, l.id)}
      role="presentation"
      title="Drag to place"
    >
      {l.name}
    </div>
  {/each}
</div>

<style>
  .room {
    position: relative;
    height: 240px;
    border: 1px dashed var(--line);
    border-radius: 10px;
    background: var(--surface);
    overflow: hidden;
    touch-action: none;
  }
  .hint {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    margin: 0;
    padding: 16px;
    text-align: center;
    color: var(--muted);
    font-size: 12px;
  }
  .dev {
    position: absolute;
    transform: translate(-50%, -50%);
    padding: 6px 10px;
    border: 1px solid var(--accent);
    border-radius: 6px;
    background: var(--raised);
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
    cursor: grab;
    user-select: none;
  }
  .dev.dragging {
    cursor: grabbing;
    box-shadow: 0 0 12px var(--accent);
  }
</style>
