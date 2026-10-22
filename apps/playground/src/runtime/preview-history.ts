import type { BrowserNavigationHost } from "../../../../runtime/ts/src/browser/provider-navigation"

/** Preview-private history with traversal notifications on a later task, as
 * with popstate. It never changes the editor's location or browser history. */
export function createPreviewHistory(
  initialHref: string
): BrowserNavigationHost & { dispose: () => void } {
  const entries = [initialHref]
  let position = 0
  let disposed = false
  const listeners = new Set<() => void>()
  const timers = new Set<ReturnType<typeof setTimeout>>()
  const traverse = (delta: number) => {
    if (disposed) return
    const timer = setTimeout(() => {
      timers.delete(timer)
      const next = position + delta
      if (next < 0 || next >= entries.length || disposed) return
      position = next
      for (const listener of listeners) listener()
    }, 0)
    timers.add(timer)
  }
  return {
    currentHref: () => entries[position]!,
    push: (href) => {
      entries.splice(++position)
      entries.push(href)
    },
    replace: (href) => {
      entries[position] = href
    },
    back: () => traverse(-1),
    forward: () => traverse(1),
    listen: (listener) => {
      listeners.add(listener)
      return () => {
        listeners.delete(listener)
      }
    },
    dispose: () => {
      disposed = true
      for (const timer of timers) clearTimeout(timer)
      timers.clear()
      listeners.clear()
    },
  }
}
