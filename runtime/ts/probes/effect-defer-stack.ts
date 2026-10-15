import {
  acquireRelease,
  createEffectExecution,
  defer,
  type Effect,
  EffectCancellation,
  fail,
  flatMap,
  run,
  scoped,
  succeed,
} from "../src/effect"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

let constructed = 0
const descend = (remaining: number): Effect<unknown, never, number> =>
  defer(() => {
    constructed++
    return remaining === 0 ? succeed(42) : descend(remaining - 1)
  })
const work = descend(100_000)
assert(constructed === 0, "defer must not construct its source early")
for (let iteration = 1; iteration <= 2; iteration++) {
  const result = await run(work, {})
  assert(result.kind === "success" && result.value === 42, "deep defer result")
  assert(
    constructed === iteration * 100_001,
    "defer must reconstruct every run"
  )
}

const events: string[] = []
const execution = createEffectExecution()
const resource = acquireRelease(succeed(undefined), () =>
  defer(() => {
    events.push("release")
    return succeed(undefined)
  })
)
const cancelled = scoped(
  flatMap(resource, () =>
    defer(() => {
      events.push("callback")
      void execution.cancel()
      return succeed(0)
    })
  )
)
const guarded = flatMap(cancelled, () => {
  events.push("unexpected")
  return succeed(1)
})
const cancellation = await run(guarded, {}, execution.context).then(
  () => false,
  (error: unknown) => error instanceof EffectCancellation
)
await execution.close()
assert(cancellation, "defer cancellation must remain outside typed failure")
assert(
  events.join(",") === "callback,release",
  "cancel must run cleanup and skip next"
)

const payload = { tag: "expected" }
const failed = await run(
  defer(() => fail(payload)),
  {}
)
assert(
  failed.kind === "failure" && failed.error === payload,
  "failure identity"
)
let escaped: unknown
try {
  await run(
    defer(() => {
      throw payload
    }),
    {}
  )
} catch (error) {
  escaped = error
}
assert(escaped === payload, "thunk defects must preserve identity")

// Alternating plans must also use the same interpreter loop.
const mixed = (remaining: number): Effect<unknown, never, number> =>
  defer(() =>
    flatMap(succeed(remaining), (value) =>
      value === 0 ? succeed(42) : mixed(value - 1)
    )
  )
assert(
  (await run(mixed(100_000), {})).kind === "success",
  "mixed bind/defer plans"
)
console.log("defer coldness, replay, stack, failure, defect, cancellation: ok")
