// Trusted bootstrap. Program evaluation stays outside the editor realm.
import { serveExecutionPort } from "./execution-session"
import type { IsolatedExecutionRequest } from "./isolated-execution"
declare const __SESERAGI_EXECUTION_WORKER__: string

let connected = false
const token = document.currentScript?.getAttribute("data-session")
window.addEventListener("message", function connect(event: MessageEvent) {
  if (connected || event.source !== parent || event.data?.session !== token)
    return
  if (event.data?.type === "loaded") {
    parent.postMessage(
      { type: "seseragi-execution-ready", session: token },
      "*"
    )
    return
  }
  if (event.data?.type !== "connect" || event.ports.length !== 1) return
  connected = true
  window.removeEventListener("message", connect)
  const port = event.ports[0]!
  port.onmessage = (message: MessageEvent) => {
    if (message.data?.type !== "run") return
    const request = message.data.request as IsolatedExecutionRequest
    if (request.entry.environment.some(({ service }) => service === "dom")) {
      // DOM programs need a document. Arbitrary foreign JS cannot be safely
      // interrupted here, so keep that combination out of the shared UI thread.
      if (Object.keys(request.foreignRuntime?.modules ?? {}).length > 0) {
        port.postMessage({
          type: "error",
          error:
            "Interactive foreign JavaScript is not supported in the restricted preview. Run this project with the CLI.",
        })
        return
      }
      serveExecutionPort(port, document)
      port.dispatchEvent(new MessageEvent("message", { data: message.data }))
      return
    }
    const url = URL.createObjectURL(
      new Blob([__SESERAGI_EXECUTION_WORKER__], { type: "text/javascript" })
    )
    const worker = new Worker(url)
    URL.revokeObjectURL(url)
    const channel = new MessageChannel()
    const stop = () => {
      worker.terminate()
      channel.port1.close()
    }
    window.addEventListener("pagehide", stop, { once: true })
    channel.port1.onmessage = (event: MessageEvent) => {
      port.postMessage(event.data)
      if (event.data?.type === "completion" || event.data?.type === "error")
        stop()
    }
    worker.onerror = () => {
      port.postMessage({ type: "error", error: "Program worker failed" })
      stop()
    }
    port.onmessage = (event: MessageEvent) => {
      if (event.data?.type === "cancel") {
        stop()
        port.postMessage({ type: "cancelled" })
      } else channel.port1.postMessage(event.data)
    }
    channel.port1.start()
    // Preserve the existing explicit deterministic test seed, not ambient
    // parent globals or storage. The provider reads it lazily on first use.
    worker.postMessage(
      { port: channel.port2, randomSeed: globalThis.__SESERAGI_RANDOM_SEED__ },
      [channel.port2]
    )
    channel.port1.postMessage(message.data)
  }
  port.start()
  port.postMessage({ type: "connected" })
})
