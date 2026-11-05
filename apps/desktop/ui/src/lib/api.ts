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

export type SceneStep = { effect: string; secs: number }

// One step is a preset; several steps play in order as a sequence.
export type Scene = { id: string; name: string; steps: SceneStep[]; repeat: boolean }

// An effect is data: a kind and the numbers it uses. Colors are [r, g, b].
export type Effect =
  | { kind: 'solid'; color: Rgb }
  | { kind: 'gradient'; stops: Rgb[]; length: number; speed: number }
  | { kind: 'band'; color: Rgb; background: Rgb; width: number; period: number }
  | { kind: 'pulse'; from: Rgb; to: Rgb; period: number }
  | { kind: 'microphone' }
  | { kind: 'screen' }

export type NamedEffect = { id: string; name: string; effect: Effect }

// A color that stands for an effect in lists.
export function swatch(e: Effect): string {
  const css = (c: Rgb) => `rgb(${c[0]}, ${c[1]}, ${c[2]})`
  switch (e.kind) {
    case 'solid': return css(e.color)
    case 'gradient': return css(e.stops[0] ?? [255, 255, 255])
    case 'band': return css(e.color)
    case 'pulse': return css(e.to)
    default: return '#9ca3af'
  }
}

export type AppState = {
  running: boolean
  scenes: Scene[]
  active_scene: string | null
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
// A new scene has an empty id; the backend assigns one and returns it.
export const saveScene = (scene: Scene) => invoke<string>('save_scene', { scene })
export const deleteScene = (id: string) => invoke<void>('delete_scene', { id })
export const playScene = (id: string) => invoke<void>('play_scene', { id })
