export type ImeInputCoordinator<Target extends object> = Readonly<{
  readonly start: (target: Target) => void
  readonly update: (target: Target) => void
  readonly input: (target: Target, nativeIsComposing: boolean) => boolean
  readonly end: (target: Target) => boolean
  readonly finalize: (target: Target) => boolean
  readonly commit: (target: Target) => boolean
  readonly targets: () => ReadonlyArray<Target>
  readonly busy: () => boolean
  readonly reset: () => void
}>

/**
 * Keeps browser-specific composition event order out of the public InputEvent
 * snapshot. A composition becomes one input action only after its final value
 * is stable; controlled rerenders remain deferred while the session is busy.
 */
export function createImeInputCoordinator<
  Target extends object,
>(): ImeInputCoordinator<Target> {
  const active = new Set<Target>()
  const pending = new Set<Target>()

  return Object.freeze({
    start(target: Target) {
      pending.delete(target)
      active.add(target)
    },
    update(target: Target) {
      if (!pending.has(target)) active.add(target)
    },
    input(target: Target, nativeIsComposing: boolean) {
      if (nativeIsComposing || active.has(target)) {
        active.add(target)
        return false
      }
      if (pending.has(target)) {
        return false
      }
      return true
    },
    end(target: Target) {
      const wasActive = active.delete(target)
      const wasPending = pending.has(target)
      if (!wasActive && !wasPending) return false
      pending.add(target)
      return !wasPending
    },
    finalize(target: Target) {
      return pending.delete(target)
    },
    commit(target: Target) {
      const wasBusy = active.delete(target) || pending.delete(target)
      if (wasBusy) pending.add(target)
      return wasBusy
    },
    targets() {
      return Object.freeze([...new Set([...active, ...pending])])
    },
    busy() {
      return active.size > 0 || pending.size > 0
    },
    reset() {
      active.clear()
      pending.clear()
    },
  })
}
