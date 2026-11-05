import { invoke } from '@tauri-apps/api/core'

export type Rgb = [number, number, number]

export type Light = {
  ip: string
  sku: string
  razer: boolean
  segments: number
  selected: boolean
}

export type AppState = {
  running: boolean
  mode: string
  status: string
  preview: Rgb[]
  lights: Light[]
}

export const getState = () => invoke<AppState>('get_state')
export const setMode = (name: string) => invoke<void>('set_mode', { name })
export const findLights = (subnet: string) => invoke<void>('find_lights', { subnet })
export const setSelected = (ips: string[]) => invoke<void>('set_selected', { ips })
export const start = () => invoke<void>('start')
export const stop = () => invoke<void>('stop')
