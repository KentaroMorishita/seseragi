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

const released: string[] = []
const resource = (name: string, defect?: string) =>
  acquireRelease(succeed(name), () =>
    defer(() => {
      released.push(name)
      if (defect !== undefined) throw new Error(defect)
      return succeed(undefined)
    })
  )

const nested = scoped(
  flatMap(resource("outer-a"), () =>
    flatMap(
      scoped(
        flatMap(resource("inner-b"), () =>
          flatMap(resource("inner-c"), () => succeed(undefined))
        )
      ),
      () => flatMap(resource("outer-d"), () => succeed("done"))
    )
  )
)
const nestedResult = await run(nested, {})
assert(
  nestedResult.kind === "success" && nestedResult.value === "done",
  "nested scope must preserve the use result"
)
assert(
  released.join(",") === "inner-c,inner-b,outer-d,outer-a",
  "nested scopes must close locally in LIFO order"
)

released.length = 0
const failed = await run(
  scoped(flatMap(resource("typed"), () => fail("expected"))),
  {}
)
assert(
  failed.kind === "failure" && failed.error === "expected",
  "typed failure must survive finalization"
)
assert(released.join(",") === "typed", "typed failure must release")

released.length = 0
let defectEscaped = false
try {
  await run(
    scoped(
      flatMap(resource("defect"), () => () => {
        throw new Error("use-defect")
      })
    ),
    {}
  )
} catch (error) {
  defectEscaped = error instanceof Error && error.message === "use-defect"
}
assert(defectEscaped, "use defect must escape after finalization")
assert(released.join(",") === "defect", "defect exit must release")

released.length = 0
const acquireFailure = await run(
  scoped(
    acquireRelease(fail("no-resource"), () =>
      defer(() => {
        released.push("impossible")
        return succeed(undefined)
      })
    )
  ),
  {}
)
assert(
  acquireFailure.kind === "failure" && acquireFailure.error === "no-resource",
  "acquire failure must be preserved"
)
assert(released.length === 0, "failed acquisition must not register release")

released.length = 0
const cancellation = createEffectExecution()
let useStarted = (): void => undefined
const started = new Promise<void>((resolve) => {
  useStarted = resolve
})
const pendingUse: Effect<unknown, never, never> = (_environment, context) =>
  new Promise<never>(() => {
    useStarted()
    context?.onCancel(() => undefined)
  })
const cancelledRun = run(
  scoped(flatMap(resource("cancelled"), () => pendingUse)),
  {},
  cancellation.context
).then(
  () => false,
  (error: unknown) => error instanceof EffectCancellation
)
await started
await cancellation.cancel()
assert(await cancelledRun, "cancellation must not become typed failure")
assert(released.join(",") === "cancelled", "cancellation must await release")

released.length = 0
const lateCancellation = createEffectExecution()
let acquireStarted = (): void => undefined
const acquiring = new Promise<void>((resolve) => {
  acquireStarted = resolve
})
const lateAcquire: Effect<unknown, never, string> = (_environment, context) =>
  new Promise<string>((resolve) => {
    acquireStarted()
    context?.onCancel(() => resolve("late"))
  })
const lateRun = run(
  scoped(
    acquireRelease(lateAcquire, (name) =>
      defer(() => {
        released.push(name)
        return succeed(undefined)
      })
    )
  ),
  {},
  lateCancellation.context
).then(
  () => false,
  (error: unknown) => error instanceof EffectCancellation
)
await acquiring
await lateCancellation.cancel()
assert(await lateRun, "late acquisition must finish as cancellation")
assert(
  released.join(",") === "late",
  "late acquisition success must release before discard"
)

let closeCount = 0
const idempotentClose = (): Effect<unknown, never, undefined> =>
  defer(() => {
    if (closeCount === 0) closeCount += 1
    return succeed(undefined)
  })
const providerLike = scoped(
  flatMap(acquireRelease(succeed("handle"), idempotentClose), () =>
    idempotentClose()
  )
)
await run(providerLike, {})
assert(closeCount === 1, "explicit and scoped provider close must compose")

released.length = 0
let finalizerDefect: unknown
try {
  await run(
    scoped(
      flatMap(resource("first", "first-defect"), () =>
        flatMap(resource("second", "second-defect"), () => succeed(undefined))
      )
    ),
    {}
  )
} catch (error) {
  finalizerDefect = error
}
assert(
  finalizerDefect instanceof Error &&
    finalizerDefect.message === "second-defect",
  "first LIFO finalizer defect must be primary"
)
assert(
  released.join(",") === "second,first",
  "a finalizer defect must not skip remaining finalizers"
)
assert(
  Array.isArray(
    (finalizerDefect as Error & { suppressed?: unknown }).suppressed
  ) &&
    (finalizerDefect as Error & { suppressed: unknown[] }).suppressed.length ===
      1,
  "later finalizer defects must be attached"
)

const frozenPrimary = Object.freeze(new Error("frozen-primary"))
let frozenDefect: unknown
try {
  await run(
    scoped(
      flatMap(resource("fallback", "fallback-defect"), () =>
        acquireRelease(succeed("frozen"), () => () => {
          throw frozenPrimary
        })
      )
    ),
    {}
  )
} catch (error) {
  frozenDefect = error
}
assert(
  frozenDefect instanceof AggregateError &&
    frozenDefect.cause === frozenPrimary,
  "a frozen primary defect must remain the aggregate cause"
)

released.length = 0
await run(resource("root"), {})
assert(released.join(",") === "root", "run must close its root resource scope")

process.stdout.write("effect resource probe passed\n")
