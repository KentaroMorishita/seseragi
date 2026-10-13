import { expect, test } from "bun:test"
import {
  isPreviewNavigationCancellation,
  loadPreviewDocument,
} from "../src/runtime/preview-navigation"

const url = "https://playground.test/runtime-preview.html"

test("unrelated operation AbortErrors remain genuine failures", () => {
  expect(
    isPreviewNavigationCancellation(
      new DOMException("fetch aborted", "AbortError")
    )
  ).toBe(false)
  expect(isPreviewNavigationCancellation(new Error("AbortError"))).toBe(false)
})

class PreviewFrame extends EventTarget {
  src = ""
  contentWindow = { location: { href: "about:blank" } } as Window
  contentDocument = { title: "preview" } as Document | null
  listeners = new Set<EventListenerOrEventListenerObject>()
  removals = 0

  override addEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject
  ) {
    this.listeners.add(listener)
    super.addEventListener(type, listener)
  }

  override removeEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject
  ) {
    this.removals++
    this.listeners.delete(listener)
    super.removeEventListener(type, listener)
  }

  load(href: string) {
    this.contentWindow.location.href = href
    this.dispatchEvent(new Event("load"))
  }
}

test("preview readiness requires its document and removes listeners", async () => {
  const frame = new PreviewFrame()
  const controller = new AbortController()
  const loading = loadPreviewDocument(frame, url, controller.signal)
  let settled = false
  void loading.then(() => {
    settled = true
  })
  frame.load("about:blank")
  await Promise.resolve()
  expect(settled).toBe(false)
  expect(frame.listeners.size).toBe(2)
  frame.load(url)
  expect(await loading).toBe(frame.contentDocument as Document)
  expect(frame.listeners.size).toBe(0)
  controller.abort()
})

test("cancelling a pending preview settles and a stale load cannot revive it", async () => {
  const frame = new PreviewFrame()
  const controller = new AbortController()
  const loading = loadPreviewDocument(frame, url, controller.signal)
  const outcome = loading.catch((error: unknown) => error)
  controller.abort()
  const error = await outcome
  expect(isPreviewNavigationCancellation(error)).toBe(true)
  expect(frame.listeners.size).toBe(0)
  frame.load(url)
  expect(frame.listeners.size).toBe(0)
  expect(await outcome).toBe(error)
  expect(frame.removals).toBe(2)
})

test("an already cancelled preparation never navigates the frame", async () => {
  const frame = new PreviewFrame()
  const controller = new AbortController()
  controller.abort()
  const error = await loadPreviewDocument(frame, url, controller.signal).catch(
    (error: unknown) => error
  )
  expect(isPreviewNavigationCancellation(error)).toBe(true)
  expect(frame.src).toBe("")
  expect(frame.listeners.size).toBe(0)
})

test("replacement navigation ignores the previous URL and completes normally", async () => {
  const frame = new PreviewFrame()
  const previous = new AbortController()
  const old = loadPreviewDocument(frame, `${url}?old`, previous.signal)
  const cancelled = old.catch((error: unknown) => error)
  previous.abort()
  const current = new AbortController()
  const next = loadPreviewDocument(frame, url, current.signal)
  let settled = false
  void next.then(() => {
    settled = true
  })
  frame.load(`${url}?old`)
  await Promise.resolve()
  expect(settled).toBe(false)
  expect(isPreviewNavigationCancellation(await cancelled)).toBe(true)
  frame.load(url)
  expect(await next).toBe(frame.contentDocument as Document)
  expect(frame.listeners.size).toBe(0)
})

test("preview errors reject and release every listener", async () => {
  const frame = new PreviewFrame()
  const controller = new AbortController()
  const loading = loadPreviewDocument(frame, url, controller.signal)
  frame.dispatchEvent(new Event("error"))
  await expect(loading).rejects.toThrow("failed to load")
  expect(frame.listeners.size).toBe(0)
  controller.abort()
  frame.load(url)
})

test("a matching load without a document fails explicitly", async () => {
  const frame = new PreviewFrame()
  frame.contentDocument = null
  const loading = loadPreviewDocument(frame, url, new AbortController().signal)
  frame.load(url)
  await expect(loading).rejects.toThrow("document is unavailable")
  expect(frame.listeners.size).toBe(0)
})

test("navigation and document-access exceptions reject with cleanup", async () => {
  for (const property of ["src", "contentDocument"] as const) {
    const frame = new PreviewFrame()
    Object.defineProperty(frame, property, {
      ...(property === "src"
        ? {
            set: () => {
              throw new Error("navigation denied")
            },
          }
        : {
            get: () => {
              throw new Error("document denied")
            },
          }),
    })
    const loading = loadPreviewDocument(
      frame,
      url,
      new AbortController().signal
    )
    if (property === "contentDocument") frame.load(url)
    await expect(loading).rejects.toThrow("denied")
    expect(frame.listeners.size).toBe(0)
  }
})
