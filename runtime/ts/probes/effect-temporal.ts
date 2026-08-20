import {
  attempt,
  createEffectExecution,
  defer,
  EffectCancellation,
  fail,
  fromMaybe,
  provide,
  provideSome,
  recurs,
  recover,
  repeat,
  retry,
  run,
  service,
  spaced,
  succeed,
  timeout,
  timeoutFail,
  whileInput,
  type ClockRequirement,
  type Effect,
  type EffectContext,
} from "../src/effect"
import {
  addDuration,
  hours,
  milliseconds,
  nanoseconds,
  seconds,
  toNanoseconds,
  zeroDuration,
} from "../src/clock"
import { get, make, modify, set, update } from "../src/ref"
import { Just, Nothing } from "../src/sum"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

async function success<Success>(
  effect: Effect<unknown, unknown, Success>,
  environment: unknown = {}
): Promise<Success> {
  const result = await run(effect, environment)
  assert(result.kind === "success", "expected Effect success")
  return result.value
}

const zero = zeroDuration()
assert(toNanoseconds(zero) === 0, "zeroDuration must be exact")
assert(nanoseconds(-1).tag === "Left", "negative Duration must fail")
assert(hours(Number.MAX_SAFE_INTEGER).tag === "Left", "Duration overflow must fail")
const oneSecond = seconds(1)
const twoMilliseconds = milliseconds(2)
assert(oneSecond.tag === "Right", "seconds must construct Duration")
assert(twoMilliseconds.tag === "Right", "milliseconds must construct Duration")
if (oneSecond.tag === "Right" && twoMilliseconds.tag === "Right") {
  const sum = addDuration(twoMilliseconds.value, oneSecond.value)
  assert(sum.tag === "Right", "Duration addition must succeed")
  if (sum.tag === "Right") {
    assert(toNanoseconds(sum.value) === 1_002_000_000, "Duration sum is wrong")
  }
}

const reference = await success(make(1))
let callbackCount = 0
await success(update((value) => {
  callbackCount += 1
  return value + 1
}, reference))
const previous = await success(modify((value) => [value, value + 3], reference))
assert(previous === 2, "Ref.modify must return the callback result")
assert(callbackCount === 1, "Ref.update callback must run once")
await success(set(9, reference))
assert(await success(get(reference)) === 9, "Ref.set/get must round trip")
let refDefect = false
try {
  await success(update(() => {
    throw new Error("ref-defect")
  }, reference))
} catch (error) {
  refDefect = error instanceof Error && error.message === "ref-defect"
}
assert(refDefect, "Ref callback defect must escape")
assert(await success(get(reference)) === 9, "Ref defect must not update the cell")

assert(
  await success(fromMaybe("missing", Just(4))) === 4,
  "fromMaybe Just must succeed"
)
const missing = await run(fromMaybe("missing", Nothing), {})
assert(
  missing.kind === "failure" && missing.error === "missing",
  "fromMaybe Nothing must fail"
)
const attemptedFailure = await success(attempt(fail("typed")))
assert(
  attemptedFailure.tag === "Left" && attemptedFailure.value === "typed",
  "attempt must move typed failure to Either"
)
const attemptedSuccess = await success(attempt(succeed(5)))
assert(
  attemptedSuccess.tag === "Right" && attemptedSuccess.value === 5,
  "attempt must move success to Either"
)
let defectEscaped = false
try {
  await run(attempt(() => {
    throw new Error("defect")
  }), {})
} catch (error) {
  defectEscaped = error instanceof Error && error.message === "defect"
}
assert(defectEscaped, "attempt must not capture defects")

const pending = (): Effect<unknown, never, never> =>
  (_environment, context) =>
    new Promise<never>(() => {
      context?.onCancel(() => undefined)
    })
const cancellation = createEffectExecution()
const cancelledAttempt = run(attempt(pending()), {}, cancellation.context).then(
  () => false,
  (error: unknown) => error instanceof EffectCancellation
)
await Promise.resolve()
await cancellation.cancel()
assert(await cancelledAttempt, "attempt must not capture cancellation")

let deferredRuns = 0
const deferred = defer(() => {
  deferredRuns += 1
  return succeed(deferredRuns)
})
assert(deferredRuns === 0, "defer must remain cold")
assert(await success(deferred) === 1, "defer first run is wrong")
assert(await success(deferred) === 2, "defer must rebuild each run")
assert(
  await success(recover(() => succeed(8), fail("recoverable"))) === 8,
  "recover must handle typed failure"
)
assert(
  await success(provide({ value: 7 }, service((environment: { value: number }) => environment.value))) === 7,
  "provide/service must use the supplied environment"
)
assert(
  await success(
    provideSome(
      (environment: { outer: number }) => ({ value: environment.outer + 1 }),
      service((environment: { value: number }) => environment.value)
    ),
    { outer: 8 }
  ) === 9,
  "provideSome must project once per run"
)

let sleeps = 0
const clockEnvironment: ClockRequirement = {
  clock: {
    sleep: async (_duration, context: EffectContext) => {
      if (context.signal.aborted) throw new EffectCancellation()
      sleeps += 1
    },
  },
}
const retryPolicy = spaced<string>(2, zero)
assert(retryPolicy.tag === "Right", "spaced policy must construct")
if (retryPolicy.tag === "Right") {
  let attempts = 0
  const flaky = defer(() => {
    attempts += 1
    return attempts < 3 ? fail("not-ready") : succeed(attempts)
  })
  assert(
    await success(retry(retryPolicy.value, flaky), clockEnvironment) === 3,
    "retry must rerun until success"
  )
}
const exhaustedPolicy = recurs<string>(1)
assert(exhaustedPolicy.tag === "Right", "recurs policy must construct")
if (exhaustedPolicy.tag === "Right") {
  const exhausted = await run(
    retry(exhaustedPolicy.value, fail("still-failing")),
    clockEnvironment
  )
  assert(
    exhausted.kind === "failure" && exhausted.error === "still-failing",
    "retry must preserve exhausted failure"
  )
}
const repeatPolicy = recurs<number>(2)
assert(repeatPolicy.tag === "Right", "repeat policy must construct")
if (repeatPolicy.tag === "Right") {
  let repetitions = 0
  assert(
    await success(
      repeat(repeatPolicy.value, defer(() => succeed(++repetitions))),
      clockEnvironment
    ) === 3,
    "repeat must return the final run"
  )
}
let whileRuns = 0
assert(
  await success(
    repeat(
      whileInput<number>((value) => value < 2),
      defer(() => succeed(++whileRuns))
    ),
    clockEnvironment
  ) === 2,
  "whileInput must observe each success exactly once"
)

const immediate = await success(timeout(zero, succeed("ready")), clockEnvironment)
assert(
  immediate.tag === "Just" && immediate.value === "ready",
  "zero timeout must prefer an immediate source"
)
let finalized = false
const never: Effect<unknown, never, never> = (_environment, context) =>
  new Promise<never>(() => {
    context?.onCancel(() => {
      finalized = true
    })
  })
const timedOut = await success(timeout(zero, never), clockEnvironment)
assert(timedOut.tag === "Nothing", "timeout must return Nothing")
assert(finalized, "timeout must await source cancellation cleanup")
const timedFailure = await run(
  timeoutFail("expired", zero, never),
  clockEnvironment
)
assert(
  timedFailure.kind === "failure" && timedFailure.error === "expired",
  "timeoutFail must use its typed failure"
)
assert(sleeps >= 8, "temporal operations must use the Clock service")

process.stdout.write("effect temporal probe passed\n")
