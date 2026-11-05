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

export type AppState = {
  running: boolean
  scanning: boolean
  subnet: string
  subnets: string[]
  has_key: boolean
  key_status: string
  mode: string
  status: string
  preview: Rgb[]
  lights: Light[]
}

export const getState = () => invoke<AppState>('get_state')
export const setMode = (name: string) => invoke<void>('set_mode', { name })
export const findLights = () => invoke<void>('find_lights')
export const addSubnet = (subnet: string) => invoke<void>('add_subnet', { subnet })
export const removeSubnet = (subnet: string) => invoke<void>('remove_subnet', { subnet })
export const renameLight = (id: string, name: string) => invoke<void>('rename_light', { id, name })
export const setApiKey = (key: string) => invoke<void>('set_api_key', { key })
export const setSelected =(ips: string[]) => invoke<void>('set_selected', { ips })
export const start = () => invoke<void>('start')
export const stop = () => invoke<void>('stop')
