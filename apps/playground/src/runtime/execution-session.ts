import { createBrowserNavigationProvider } from "../../../../runtime/ts/src/browser/provider-navigation"
import { providerRuntimeAbi } from "../../../../runtime/ts/src/provider"
import { defineProviderPackage } from "../../../../runtime/ts/src/provider-package"
import {
  type BrowserExecution,
  startGeneratedProject,
} from "./browser-execution"
import type { IsolatedExecutionRequest } from "./isolated-execution"
import { createExecutionCheckpoint } from "./execution-budget"
import { createPreviewHistory } from "./preview-history"

/** Runs only in an opaque frame or its dedicated Worker, never the editor. */
export function serveExecutionPort(
  port: MessagePort,
  domDocument?: Document
): void {
  const pending = new Map<
    number,
    { resolve: (value: unknown) => void; reject: (cause: Error) => void }
  >()
  let sequence = 0
  let execution: BrowserExecution | undefined
  let cancelled = false
  const rpc = (operation: string, input: unknown): Promise<unknown> => {
    if (cancelled) return Promise.reject(new Error("execution cancelled"))
    return new Promise((resolve, reject) => {
      const id = ++sequence
      pending.set(id, { resolve, reject })
      port.postMessage({ type: "storage", id, operation, input })
    })
  }
  const storage = defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: "seseragi/runtime-browser#storage",
    service: "std/web/storage::Storage",
    targets: ["browser"],
    operations: Object.fromEntries(
      ["get", "set", "remove", "clear", "keys"].map((name) => [
        name,
        (input: unknown) => rpc(name, input),
      ])
    ),
  })
  const rejectPending = () => {
    for (const waiter of pending.values())
      waiter.reject(new Error("execution cancelled"))
    pending.clear()
  }
  globalThis.addEventListener(
    "pagehide",
    () => {
      cancelled = true
      rejectPending()
      port.postMessage({ type: "cancelled" })
      void execution?.cancel()
    },
    { once: true }
  )
  let disposeHistory = () => {}
  let started = false
  port.onmessage = (message: MessageEvent) => {
    const data = message.data
    if (data?.type === "storage-result") {
      const waiter = pending.get(data.id)
      if (waiter === undefined) return
      pending.delete(data.id)
      if (data.error === undefined) waiter.resolve(data.result)
      else waiter.reject(new Error(String(data.error)))
    } else if (data?.type === "cancel") {
      cancelled = true
      rejectPending()
      void execution
        ?.cancel()
        .finally(() => port.postMessage({ type: "cancelled" }))
      if (execution === undefined) port.postMessage({ type: "cancelled" })
    } else if (data?.type === "run" && !started && !cancelled) {
      started = true
      const request = data.request as IsolatedExecutionRequest
      void (async () => {
        // Navigation is private to this preview. Opaque srcdoc cannot mutate
        // origin history; model the same-origin history inside the frame.
        const history = createPreviewHistory(request.initialHref)
        disposeHistory = history.dispose
        const navigation = createBrowserNavigationProvider(history)
        execution = await startGeneratedProject(
          request.modules,
          request.entryPath,
          request.entry,
          request.input,
          {
            maxOutputCodeUnits: 256 * 1024,
            ...(domDocument === undefined
              ? {}
              : { checkpoint: createExecutionCheckpoint() }),
            foreignRuntime: request.foreignRuntime,
            ...(domDocument === undefined ? {} : { domDocument }),
            storageProvider: storage,
            navigationProvider: navigation,
            onDomMounted: () => {
              if (!cancelled) port.postMessage({ type: "mounted" })
            },
          }
        )
        if (cancelled) await execution.cancel()
        const completion = await execution.completion
        if (!cancelled) port.postMessage({ type: "completion", completion })
      })()
        .catch((error: unknown) => {
          if (!cancelled)
            port.postMessage({
              type: "error",
              error: (error instanceof Error
                ? error.message
                : String(error)
              ).slice(0, 4096),
            })
        })
        .finally(() => {
          rejectPending()
          disposeHistory()
        })
    }
  }
  port.start()
}
