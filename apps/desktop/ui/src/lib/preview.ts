import { previewMany, type Effect, type Rgb } from './api'

// One shared clock draws every subscribed preview: one backend call per frame, however many are on screen.
const FRAME_MS = 120

type Sub = { effect: Effect; cols: number; rows: number; draw: (cells: Rgb[]) => void }

const subs = new Set<Sub>()
let looping = false
// One clock for the whole app. It never restarts, so an effect that is re-subscribed does not jump back to its start.
const start = performance.now()

function loop() {
  if (subs.size === 0) {
    looping = false
    return
  }
  const list = [...subs]
  const t = (performance.now() - start) / 1000
  previewMany(
    list.map((s) => ({ effect: s.effect, cols: s.cols, rows: s.rows })),
    t,
  )
    .then((frames) => list.forEach((s, i) => subs.has(s) && s.draw(frames[i])))
    .catch(() => {
      // No backend in a plain browser tab: nothing to draw.
    })
    .finally(() => setTimeout(loop, FRAME_MS))
}

// Draws `effect` on a cols-by-rows grid of room spots until the returned function is called.
export function watch(effect: Effect, cols: number, rows: number, draw: (cells: Rgb[]) => void): () => void {
  const sub: Sub = { effect, cols, rows, draw }
  subs.add(sub)
  if (!looping) {
    looping = true
    loop()
  }
  return () => {
    subs.delete(sub)
  }
}
