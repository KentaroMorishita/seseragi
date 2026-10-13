type PreviewFrame = Pick<
  HTMLIFrameElement,
  | "src"
  | "contentWindow"
  | "contentDocument"
  | "addEventListener"
  | "removeEventListener"
>

class PreviewNavigationCancellation extends DOMException {
  constructor() {
    super("Interactive preview navigation cancelled", "AbortError")
  }
}

export function previewNavigationCancelled(): DOMException {
  return new PreviewNavigationCancellation()
}

export function isPreviewNavigationCancellation(error: unknown): boolean {
  return error instanceof PreviewNavigationCancellation
}

/** Wait only for this navigation, and settle every explicit cancellation. */
export function loadPreviewDocument(
  frame: PreviewFrame,
  url: string,
  signal: AbortSignal
): Promise<Document> {
  return new Promise((resolve, reject) => {
    let settled = false
    const cleanup = (): void => {
      frame.removeEventListener("load", onLoad)
      frame.removeEventListener("error", onError)
      signal.removeEventListener("abort", onAbort)
    }
    const fail = (error: unknown): void => {
      if (settled) return
      settled = true
      cleanup()
      reject(error)
    }
    const onAbort = (): void => fail(previewNavigationCancelled())
    const onError = (): void =>
      fail(new Error("interactive preview failed to load"))
    const onLoad = (): void => {
      if (settled) return
      try {
        if (frame.contentWindow?.location.href !== url) return
        const document = frame.contentDocument
        if (document === null) {
          fail(new Error("interactive preview document is unavailable"))
          return
        }
        settled = true
        cleanup()
        resolve(document)
      } catch (error) {
        fail(error)
      }
    }
    if (signal.aborted) {
      onAbort()
      return
    }
    frame.addEventListener("load", onLoad)
    frame.addEventListener("error", onError)
    signal.addEventListener("abort", onAbort, { once: true })
    try {
      frame.src = url
    } catch (error) {
      fail(error)
    }
  })
}
