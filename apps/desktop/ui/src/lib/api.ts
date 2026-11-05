import { invoke as tauriInvoke } from '@tauri-apps/api/core'

// Outside the desktop window (for example a plain browser tab), there is no backend to call.
const inTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!inTauri) {
    return Promise.reject(new Error('Open Luma as the desktop app (run `pnpm tauri dev`), not in a browser tab.'))
  }
  return tauriInvoke<T>(cmd, args)
}

export type Rgb = [number, number, number]

export type Light = {
  id: string
  name: string
  ip: string
  sku: string
  razer: boolean
  segments: number
  selected: boolean
}

export type Group = {
  id: string
  name: string
  members: string[]
  positions: Record<string, [number, number]>
}

// An effect is data: a kind and the numbers it uses. Colors are [r, g, b].
export type Effect =
  | { kind: 'solid'; color: Rgb }
  | { kind: 'gradient'; stops: Rgb[]; length: number; speed: number }
  | { kind: 'band'; color: Rgb; background: Rgb; width: number; period: number }
  | { kind: 'pulse'; from: Rgb; to: Rgb; period: number }
  | { kind: 'path'; points: PathPoint[]; stops: Stop[]; speed: number; orbit: Orbit }
  | { kind: 'microphone' }
  | { kind: 'screen' }
  | { kind: 'sequence'; steps: SequenceStep[]; repeat: boolean }

// A step names a library effect by id and how long it runs.
export type SequenceStep = { effect: string; secs: number }

// A path runs as a line across the room, or as a ring around its center, clockwise or counter-clockwise.
export type Orbit = 'line' | 'clockwise' | 'counterclockwise'

// One color of a path's palette, and its brightness (0 to 1). Colors spread evenly along the route.
export type Stop = { color: Rgb; brightness: number }

// A point that shapes a path's route: where it is in the room (0 to 1 both ways). Its color and brightness are a fallback for old paths.
export type PathPoint = { at: [number, number]; color: Rgb; brightness: number }

export type NamedEffect = { id: string; name: string; effect: Effect }

// A color that stands for an effect in lists.
export function swatch(e: Effect): string {
  const css = (c: Rgb) => `rgb(${c[0]}, ${c[1]}, ${c[2]})`
  switch (e.kind) {
    case 'solid': return css(e.color)
    case 'gradient': return css(e.stops[0] ?? [255, 255, 255])
    case 'band': return css(e.color)
    case 'pulse': return css(e.to)
    case 'path': return css(e.stops?.[0]?.color ?? e.points[0]?.color ?? [255, 255, 255])
    case 'sequence': return '#e879f9'
    default: return '#9ca3af'
  }
}

export type AppState = {
  running: boolean
  effects: NamedEffect[]
  active_effect: string | null
  scanning: boolean
  subnet: string
  subnets: string[]
  has_key: boolean
  key_status: string
  groups: Group[]
  status: string
  preview: Rgb[]
  positions: [number, number][]
  lights: Light[]
}

export const getState = () => invoke<AppState>('get_state')
export const saveEffect = (effect: NamedEffect) => invoke<string>('save_effect', { effect })
export const resetEffect = (id: string) => invoke<void>('reset_effect', { id })
export const deleteEffect = (id: string) => invoke<void>('delete_effect', { id })
export const playEffect = (id: string) => invoke<void>('play_effect', { id })
// Colors of several grids of room spots (row by row), one grid per request, at time t seconds.
export const previewMany = (requests: { effect: Effect; cols: number; rows: number }[], t: number) =>
  invoke<Rgb[][]>('preview_many', { requests, t })

// The effect a list icon or preview shows: a timeline shows its first block.
export function iconEffect(e: Effect, library: NamedEffect[]): Effect {
  if (e.kind !== 'sequence') return e
  const first = library.find((x) => x.id === e.steps[0]?.effect)?.effect
  return first && first.kind !== 'sequence' ? first : { kind: 'solid', color: [0, 0, 0] }
}
export const findLights = () => invoke<void>('find_lights')
export const addSubnet = (subnet: string) => invoke<void>('add_subnet', { subnet })
export const removeSubnet = (subnet: string) => invoke<void>('remove_subnet', { subnet })
export const createGroup = (name: string) => invoke<string>('create_group', { name })
export const renameGroup = (id: string, name: string) => invoke<void>('rename_group', { id, name })
export const setGroupMembers = (id: string, members: string[]) =>
  invoke<void>('set_group_members', { id, members })
export const setGroupPosition = (id: string, device: string, x: number, y: number) =>
  invoke<void>('set_group_position', { id, device, x, y })
export const lightDevice = (group: string, device: string, color: [number, number, number]) =>
  invoke<void>('light_device', { group, device, color })
export const releaseDevice = (device: string) => invoke<void>('release_device', { device })
export const startGroup =(id: string) => invoke<void>('start_group', { id })
export const deleteGroup = (id: string) => invoke<void>('delete_group', { id })
export const setApiKey = (key: string) => invoke<void>('set_api_key', { key })
export const stop = () => invoke<void>('stop')
