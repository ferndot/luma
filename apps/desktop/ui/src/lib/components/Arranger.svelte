<script lang="ts">
  import type { Light } from '../api'

  let {
    members,
    positions,
    disabled = false,
    onmove,
    onlight,
    onrelease,
  }: {
    members: Light[]
    positions: Record<string, [number, number]>
    disabled?: boolean
    onmove: (id: string, x: number, y: number) => void
    onlight: (id: string, color: [number, number, number]) => void
    onrelease: (id: string) => void
  } = $props()

  // Each device gets a color; pressing a chip lights the real device in that color.
  const palette: [number, number, number][] = [
    [255, 79, 163],
    [255, 138, 43],
    [250, 204, 21],
    [74, 222, 128],
    [61, 139, 255],
    [168, 85, 247],
  ]
  const colorOf = (i: number) => palette[i % palette.length]

  let box: HTMLDivElement
  let dragId = $state('')
  let moved = false
  // Where a device is while it is being dragged, before the saved position catches up.
  let live = $state<Record<string, [number, number]>>({})

  // Devices without a saved spot are lined up along the top until they are placed.
  function spot(l: Light, i: number): [number, number] {
    return live[l.id] ?? positions[l.id] ?? [0.15 + (i % 4) * 0.23, 0.2 + Math.floor(i / 4) * 0.3]
  }

  const clamp = (v: number) => Math.min(1, Math.max(0, v))

  // Pressing lights the device right away and keeps it lit while the pointer is down.
  function down(e: PointerEvent, id: string) {
    if (disabled) return
    dragId = id
    moved = false
    const i = members.findIndex((l) => l.id === id)
    if (i >= 0) onlight(id, colorOf(i))
    ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  }

  function move(e: PointerEvent) {
    if (!dragId || disabled) return
    moved = true
    const r = box.getBoundingClientRect()
    live = { ...live, [dragId]: [clamp((e.clientX - r.left) / r.width), clamp((e.clientY - r.top) / r.height)] }
  }

  function up() {
    if (!dragId) return
    const id = dragId
    dragId = ''
    onrelease(id)
    const p = live[id]
    if (moved && p) onmove(id, p[0], p[1])
  }
</script>

<div
  class="room"
  bind:this={box}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={up}
  role="group"
  aria-label="Room layout. Press a device to light it, drag to match where it is in the room."
>
  {#if members.length === 0}
    <p class="hint">Add devices to this group, then drag them to where they are in the room.</p>
  {/if}
  {#each members as l, i (l.id)}
    {@const p = spot(l, i)}
    {@const c = colorOf(i)}
    <div
      class="dev"
      class:dragging={dragId === l.id}
      style="left: {p[0] * 100}%; top: {p[1] * 100}%; --c: rgb({c[0]}, {c[1]}, {c[2]})"
      onpointerdown={(e) => down(e, l.id)}
      role="button"
      tabindex="-1"
      aria-label="{l.name}. Press to light it, drag to move it."
      title="Press to light it, drag to move"
    >
      {l.name}
    </div>
  {/each}
</div>

<style>
  .room {
    position: relative;
    flex: 1;
    min-height: 240px;
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
    border: 2px solid var(--c);
    border-radius: 6px;
    background: var(--raised);
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
    cursor: grab;
    user-select: none;
    box-shadow: 0 0 10px color-mix(in srgb, var(--c) 35%, transparent);
  }
  .dev.dragging {
    cursor: grabbing;
    box-shadow: 0 0 16px var(--c);
  }
</style>
