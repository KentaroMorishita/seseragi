import { createEffectExecution, run } from "@seseragi/runtime/effect"
import { assertProviderConformanceCase } from "@seseragi/runtime/provider-conformance"
import { createProviderChildProcesses } from "@seseragi/runtime/provider-child-process"
import { ProviderPackageLoader } from "@seseragi/runtime/provider-package"
import { runCollect } from "@seseragi/runtime/stream"
import { provider as bunProvider } from "seseragi/runtime-bun/child-process"
import { provider as nodeProvider } from "seseragi/runtime-node/child-process"
import {
  cancellableFixture,
  capturedFixture,
  limitedFixture,
  streamingFixture,
} from "./child-process-application.ts"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

const provider = requiredEnvironment("SESERAGI_CHILD_PROCESS_PROVIDER")
const service = requiredEnvironment("SESERAGI_CHILD_PROCESS_SERVICE")
const module = requiredEnvironment("SESERAGI_CHILD_PROCESS_MODULE")
const exportName = requiredEnvironment("SESERAGI_CHILD_PROCESS_EXPORT")
const target = requiredEnvironment("SESERAGI_CHILD_PROCESS_TARGET") as
  | "bun-process"
  | "node-process"
const loader = new ProviderPackageLoader(target, [
  {
    provider,
    service,
    target,
    module,
    exportName,
    loadMode: "lazy",
    importModule: selectedImport(module),
    source: { path: "src/main.ssrg", start: 0, end: 10 },
  },
])
const environment = Object.freeze({
  childProcesses: createProviderChildProcesses(await loader.load(provider)),
})

const captured = await run(capturedFixture(), environment)
assert(captured.kind === "success", "captured child must succeed")
assert(captured.value.status.tag === "ChildExited", "child must exit by code")
assert(captured.value.status.value === 7, "child exit code must be preserved")
assert(text(captured.value.stdout) === "HELLO", "stdout must be captured")
assert(text(captured.value.stderr) === "warn", "stderr must be captured")
assertProviderConformanceCase({ id: "success", terminal: captured.kind })

const limited = await run(limitedFixture(), environment)
assert(limited.kind === "failure", "capture limit must fail explicitly")
assert(
  limited.error.tag === "ChildOutputLimitExceeded" &&
    limited.error.value.channel.tag === "ChildStdout" &&
    limited.error.value.limitBytes === 4,
  "capture limit failure must preserve channel and bound"
)

const streamed = await run(runCollect(streamingFixture()), environment)
assert(streamed.kind === "success", "streaming child must succeed")
const events = streamed.value
const stdout = events
  .filter((event) => event.tag === "ChildStdoutChunk")
  .map((event) => text(event.value))
  .join("")
const stderr = events
  .filter((event) => event.tag === "ChildStderrChunk")
  .map((event) => text(event.value))
  .join("")
const exited = events.at(-1)
assert(stdout === "STREAM", "streaming stdout must preserve bytes")
assert(stderr === "err", "streaming stderr must preserve bytes")
assert(
  exited?.tag === "ChildExitedWith" &&
    exited.value.tag === "ChildExited" &&
    exited.value.value === 3,
  "streaming must end with exactly one exit event"
)
const execution = createEffectExecution()
const cursor = await cancellableFixture().open(environment, execution.context)
const first = await cursor.next()
assert(!first.done && first.value.tag === "ChildStdoutChunk", "pid event missing")
const pid = Number.parseInt(text(first.value.value), 10)
assert(Number.isSafeInteger(pid) && pid > 0, "child pid must be valid")
await execution.cancel()
await cursor.close()
assert(!processExists(pid), "cancellation must terminate and reap child")
assertProviderConformanceCase({
  id: "cancellation",
  terminal: "cancellation",
  notifications: 1,
  lateCompletion: "discarded",
})

await loader.shutdown()
assertProviderConformanceCase({ id: "cleanup", acquired: 4, released: 4, active: 0 })
assertProviderConformanceCase({ id: "leak", activeAfterCleanup: 0 })
process.stdout.write(`child process provider probe passed: ${target}\n`)

function text(value: Uint8Array): string {
  return new TextDecoder().decode(value)
}

function processExists(pid: number): boolean {
  try {
    process.kill(pid, 0)
    return true
  } catch {
    return false
  }
}

function requiredEnvironment(name: string): string {
  const value = process.env[name]
  if (value === undefined || value.length === 0) throw new Error(`missing ${name}`)
  return value
}

function selectedImport(module: string): () => Promise<unknown> {
  if (module === "seseragi/runtime-bun/child-process") {
    return async () => Object.freeze({ provider: bunProvider })
  }
  if (module === "seseragi/runtime-node/child-process") {
    return async () => Object.freeze({ provider: nodeProvider })
  }
  throw new Error(`unexpected child process provider module: ${module}`)
}
