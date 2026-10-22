import {
  createBrowserStorageProvider,
  createBudgetedStorageHost,
  createNamespacedStorageHost,
  createWindowStorageHost,
} from "../../../../runtime/ts/src/browser/provider-storage"
import type { EntryContract } from "../compiler/types"
import { createPreviewDocument } from "../preview-document"
import type {
  BrowserExecution,
  BrowserExecutionCompletion,
  GeneratedProjectModule,
} from "./browser-execution"
import type { ForeignRuntimeBundle } from "./foreign-runtime"
import { previewNavigationCancelled } from "./preview-navigation"

export type IsolatedExecutionRequest = Readonly<{
  modules: readonly GeneratedProjectModule[]
  entryPath: string
  entry: EntryContract
  input: string
  foreignRuntime?: ForeignRuntimeBundle
  initialHref: string
  storageScope?: string
}>

export type IsolatedExecutionOptions = Readonly<{
  signal: AbortSignal
  onDomMounted?: () => void
  networkAllowed?: boolean
  /** Only reduced deadlines are accepted, primarily for deterministic tests. */
  timeoutMs?: number
}>

const storageNamespace = "seseragi:playground:application:"
const storageOperations = new Set(["get", "set", "remove", "clear", "keys"])

export function createExecutionFrameDocument(
  source: string,
  session: string,
  networkAllowed = false
): string {
  // Only trusted bundled code is interpolated. Program sources travel over the
  // MessagePort after the frame is established, never through an HTML parser.
  const script = `Function(${JSON.stringify(source).replaceAll("<", "\\u003c")})()`
  return createPreviewDocument('<div id="app"></div>', networkAllowed)
    .replace(
      "script-src 'none'",
      `script-src 'unsafe-inline' 'unsafe-eval'; worker-src blob:; connect-src ${networkAllowed ? "https: http: wss: ws:" : "'none'"}`
    )
    .replace(
      "</body>",
      () => `<script data-session="${session}">${script}</script></body>`
    )
}

export async function startIsolatedProject(
  frame: HTMLIFrameElement,
  request: IsolatedExecutionRequest,
  options: IsolatedExecutionOptions
): Promise<BrowserExecution> {
  const { default: source } = await import("virtual:seseragi-execution-frame")
  if (options.signal.aborted) throw previewNavigationCancelled()
  const session = crypto.randomUUID()
  const channel = new MessageChannel()
  const port = channel.port1
  const owner = frame.ownerDocument.defaultView!
  // Scope is chosen by the editor, never by code running inside the frame.
  const namespace = `${storageNamespace}v2:${encodeURIComponent(request.storageScope ?? crypto.randomUUID())}:`
  const storage = createBrowserStorageProvider(
    createBudgetedStorageHost(
      createNamespacedStorageHost(createWindowStorageHost(owner), namespace),
      256 * 1024,
      namespace.length
    )
  )
  const allowStorage = request.entry.environment.some(
    ({ service }) => service === "storage"
  )
  let connected = false
  let running = false
  let messages = 0
  let settled = false
  let closed = false
  let cancellation: Promise<void> | undefined
  let deadline: ReturnType<typeof setTimeout> | undefined
  let acknowledgeCancellation: (() => void) | undefined
  let resolveCompletion!: (value: BrowserExecutionCompletion) => void
  let rejectCompletion!: (cause: unknown) => void
  const completion = new Promise<BrowserExecutionCompletion>(
    (resolve, reject) => {
      resolveCompletion = resolve
      rejectCompletion = reject
    }
  )
  // A setup error may occur before the caller owns completion.
  void completion.catch(() => undefined)
  const retire = () => {
    if (closed) return
    closed = true
    clearTimeout(deadline)
    frame.removeEventListener("load", onLoad)
    port.close()
    channel.port2.close()
    // Do not retire a newer run that has already reused this iframe.
    if (frame.dataset.executionSession === session) {
      delete frame.dataset.executionSession
      frame.removeAttribute("srcdoc")
      frame.removeAttribute("src")
    }
  }
  const finish = (value: BrowserExecutionCompletion) => {
    if (settled) return
    settled = true
    clearTimeout(deadline)
    resolveCompletion(value)
  }
  const onLoad = () => {
    if (frame.dataset.executionSession !== session) return
    if (connected) {
      finish({ kind: "cancelled" })
      acknowledgeCancellation?.()
      retire()
    } else {
      frame.contentWindow?.postMessage({ type: "loaded", session }, "*")
    }
  }
  let storagePending = 0
  const handleStorage = async (data: Record<string, unknown>) => {
    const { id, operation, input } = data
    if (
      !Number.isSafeInteger(id) ||
      (id as number) < 1 ||
      typeof operation !== "string" ||
      !storageOperations.has(operation)
    )
      return
    if (!allowStorage || settled || storagePending >= 64) {
      port.postMessage({
        type: "storage-result",
        id,
        error: "storage capability unavailable",
      })
      return
    }
    storagePending++
    try {
      const result = await storage[operation]!(input)
      if (!closed) port.postMessage({ type: "storage-result", id, result })
    } catch {
      if (!closed)
        port.postMessage({
          type: "storage-result",
          id,
          error: "invalid storage operation",
        })
    } finally {
      storagePending--
    }
  }
  try {
    await new Promise<void>((resolve, reject) => {
      const timer = setTimeout(
        () => fail(new Error("isolated preview startup timed out")),
        15_000
      )
      const cleanup = () => {
        clearTimeout(timer)
        owner.removeEventListener("message", onReady)
        options.signal.removeEventListener("abort", onAbort)
      }
      const fail = (error: unknown) => {
        cleanup()
        reject(error)
      }
      const onAbort = () => fail(previewNavigationCancelled())
      const onReady = (event: MessageEvent) => {
        if (
          event.source !== frame.contentWindow ||
          event.origin !== "null" ||
          event.data?.type !== "seseragi-execution-ready" ||
          event.data?.session !== session ||
          connected
        )
          return
        connected = true
        frame.contentWindow!.postMessage({ type: "connect", session }, "*", [
          channel.port2,
        ])
      }
      port.onmessage = (event: MessageEvent) => {
        const data = event.data
        if (data === null || typeof data !== "object") return
        if (++messages > 20_000) {
          if (!settled) {
            settled = true
            rejectCompletion(new Error("Program message limit exceeded"))
          }
          port.postMessage({ type: "cancel" })
          retire()
          return
        }
        if (data.type === "connected") {
          if (running) return
          running = true
          cleanup()
          if (options.signal.aborted) {
            reject(previewNavigationCancelled())
            return
          }
          port.postMessage({ type: "run", request })
          if (
            !request.entry.environment.some(({ service }) => service === "dom")
          ) {
            deadline = setTimeout(
              () => {
                if (settled) return
                settled = true
                port.postMessage({ type: "cancel" })
                rejectCompletion(
                  new Error(
                    "Program time limit exceeded (10 seconds). Simplify the program and run again."
                  )
                )
                retire()
              },
              Math.min(10_000, Math.max(1, options.timeoutMs ?? 10_000))
            )
          }
          resolve()
        } else if (data.type === "storage") {
          void handleStorage(data)
        } else if (data.type === "mounted" && !settled) {
          options.onDomMounted?.()
        } else if (data.type === "completion") {
          const value = data.completion
          if (
            value?.kind === "completed" &&
            typeof value.result?.stdout === "string" &&
            value.result.stdout.length <= 256 * 1024 &&
            typeof value.result?.debug === "string" &&
            value.result.debug.length <= 16 * 1024
          ) {
            finish({
              kind: "completed",
              result: {
                stdout: value.result.stdout,
                debug: value.result.debug,
              },
            })
            frame.removeEventListener("load", onLoad)
            port.close()
          } else if (value?.kind === "cancelled") {
            finish({ kind: "cancelled" })
            retire()
          } else if (!settled) {
            settled = true
            rejectCompletion(new Error("Invalid or oversized program result"))
            retire()
          }
        } else if (data.type === "cancelled") {
          acknowledgeCancellation?.()
          finish({ kind: "cancelled" })
          retire()
        } else if (data.type === "error" && !settled) {
          settled = true
          clearTimeout(deadline)
          rejectCompletion(
            new Error(
              typeof data.error === "string"
                ? data.error.slice(0, 4096)
                : "isolated execution failed"
            )
          )
          retire()
        }
      }
      frame.addEventListener("load", onLoad)
      owner.addEventListener("message", onReady)
      options.signal.addEventListener("abort", onAbort, { once: true })
      frame.setAttribute("sandbox", "allow-scripts allow-forms")
      frame.dataset.executionSession = session
      frame.removeAttribute("src")
      frame.srcdoc = createExecutionFrameDocument(
        source,
        session,
        options.networkAllowed
      )
      port.start()
    })
  } catch (error) {
    finish({ kind: "cancelled" })
    retire()
    throw error
  }
  return {
    completion,
    cancel: () => {
      cancellation ??= (async () => {
        if (!settled && !closed) {
          finish({ kind: "cancelled" })
          await new Promise<void>((resolve) => {
            const timer = setTimeout(resolve, 1_000)
            acknowledgeCancellation = () => {
              clearTimeout(timer)
              resolve()
            }
            port.postMessage({ type: "cancel" })
          })
        }
        finish({ kind: "cancelled" })
        retire()
      })()
      return cancellation
    },
  }
}
