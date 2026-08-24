import { arrayIterable } from "../src/array"
import {
  acquireRelease,
  attempt,
  createEffectExecution,
  defer,
  type Effect,
  EffectCancellation,
  fail,
  flatMap as flatMapEffect,
  run,
  succeed,
} from "../src/effect"
import {
  buffer,
  bufferCapacity,
  filter,
  flatMap,
  fromArray,
  fromEffect,
  fromIterable,
  fromPull,
  map,
  merge,
  runCollect,
  runForEach,
  singleton,
  streamMonad,
  take,
} from "../src/stream"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

async function success<Success>(
  effect: Effect<unknown, unknown, Success>
): Promise<Success> {
  const result = await run(effect, {})
  assert(result.kind === "success", "expected Effect success")
  return result.value
}

const capacityResult = bufferCapacity(1)
assert(capacityResult.tag === "Right", "positive capacity must validate")
const capacity = capacityResult.value
const invalid = bufferCapacity(0)
assert(
  invalid.tag === "Left" &&
    invalid.value.tag === "NonPositiveBufferCapacity" &&
    invalid.value.value === 0,
  "non-positive capacity must be rejected before execution"
)

let effectRuns = 0
const cold = fromEffect(
  defer(() => {
    effectRuns += 1
    return succeed(effectRuns)
  })
)
assert(effectRuns === 0, "fromEffect construction must stay cold")
assert((await success(runCollect(cold))).join(",") === "1", "first run")
assert((await success(runCollect(cold))).join(",") === "2", "second run")

const transformed = fromArray([1, 2, 3])
const values = await success(
  runCollect(
    flatMap(
      (value) => singleton(value * 10),
      filter(
        (value) => value > 1,
        map((value) => value + 1, transformed)
      )
    )
  )
)
assert(
  values.join(",") === "20,30,40",
  "map/filter/flatMap must retain sequential source order"
)
const iterableValues = await success(
  runCollect(fromIterable([4, 5], arrayIterable))
)
assert(
  iterableValues.join(",") === "4,5",
  "fromIterable must create and drain a fresh iterator"
)
const monadValues = await success(
  runCollect(
    streamMonad.flatMap((value: number) => singleton(value + 1))(
      streamMonad.pure(6)
    )
  )
)
assert(monadValues.join(",") === "7", "Stream Monad must be executable")

let pullFactories = 0
let pullCount = 0
let pullCloses = 0
const pullSource = fromPull(() => {
  pullFactories += 1
  let next = 1
  return {
    async pull() {
      pullCount += 1
      return next <= 3
        ? { done: false as const, value: next++ }
        : { done: true as const, value: undefined }
    },
    close() {
      pullCloses += 1
    },
  }
})
assert(pullFactories === 0, "pull factory must stay cold")
assert(
  (await success(runCollect(take(2, pullSource)))).join(",") === "1,2",
  "take must stop after its exact demand"
)
assert(pullCount === 2, "take must not issue demand past its count")
assert(pullCloses === 1, "early stop must close a pull subscription once")
await success(runCollect(take(1, pullSource)))
assert(pullFactories === 2, "each terminal run must create a new subscription")
assert(pullCloses === 2, "each terminal scope must close independently")

let bufferedPulls = 0
let firstActionStarted = (): void => undefined
const firstAction = new Promise<void>((resolve) => {
  firstActionStarted = resolve
})
let releaseFirstAction = (): void => undefined
const firstActionGate = new Promise<void>((resolve) => {
  releaseFirstAction = resolve
})
const bufferedSource = fromPull(() => {
  let next = 1
  return {
    async pull() {
      bufferedPulls += 1
      return next <= 3
        ? { done: false as const, value: next++ }
        : { done: true as const, value: undefined }
    },
    close() {},
  }
})
const consumed: number[] = []
const bufferedRun = run(
  runForEach(
    (value) => async () => {
      consumed.push(value)
      if (value === 1) {
        firstActionStarted()
        await firstActionGate
      }
    },
    buffer(capacity, bufferedSource)
  ),
  {}
)
await firstAction
for (let attempt = 0; attempt < 20 && bufferedPulls < 2; attempt += 1) {
  await new Promise<void>((resolve) => setTimeout(resolve, 0))
}
assert(
  bufferedPulls === 2,
  "capacity one may hold one unread value but must not pull a third"
)
releaseFirstAction()
const bufferedResult = await bufferedRun
assert(bufferedResult.kind === "success", "buffered stream must complete")
assert(consumed.join(",") === "1,2,3", "lossless buffer must preserve FIFO")

const releases: string[] = []
const failing = (label: string) =>
  fromEffect(
    flatMapEffect(
      acquireRelease(succeed(label), () =>
        defer(() => {
          releases.push(label)
          return succeed(undefined)
        })
      ),
      () => fail(label)
    )
  )
const mergedFailure = await success(
  attempt(runCollect(merge(failing("right"), failing("left"))))
)
assert(
  mergedFailure.tag === "Left" && mergedFailure.value === "left",
  "same-turn merge failures must choose left"
)
assert(
  releases.includes("left") && releases.includes("right"),
  "merge failure must await both source finalizers"
)

let cancellationStarted = (): void => undefined
const started = new Promise<void>((resolve) => {
  cancellationStarted = resolve
})
let cancellationCloses = 0
const pending = fromPull(() => ({
  pull: () => new Promise<IteratorResult<never>>(() => cancellationStarted()),
  close() {
    cancellationCloses += 1
  },
}))
const execution = createEffectExecution()
const cancelled = run(runCollect(pending), {}, execution.context).then(
  () => false,
  (error: unknown) => error instanceof EffectCancellation
)
await started
await execution.cancel()
assert(await cancelled, "Stream cancellation must stay outside typed failure")
assert(cancellationCloses === 1, "cancellation must close producer once")

process.stdout.write("stream runtime probe passed\n")
