import { milliseconds, sleep, zeroDuration } from "@seseragi/runtime/clock"
import {
  createEffectExecution,
  isEffectCancellation,
  run,
} from "@seseragi/runtime/effect"
import { createProviderClock } from "@seseragi/runtime/provider-clock"
import { assertProviderConformanceCase } from "@seseragi/runtime/provider-conformance"
import { ProviderPackageLoader } from "@seseragi/runtime/provider-package"
import { observeThenSleep } from "./clock-application"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

const provider = requiredEnvironment("SESERAGI_CLOCK_PROVIDER")
const service = requiredEnvironment("SESERAGI_CLOCK_SERVICE")
const module = requiredEnvironment("SESERAGI_CLOCK_MODULE")
const exportName = requiredEnvironment("SESERAGI_CLOCK_EXPORT")
const loader = new ProviderPackageLoader("bun-process", [
  {
    provider,
    service,
    target: "bun-process",
    module,
    exportName,
    loadMode: "lazy",
    importModule: () => import(module),
    source: { path: "src/main.ssrg", start: 0, end: 5 },
  },
])

await loader.start()
const clock = createProviderClock(await loader.load(provider))
const environment = Object.freeze({ clock })
const observed = await run(observeThenSleep(zeroDuration()), environment)
assert(observed.kind === "success", "Clock now and zero sleep must succeed")
assert(
  typeof observed.value === "object" &&
    observed.value !== null &&
    Object.isFrozen(observed.value),
  "Clock now must return an opaque frozen Instant"
)
assertProviderConformanceCase({ id: "success", terminal: observed.kind })

const duration = milliseconds(60_000)
assert(duration.tag === "Right", "one minute must be a valid Duration")
const execution = createEffectExecution()
const pending = run(
  sleep(duration.value),
  environment,
  execution.context
).catch((error: unknown) => error)
const firstCancel = execution.cancel()
const secondCancel = execution.cancel()
assert(firstCancel === secondCancel, "Effect cancellation must be idempotent")
assert(
  isEffectCancellation(await pending),
  "Clock sleep cancellation must stay outside the typed failure channel"
)
assertProviderConformanceCase({
  id: "cancellation",
  terminal: "cancellation",
  notifications: 1,
  lateCompletion: "discarded",
})
await firstCancel
await loader.shutdown()

process.stdout.write("clock provider probe passed\n")

function requiredEnvironment(name: string): string {
  const value = process.env[name]
  if (value === undefined || value.length === 0) {
    throw new Error(`missing ${name}`)
  }
  return value
}
