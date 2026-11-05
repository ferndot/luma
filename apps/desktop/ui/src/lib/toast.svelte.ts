// Short notices that appear and fade on their own: saved, reset, deleted.
export type Toast = { id: number; text: string }

export const toasts = $state<Toast[]>([])

let next = 0

export function toast(text: string, ms = 2600) {
  const id = ++next
  toasts.push({ id, text })
  setTimeout(() => {
    const i = toasts.findIndex((t) => t.id === id)
    if (i >= 0) toasts.splice(i, 1)
  }, ms)
}
