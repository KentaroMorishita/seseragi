import type { Page } from "@playwright/test"

declare global {
  interface Window {
    previewReadiness: {
      listeners: Set<EventListenerOrEventListenerObject>
      hold: boolean
      held: MessageEvent[]
      cancelButton?: string
      release: (index: number, stopHolding: boolean) => void
    }
  }
}

export async function installListenerAudit(
  page: Page,
  _frameId = "html-preview"
): Promise<void> {
  await page.addInitScript(() => {
    const add = EventTarget.prototype.addEventListener
    const remove = EventTarget.prototype.removeEventListener
    let replaying = false
    const state: Window["previewReadiness"] = {
      listeners: new Set<EventListenerOrEventListenerObject>(),
      hold: false,
      held: [] as MessageEvent[],
      cancelButton: undefined as string | undefined,
      release(index: number, stopHolding: boolean) {
        if (stopHolding) state.hold = false
        const event = state.held[index]
        if (event === undefined) throw new Error("missing held frame readiness")
        replaying = true
        try {
          window.dispatchEvent(
            new MessageEvent("message", {
              data: event.data,
              source: event.source,
              origin: event.origin,
            })
          )
        } finally {
          replaying = false
        }
      },
    }
    window.previewReadiness = state
    // Delay the actual opaque frame handshake without replacing its program,
    // DOM, origin, or MessagePort transport.
    Reflect.apply(add, window, [
      "message",
      (event: MessageEvent) => {
        if (event.data?.type !== "seseragi-execution-ready" || replaying) return
        if (state.cancelButton !== undefined) {
          const id = state.cancelButton
          state.cancelButton = undefined
          document.getElementById(id)?.click()
        }
        if (state.hold) {
          event.stopImmediatePropagation()
          state.held.push(event)
        }
      },
      true,
    ])
    EventTarget.prototype.addEventListener = function (
      type,
      listener,
      options
    ) {
      if (this === window && type === "message" && listener)
        state.listeners.add(listener)
      return Reflect.apply(add, this, [type, listener, options])
    }
    EventTarget.prototype.removeEventListener = function (
      type,
      listener,
      options
    ) {
      if (this === window && type === "message" && listener)
        state.listeners.delete(listener)
      return Reflect.apply(remove, this, [type, listener, options])
    }
  })
}

export async function listenerCount(page: Page): Promise<number> {
  return page.evaluate(() => window.previewReadiness.listeners.size)
}

export async function holdPreviewReadiness(page: Page): Promise<void> {
  await page.evaluate(() => {
    window.previewReadiness.hold = true
  })
}

export async function heldPreviewCount(page: Page): Promise<number> {
  return page.evaluate(() => window.previewReadiness.held.length)
}

export async function releasePreviewReadiness(
  page: Page,
  index = 0,
  stopHolding = true
): Promise<void> {
  await page.evaluate(
    ({ index, stopHolding }) =>
      window.previewReadiness.release(index, stopHolding),
    { index, stopHolding }
  )
}
