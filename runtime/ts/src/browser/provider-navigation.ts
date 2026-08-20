import { providerRuntimeAbi, withProviderCancellation } from "../provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
} from "../provider-package"

export type BrowserNavigationHost = Readonly<{
  currentHref: () => string
  push: (href: string) => void
  replace: (href: string) => void
  back: () => void
  forward: () => void
  listen: (listener: () => void) => () => void
}>

export function createWindowNavigationHost(
  windowHost?: Window
): BrowserNavigationHost {
  const live = (): Window => {
    const selected = windowHost ?? globalThis.window
    if (selected === undefined) {
      throw new Error("browser navigation host is unavailable")
    }
    return selected
  }
  return Object.freeze({
    currentHref: () => live().location.href,
    push: (href) => live().history.pushState(null, "", href),
    replace: (href) => live().history.replaceState(null, "", href),
    back: () => live().history.back(),
    forward: () => live().history.forward(),
    listen: (listener) => {
      const selected = live()
      selected.addEventListener("popstate", listener)
      return () => selected.removeEventListener("popstate", listener)
    },
  })
}

export function createBrowserNavigationProvider(
  host: BrowserNavigationHost = createWindowNavigationHost()
): ProviderPackageEntry {
  const waiters = new Set<{
    resolve: (value: { kind: "success"; value: string }) => void
    reject: (cause: unknown) => void
    dispose: () => void
  }>()
  const notify = (): void => {
    let href: string
    try {
      href = host.currentHref()
    } catch (cause) {
      for (const waiter of [...waiters]) {
        waiter.dispose()
        waiter.reject(cause)
      }
      return
    }
    for (const waiter of [...waiters]) {
      waiter.dispose()
      waiter.resolve({ kind: "success", value: href })
    }
  }
  let stopListening: (() => void) | undefined
  const ensureListening = (): void => {
    stopListening ??= host.listen(notify)
  }
  const navigate = (kind: "push" | "replace", value: unknown) => {
    if (typeof value !== "string") {
      return Promise.reject(new TypeError("navigation URL must be a string"))
    }
    const current = new URL(host.currentHref())
    const target = new URL(value)
    if (target.origin !== current.origin) {
      return Promise.resolve({
        kind: "failure" as const,
        failure: Object.freeze({
          tag: "CrossOriginNavigation",
          value: Object.freeze({
            expected: current.origin,
            actual: target.origin,
          }),
        }),
      })
    }
    try {
      host[kind](target.href)
      notify()
      return Promise.resolve({
        kind: "success" as const,
        value: host.currentHref(),
      })
    } catch (cause) {
      return Promise.resolve({
        kind: "failure" as const,
        failure: hostFailure(cause),
      })
    }
  }
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: "seseragi/runtime-browser#navigation",
    service: "std/web/navigation::Navigation",
    targets: ["browser"],
    operations: {
      async current() {
        try {
          return { kind: "success", value: host.currentHref() }
        } catch (cause) {
          return { kind: "failure", failure: hostFailure(cause) }
        }
      },
      push: (value) => navigate("push", value),
      replace: (value) => navigate("replace", value),
      async back() {
        try {
          host.back()
          return { kind: "success", value: undefined }
        } catch (cause) {
          return { kind: "failure", failure: hostFailure(cause) }
        }
      },
      async forward() {
        try {
          host.forward()
          return { kind: "success", value: undefined }
        } catch (cause) {
          return { kind: "failure", failure: hostFailure(cause) }
        }
      },
      nextChange() {
        ensureListening()
        let active = true
        let activeWaiter:
          | {
              resolve: (value: { kind: "success"; value: string }) => void
              reject: (cause: unknown) => void
              dispose: () => void
            }
          | undefined
        let rejectCompletion: (cause: unknown) => void = () => undefined
        const completion = new Promise<{
          kind: "success"
          value: string
        }>((resolve, reject) => {
          rejectCompletion = reject
          const waiter = {
            resolve,
            reject,
            dispose: () => {
              if (!active) return
              active = false
              waiters.delete(waiter)
            },
          }
          activeWaiter = waiter
          waiters.add(waiter)
        })
        return withProviderCancellation(completion, () => {
          if (!active) return
          activeWaiter?.dispose()
          rejectCompletion(new Error("navigation change wait cancelled"))
        })
      },
    },
    async shutdown() {
      stopListening?.()
      for (const waiter of [...waiters]) {
        waiter.dispose()
        waiter.reject(new Error("navigation provider shut down"))
      }
    },
  })
}

function hostFailure(cause: unknown) {
  const message = cause instanceof Error ? cause.message : "navigation failed"
  return Object.freeze({
    tag:
      typeof DOMException !== "undefined" &&
      cause instanceof DOMException &&
      cause.name === "SecurityError"
        ? ("NavigationSecurityFailure" as const)
        : ("NavigationUnavailable" as const),
    value: message,
  })
}

export const provider = createBrowserNavigationProvider()
