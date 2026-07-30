import { describe, expect, test } from "bun:test"
import { connectPreviewFullscreen } from "../src/ui/preview-fullscreen"

class FakeClassList {
  readonly values = new Set<string>()

  add(value: string): void {
    this.values.add(value)
  }

  toggle(value: string, force?: boolean): boolean {
    const enabled = force ?? !this.values.has(value)
    if (enabled) this.values.add(value)
    else this.values.delete(value)
    return enabled
  }

  contains(value: string): boolean {
    return this.values.has(value)
  }
}

class FakeDocument extends EventTarget {
  readonly body = { classList: new FakeClassList() }
  fullscreenElement: EventTarget | null = null

  async exitFullscreen(): Promise<void> {
    this.fullscreenElement = null
    this.dispatchEvent(new Event("fullscreenchange"))
  }
}

class FakeSurface extends EventTarget {
  readonly classList = new FakeClassList()
  readonly dataset: Record<string, string | undefined> = {}
  readonly previewIdentity = {}
  requestFullscreen?: () => Promise<void>

  constructor(readonly ownerDocument: FakeDocument) {
    super()
  }
}

class FakeButton extends EventTarget {
  readonly attributes = new Map<string, string>()
  textContent = ""
  title = ""

  setAttribute(name: string, value: string): void {
    this.attributes.set(name, value)
  }

  removeAttribute(name: string): void {
    this.attributes.delete(name)
    if (name === "title") this.title = ""
  }
}

function connect(
  setup?: (surface: FakeSurface, ownerDocument: FakeDocument) => void
) {
  const ownerDocument = new FakeDocument()
  const surface = new FakeSurface(ownerDocument)
  const button = new FakeButton()
  setup?.(surface, ownerDocument)
  const controller = connectPreviewFullscreen(
    surface as unknown as HTMLElement,
    button as unknown as HTMLButtonElement
  )
  return { button, controller, ownerDocument, surface }
}

describe("Preview fullscreen controller", () => {
  test("uses a reversible fallback without replacing preview state", async () => {
    const { button, controller, ownerDocument, surface } = connect()
    const previewIdentity = surface.previewIdentity

    await controller.toggle()

    expect(controller.mode()).toBe("fallback")
    expect(surface.dataset.previewFullscreen).toBe("fallback")
    expect(surface.dataset.previewFullscreenFallback).toBe("unsupported")
    expect(
      ownerDocument.body.classList.contains("preview-fullscreen-fallback-open")
    ).toBe(true)
    expect(button.attributes.get("aria-pressed")).toBe("true")
    expect(button.textContent).toBe("Close")
    expect(surface.previewIdentity).toBe(previewIdentity)

    await controller.toggle()

    expect(controller.mode()).toBeUndefined()
    expect(surface.dataset.previewFullscreen).toBeUndefined()
    expect(
      ownerDocument.body.classList.contains("preview-fullscreen-fallback-open")
    ).toBe(false)
    expect(button.textContent).toBe("Full screen")

    await controller.toggle()
    const escapeEvent = new Event("keydown", { cancelable: true })
    Object.defineProperty(escapeEvent, "key", { value: "Escape" })
    ownerDocument.dispatchEvent(escapeEvent)

    expect(escapeEvent.defaultPrevented).toBe(true)
    expect(controller.mode()).toBeUndefined()
    expect(surface.previewIdentity).toBe(previewIdentity)
  })

  test("falls back when the native request is rejected", async () => {
    const { controller, surface } = connect((target) => {
      target.requestFullscreen = async () => {
        throw new Error("Fullscreen denied")
      }
    })

    await controller.toggle()

    expect(controller.mode()).toBe("fallback")
    expect(surface.dataset.previewFullscreenFallback).toBe("rejected")
  })

  test("tracks native entry, explicit exit, and browser-initiated exit", async () => {
    const { controller, ownerDocument, surface } = connect(
      (target, document) => {
        target.requestFullscreen = async () => {
          document.fullscreenElement = target
          document.dispatchEvent(new Event("fullscreenchange"))
        }
      }
    )

    await controller.toggle()
    expect(controller.mode()).toBe("native")
    expect(surface.dataset.previewFullscreen).toBe("native")

    await controller.toggle()
    expect(controller.mode()).toBeUndefined()

    await controller.toggle()
    ownerDocument.fullscreenElement = null
    ownerDocument.dispatchEvent(new Event("fullscreenchange"))
    expect(controller.mode()).toBeUndefined()
  })
})
