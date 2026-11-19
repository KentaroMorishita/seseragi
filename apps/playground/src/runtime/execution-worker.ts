import { serveExecutionPort } from "./execution-session"
import { timingClock } from "./run-timing"

// ここまで来たら、このスクリプトの読み込みと解析は済んでいる（実行時間の内訳）
const scriptStartedAt = timingClock()
// A self-contained classic Worker inherits the opaque frame's CSP. It cannot
// access a document, origin storage, or the editor. The frame can terminate it.
globalThis.addEventListener("message", function connect(event: MessageEvent) {
  const port = event.data?.port
  if (!(port instanceof MessagePort)) return
  globalThis.removeEventListener("message", connect)
  if (
    typeof event.data.randomSeed === "string" &&
    event.data.randomSeed.length <= 100
  )
    globalThis.__SESERAGI_RANDOM_SEED__ = event.data.randomSeed
  serveExecutionPort(port, undefined, scriptStartedAt)
})
