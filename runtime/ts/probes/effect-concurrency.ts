import { arrayReducible } from "../src/array"
import {
  awaitFiber,
  createEffectExecution,
  defer,
  type Effect,
  EffectCancellation,
  fail,
  flatMap,
  forEachParallel,
  fork,
  interrupt,
  join,
  parallelism,
  race,
  run,
  scoped,
  succeed,
  traverseParallel,
} from "../src/effect"
import {
  awaitDeferred,
  fail as failDeferred,
  make as makeDeferred,
  poll as pollDeferred,
  succeed as succeedDeferred,
} from "../src/deferred"
import { bounded, close, offer, take, tryTake, unbounded } from "../src/queue"
import {
  acquire,
  available,
  make as makeSemaphore,
  release,
  withPermit,
} from "../src/semaphore"

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

const deferred = await success(makeDeferred<string, number>())
const waiterOrder: number[] = []
const deferredWaiters = [1, 2, 3].map((index) =>
  success(awaitDeferred(deferred)).then((value) => {
    waiterOrder.push(index)
    return value
  })
)
await Promise.resolve()
assert(
  await success(succeedDeferred(42, deferred)),
  "first completion must win"
)
assert(
  !(await success(failDeferred("late", deferred))),
  "later completion must lose"
)
assert(
  (await Promise.all(deferredWaiters)).join(",") === "42,42,42",
  "Deferred must resume every waiter"
)
assert(
  waiterOrder.join(",") === "1,2,3",
  "Deferred waiters must resume in registration order"
)
const deferredPoll = await success(pollDeferred(deferred))
assert(
  deferredPoll.tag === "Just" &&
    deferredPoll.value.tag === "Right" &&
    deferredPoll.value.value === 42,
  "Deferred poll must expose the completed result"
)

const cancellableDeferred = await success(makeDeferred<never, number>())
const deferredExecution = createEffectExecution()
const cancelledWait = run(
  awaitDeferred(cancellableDeferred),
  {},
  deferredExecution.context
).then(
  () => false,
  (error: unknown) => error instanceof EffectCancellation
)
await Promise.resolve()
await deferredExecution.cancel()
assert(await cancelledWait, "Deferred waiter cancellation must stay untyped")
assert(
  await success(succeedDeferred(7, cancellableDeferred)),
  "waiter cancellation must not complete Deferred"
)
assert(
  (await success(awaitDeferred(cancellableDeferred))) === 7,
  "Deferred must remain usable after waiter cancellation"
)

const fifoQueue = await success(bounded<number>(1))
await success(offer(1, fifoQueue))
const blockedOffer = success(offer(2, fifoQueue))
await Promise.resolve()
assert((await success(take(fifoQueue))) === 1, "Queue must take FIFO head")
await blockedOffer
assert(
  (await success(take(fifoQueue))) === 2,
  "free capacity must accept the oldest blocked offer"
)

const takerQueue = await success(unbounded<number>())
const takers = [success(take(takerQueue)), success(take(takerQueue))]
await Promise.resolve()
await success(offer(10, takerQueue))
await success(offer(20, takerQueue))
assert(
  (await Promise.all(takers)).join(",") === "10,20",
  "Queue takers must be FIFO"
)

const cancelledOfferQueue = await success(bounded<number>(1))
await success(offer(1, cancelledOfferQueue))
const offerExecution = createEffectExecution()
const cancelledOffer = run(
  offer(2, cancelledOfferQueue),
  {},
  offerExecution.context
).then(
  () => false,
  (error: unknown) => error instanceof EffectCancellation
)
await Promise.resolve()
await offerExecution.cancel()
assert(await cancelledOffer, "Queue offer cancellation must stay untyped")
assert(
  (await success(take(cancelledOfferQueue))) === 1,
  "cancelled offer must not enqueue its value"
)
const emptyAfterCancellation = await success(tryTake(cancelledOfferQueue))
assert(
  emptyAfterCancellation.tag === "Right" &&
    emptyAfterCancellation.value.tag === "Nothing",
  "cancelled offer must leave no hidden value"
)

const cancelledTakeQueue = await success(unbounded<number>())
const takeExecution = createEffectExecution()
const cancelledTake = run(
  take(cancelledTakeQueue),
  {},
  takeExecution.context
).then(
  () => false,
  (error: unknown) => error instanceof EffectCancellation
)
await Promise.resolve()
await takeExecution.cancel()
assert(await cancelledTake, "Queue take cancellation must stay untyped")
await success(offer(9, cancelledTakeQueue))
assert(
  (await success(take(cancelledTakeQueue))) === 9,
  "cancelled take must not consume a later value"
)

const closingQueue = await success(bounded<number>(2))
await success(offer(1, closingQueue))
await success(offer(2, closingQueue))
const rejectedOffer = run(offer(3, closingQueue), {})
await Promise.resolve()
await success(close(closingQueue))
assert(
  (await success(take(closingQueue))) === 1,
  "close must drain FIFO value 1"
)
assert(
  (await success(take(closingQueue))) === 2,
  "close must drain FIFO value 2"
)
const closedTake = await run(take(closingQueue), {})
const closedOffer = await rejectedOffer
assert(
  closedTake.kind === "failure" && closedTake.error.tag === "QueueClosed",
  "take after drain must fail QueueClosed"
)
assert(
  closedOffer.kind === "failure" && closedOffer.error.tag === "QueueClosed",
  "pending offer at close must fail QueueClosed"
)

const semaphore = await success(makeSemaphore(1))
const firstPermit = await success(acquire(semaphore))
const acquisitionOrder: number[] = []
const secondPermit = success(acquire(semaphore)).then((permit) => {
  acquisitionOrder.push(2)
  return permit
})
const thirdPermit = success(acquire(semaphore)).then((permit) => {
  acquisitionOrder.push(3)
  return permit
})
await Promise.resolve()
await success(release(firstPermit))
const acquiredSecond = await secondPermit
assert(acquisitionOrder.join(",") === "2", "Semaphore acquire must be FIFO")
await success(release(acquiredSecond))
const acquiredThird = await thirdPermit
assert(
  acquisitionOrder.join(",") === "2,3",
  "Semaphore must grant the next registered waiter"
)
await success(release(acquiredThird))
await success(release(acquiredThird))
assert(
  (await success(available(semaphore))) === 1,
  "Permit release must be idempotent"
)

const cancelledAcquireSemaphore = await success(makeSemaphore(1))
const heldPermit = await success(acquire(cancelledAcquireSemaphore))
const acquireExecution = createEffectExecution()
const cancelledAcquire = run(
  acquire(cancelledAcquireSemaphore),
  {},
  acquireExecution.context
).then(
  () => false,
  (error: unknown) => error instanceof EffectCancellation
)
await Promise.resolve()
await acquireExecution.cancel()
assert(
  await cancelledAcquire,
  "Semaphore acquire cancellation must stay untyped"
)
await success(release(heldPermit))
assert(
  (await success(available(cancelledAcquireSemaphore))) === 1,
  "cancelled acquire must not consume a permit"
)

const permitFailure = await run(withPermit(semaphore, fail("expected")), {})
assert(
  permitFailure.kind === "failure" && permitFailure.error === "expected",
  "withPermit must preserve typed failure"
)
assert(
  (await success(available(semaphore))) === 1,
  "withPermit must release after typed failure"
)

let permitUseStarted = (): void => undefined
const permitUse = new Promise<void>((resolve) => {
  permitUseStarted = resolve
})
const pendingPermitUse: Effect<unknown, never, never> = () =>
  new Promise<never>(() => permitUseStarted())
const permitExecution = createEffectExecution()
const cancelledPermitUse = run(
  withPermit(semaphore, pendingPermitUse),
  {},
  permitExecution.context
).then(
  () => false,
  (error: unknown) => error instanceof EffectCancellation
)
await permitUse
await permitExecution.cancel()
assert(await cancelledPermitUse, "withPermit cancellation must stay untyped")
assert(
  (await success(available(semaphore))) === 1,
  "withPermit must release after cancellation"
)

const joinedFailure = await run(
  scoped(flatMap(fork(fail("fiber-failure")), join)),
  {}
)
assert(
  joinedFailure.kind === "failure" && joinedFailure.error === "fiber-failure",
  "join must rethrow the child typed failure"
)

let supervisedChildStarted = (): void => undefined
const supervisedStarted = new Promise<void>((resolve) => {
  supervisedChildStarted = resolve
})
let supervisedCleanup = false
const supervisedChild: Effect<unknown, never, never> = (
  _environment,
  context
) =>
  new Promise<never>(() => {
    supervisedChildStarted()
    context?.onCancel(() => {
      supervisedCleanup = true
    })
  })
const supervisedRun = run(
  scoped(flatMap(fork(supervisedChild), () => succeed("parent-done"))),
  {}
)
await supervisedStarted
const supervisedResult = await supervisedRun
assert(
  supervisedResult.kind === "success" &&
    supervisedResult.value === "parent-done",
  "parent result must survive child supervision"
)
assert(
  supervisedCleanup,
  "normal parent scope exit must cancel and await unfinished child cleanup"
)

const interruptExecution = createEffectExecution()
const pendingFiberResult = await run(
  fork<unknown, never, never>(
    (_environment, context) =>
      new Promise<never>(() => context?.onCancel(() => undefined))
  ),
  {},
  interruptExecution.context
)
assert(
  pendingFiberResult.kind === "success",
  "fork must return its child Fiber"
)
const pendingFiber = pendingFiberResult.value
await success(interrupt(pendingFiber))
const interruptedExit = await success(awaitFiber(pendingFiber))
assert(
  interruptedExit.tag === "FiberCancelled",
  "interrupt must complete Fiber as cancelled"
)
await interruptExecution.close()

let raceLoserCleaned = false
const raceLoser: Effect<unknown, never, never> = (_environment, context) =>
  new Promise<never>(() =>
    context?.onCancel(() => {
      raceLoserCleaned = true
    })
  )
assert(
  (await success(race(succeed("left"), raceLoser))) === "left",
  "race must return the first result"
)
assert(raceLoserCleaned, "race must await loser cleanup")

const boundedParallelism = parallelism(2)
assert(boundedParallelism.tag === "Right", "parallelism must accept positives")
if (boundedParallelism.tag === "Right") {
  let active = 0
  let maximumActive = 0
  const parallel = traverseParallel(
    boundedParallelism.value,
    (value: number): Effect<unknown, never, number> =>
      async () => {
        active += 1
        maximumActive = Math.max(maximumActive, active)
        await new Promise<void>((resolve) => setTimeout(resolve, 4 - value))
        active -= 1
        return value * 10
      },
    [1, 2, 3],
    arrayReducible
  )
  assert(
    (await success(parallel)).join(",") === "10,20,30",
    "parallel traversal results must retain input order"
  )
  assert(maximumActive === 2, "parallel traversal must respect its bound")

  const observed: number[] = []
  await success(
    forEachParallel(
      boundedParallelism.value,
      (value: number) => () => {
        observed.push(value)
      },
      [1, 2, 3],
      arrayReducible
    )
  )
  assert(
    observed.slice().sort().join(",") === "1,2,3",
    "parallel forEach must execute every input exactly once"
  )

  const simultaneousFailure = await run(
    traverseParallel(
      boundedParallelism.value,
      (value: number) => fail(`failure-${value}`),
      [1, 2],
      arrayReducible
    ),
    {}
  )
  assert(
    simultaneousFailure.kind === "failure" &&
      simultaneousFailure.error === "failure-1",
    "same-turn parallel failures must select the lowest input index"
  )
}

assert(
  parallelism(0).tag === "Left",
  "parallelism must reject a non-positive bound"
)

process.stdout.write("effect concurrency probe passed\n")
