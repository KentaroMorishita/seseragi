import type { Page } from "@playwright/test"

export async function installListenerAudit(
  page: Page,
  frameId = "html-preview"
): Promise<void> {
  await page.addInitScript((frameId) => {
    const listeners = new Set<EventListenerOrEventListenerObject>()
    Object.assign(window, { previewListeners: listeners })
    const add = EventTarget.prototype.addEventListener
    const remove = EventTarget.prototype.removeEventListener
    EventTarget.prototype.addEventListener = function (
      type,
      listener,
      options
    ) {
      if (
        this instanceof HTMLIFrameElement &&
        this.id === frameId &&
        (type === "load" || type === "error") &&
        listener &&
        !(typeof options === "object" && options?.once)
      )
        listeners.add(listener)
      return Reflect.apply(add, this, [type, listener, options])
    }
    EventTarget.prototype.removeEventListener = function (
      type,
      listener,
      options
    ) {
      if (
        this instanceof HTMLIFrameElement &&
        this.id === frameId &&
        (type === "load" || type === "error") &&
        listener
      )
        listeners.delete(listener)
      return Reflect.apply(remove, this, [type, listener, options])
    }
  }, frameId)
}

export async function listenerCount(page: Page): Promise<number> {
  return page.evaluate(
    () =>
      (window as unknown as { previewListeners: Set<unknown> }).previewListeners
        .size
  )
}
