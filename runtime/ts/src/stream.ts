import { defectWithSuppressed } from "./cleanup-defect"
import { type Iterable, NonPositiveSize, type SizeError } from "./collection"
import {
  awaitEffectValue,
  awaitTurnEnd,
  createEffectExecution,
  type Effect,
  type EffectContext,
  type EffectExecution,
  executionAcquiring,
  handOverExecution,
  mapError as mapEffectError,
  throwIfCancelled,
  type Unit,
  unit,
  withResourceAcquisition,
} from "./effect"
import type { Iterator as SeseragiIterator } from "./iterator"
import { type Either, Left, type Maybe, Right } from "./sum"
import type { RuntimeDictionary } from "./traversable"

const streamBrand: unique symbol = Symbol("seseragi.stream")
const bufferCapacityBrand: unique symbol = Symbol("seseragi.buffer-capacity")

export type BufferCapacityError = Readonly<{
  tag: "NonPositiveBufferCapacity"
  value: number
}>

export type BufferCapacity = Readonly<{
  readonly [bufferCapacityBrand]: true
  readonly value: number
}>

/**
 * `next` runs in the context it is given, or else in the one the stream was
 * opened with. A stream that pulls its sources concurrently, as `zip` does,
 * gives each pull its own context, so it can stop a pull and still close the
 * cursor in the context the stream was opened with.
 */
export type StreamCursor<Value> = Readonly<{
  next: (context?: EffectContext) => Promise<IteratorResult<Value>>
  close: () => Promise<void>
}>

export type Stream<Environment, Failure, Value> = Readonly<{
  readonly [streamBrand]: (failure: Failure) => Environment
  readonly open: (
    environment: Environment,
    context: EffectContext
  ) => Promise<StreamCursor<Value>> | StreamCursor<Value>
}>

/**
 * Shared pull bridge for Provider adapters. Each call to `pull` represents one
 * positive unit of downstream demand. The factory is invoked independently for
 * every terminal execution, and `close` must be idempotent.
 */
export type PullStreamSource<Value> = Readonly<{
  pull: (context: EffectContext) => Promise<IteratorResult<Value>>
  close: () => void | Promise<void>
}>

export function fromPull<Environment, Failure, Value>(
  factory: (
    environment: Environment,
    context: EffectContext
  ) => Promise<PullStreamSource<Value>> | PullStreamSource<Value>
): Stream<Environment, Failure, Value> {
  return makeStream(async (environment, context) => {
    const source = await factory(environment, context)
    let closed = false
    const close = async (): Promise<void> => {
      if (closed) return
      closed = true
      await source.close()
    }
    return Object.freeze({
      async next(pull = context) {
        if (closed) return done()
        throwIfCancelled(pull)
        return awaitEffectValue(source.pull(pull), pull)
      },
      close,
    })
  })
}

export const empty: Stream<unknown, never, never> = /* @__PURE__ */ makeStream(
  () => cursor(async () => done())
)

export function singleton<Value>(value: Value): Stream<unknown, never, Value> {
  return makeStream(() => {
    let emitted = false
    return cursor(async () => {
      if (emitted) return done()
      emitted = true
      return item(value)
    })
  })
}

export function fromArray<Value>(
  values: ReadonlyArray<Value>
): Stream<unknown, never, Value> {
  const snapshot = values.slice()
  return makeStream(() => {
    let index = 0
    return cursor(async () =>
      index < snapshot.length ? item(snapshot[index++] as Value) : done()
    )
  })
}

export function fromIterable<Collection, Value>(
  values: Collection,
  dictionary: Iterable<Collection, Value> | RuntimeDictionary
): Stream<unknown, never, Value> {
  return makeStream(() => {
    let iterator: SeseragiIterator<Value> = (
      dictionary as Iterable<Collection, Value>
    ).iterate(values)
    return cursor(async () => {
      const result = iterator.next()
      if (result.tag === "Nothing") return done()
      const [value, next] = result.value
      iterator = next
      return item(value)
    })
  })
}

export function fromEffect<Environment, Failure, Value>(
  effect: Effect<Environment, Failure, Value>
): Stream<Environment, Failure, Value> {
  return makeStream((environment, context) => {
    let state: "pending" | "emitted" = "pending"
    return cursor(async (pull = context) => {
      if (state === "emitted") return done()
      state = "emitted"
      return item(await awaitEffectValue(effect(environment, pull), pull))
    })
  })
}

export function unfold<State, Value>(
  step: (state: State) => Maybe<readonly [Value, State]>,
  initial: State
): Stream<unknown, never, Value> {
  return makeStream(() => {
    let state = initial
    return cursor(async () => {
      const result = step(state)
      if (result.tag === "Nothing") return done()
      const [value, next] = result.value
      state = next
      return item(value)
    })
  })
}

export function map<Environment, Failure, Value, Result>(
  mapper: (value: Value) => Result,
  source: Stream<Environment, Failure, Value>
): Stream<Environment, Failure, Result> {
  return transform(source, async (sourceCursor, pull) => {
    const result = await sourceCursor.next(pull)
    return result.done ? done() : item(mapper(result.value))
  })
}

export function filter<Environment, Failure, Value>(
  predicate: (value: Value) => boolean,
  source: Stream<Environment, Failure, Value>
): Stream<Environment, Failure, Value> {
  return transform(source, async (sourceCursor, pull) => {
    while (true) {
      const result = await sourceCursor.next(pull)
      if (result.done || predicate(result.value)) return result
    }
  })
}

export function filterMap<Environment, Failure, Value, Result>(
  mapper: (value: Value) => Maybe<Result>,
  source: Stream<Environment, Failure, Value>
): Stream<Environment, Failure, Result> {
  return transform(source, async (sourceCursor, pull) => {
    while (true) {
      const result = await sourceCursor.next(pull)
      if (result.done) return done()
      const mapped = mapper(result.value)
      if (mapped.tag === "Just") return item(mapped.value)
    }
  })
}

export function mapError<Environment, Failure, NextFailure, Value>(
  mapper: (failure: Failure) => NextFailure,
  source: Stream<Environment, Failure, Value>
): Stream<Environment, NextFailure, Value> {
  return makeStream(async (environment, context) => {
    const sourceCursor = await source.open(environment, context)
    return Object.freeze({
      next: async (pull = context) =>
        await mapEffectError(mapper, ((_environment, pullContext) =>
          sourceCursor.next(pullContext)) as Effect<
          Environment,
          Failure,
          IteratorResult<Value>
        >)(environment, pull),
      close: sourceCursor.close,
    })
  })
}

export function flatMap<Environment, Failure, Value, Result>(
  mapper: (value: Value) => Stream<Environment, Failure, Result>,
  source: Stream<Environment, Failure, Value>
): Stream<Environment, Failure, Result> {
  return makeStream(async (environment, context) => {
    const outer = await source.open(environment, context)
    let inner: OwnedCursor<Result> | undefined
    // A close that comes while the inner stream is closing waits for it.
    let innerClosing: Promise<void> = Promise.resolve()
    const closeInner = (): Promise<void> => {
      if (inner !== undefined) {
        innerClosing = inner.close()
        inner = undefined
      }
      return innerClosing
    }
    return cursor(
      async (pull = context) => {
        while (true) {
          if (inner !== undefined) {
            const result = await inner.cursor.next(pull)
            if (!result.done) return result
            await closeInner()
          }
          // The pull for the next value and the inner stream it opens share
          // one execution, so the inner cursor's close can still use what
          // that pull acquired.
          const opened = await duringPull(context, pull, async (element) => {
            const result = await outer.next(element)
            return result.done
              ? undefined
              : await mapper(result.value).open(environment, element)
          })
          if (opened === undefined) return done()
          inner = opened
        }
      },
      async () => {
        await closeAll([{ close: closeInner }, outer])
      }
    )
  })
}

export function take<Environment, Failure, Value>(
  count: number,
  source: Stream<Environment, Failure, Value>
): Stream<Environment, Failure, Value> {
  return makeStream(async (environment, context) => {
    const sourceCursor = await source.open(environment, context)
    let remaining = Math.max(0, count)
    let closed = false
    const close = async (): Promise<void> => {
      if (closed) return
      closed = true
      await sourceCursor.close()
    }
    return Object.freeze({
      async next(pull = context) {
        if (remaining <= 0) {
          await close()
          return done()
        }
        const result = await sourceCursor.next(pull)
        if (result.done) {
          await close()
          return result
        }
        remaining -= 1
        return result
      },
      close,
    })
  })
}

export function drop<Environment, Failure, Value>(
  count: number,
  source: Stream<Environment, Failure, Value>
): Stream<Environment, Failure, Value> {
  return makeStream(async (environment, context) => {
    const sourceCursor = await source.open(environment, context)
    let remaining = Math.max(0, count)
    return cursor(async (pull = context) => {
      while (remaining > 0) {
        const skipped = await sourceCursor.next(pull)
        if (skipped.done) return skipped
        remaining -= 1
      }
      return sourceCursor.next(pull)
    }, sourceCursor.close)
  })
}

/**
 * Groups adjacent values into Arrays of `size`, keeping a shorter final group.
 * Each group pulls only the values it needs; an upstream failure propagates
 * without emitting the partial group.
 */
export function chunk<Environment, Failure, Value>(
  size: number,
  source: Stream<Environment, Failure, Value>
): Either<SizeError, Stream<Environment, Failure, ReadonlyArray<Value>>> {
  if (size <= 0) return Left(NonPositiveSize(size))
  return Right(
    makeStream(async (environment, context) => {
      const sourceCursor = await source.open(environment, context)
      let finished = false
      return cursor(async (pull = context) => {
        const values: Value[] = []
        while (!finished && values.length < size) {
          const result = await sourceCursor.next(pull)
          if (result.done) finished = true
          else values.push(result.value)
        }
        return values.length === 0 ? done() : item(values)
      }, sourceCursor.close)
    })
  )
}

export function concat<Environment, Failure, Value>(
  suffix: Stream<Environment, Failure, Value>,
  prefix: Stream<Environment, Failure, Value>
): Stream<Environment, Failure, Value> {
  return makeStream(async (environment, context) => {
    const prefixCursor = await prefix.open(environment, context)
    let current: OwnedCursor<Value> = {
      cursor: prefixCursor,
      close: prefixCursor.close,
    }
    let suffixStarted = false
    return cursor(
      async (pull = context) => {
        const result = await current.cursor.next(pull)
        if (!result.done || suffixStarted) return result
        await current.close()
        suffixStarted = true
        current = (await duringPull(context, pull, async (opening) =>
          suffix.open(environment, opening)
        )) as OwnedCursor<Value>
        return current.cursor.next(pull)
      },
      () => current.close()
    )
  })
}

export function zip<Environment, Failure, LeftValue, RightValue>(
  right: Stream<Environment, Failure, RightValue>,
  left: Stream<Environment, Failure, LeftValue>
): Stream<Environment, Failure, readonly [LeftValue, RightValue]> {
  return makeStream(async (environment, context) => {
    const leftBranch = await openMergeBranch(left, environment, context)
    const rightBranch = await openMergeBranch(
      right,
      environment,
      context
    ).catch((error) => cleanupThenThrow(error, [() => closeBranch(leftBranch)]))
    let closed = false
    // Both sides are pulled at once, so a close can start while another is
    // still running; the second one waits for the first to finish.
    const close = onceAsync(async (): Promise<void> => {
      closed = true
      await closeBranches([leftBranch, rightBranch])
    })
    return Object.freeze({
      async next() {
        if (closed) return done()
        const branches: ReadonlyArray<MergeBranch<unknown>> = [
          leftBranch,
          rightBranch,
        ]
        for (const branch of branches) startMergeDemand(branch)
        let turnEnded = false
        while (true) {
          const settled = branches.map((branch) => branch.settled)
          const stops = settled.some(
            (outcome) =>
              outcome?.kind === "failure" ||
              (outcome?.kind === "result" && outcome.value.done)
          )
          if (stops && !turnEnded && settled.includes(undefined)) {
            // Before stopping the other side, let it finish what it can in
            // this turn, as native code waits until every other Fiber has
            // reached a wait. A Fiber that yielded resumes only afterwards,
            // so the stop cancels it first.
            turnEnded = true
            await awaitTurnEnd()
            continue
          }
          // A failure wins over an end, and the left failure over the right
          // one. Either stops the pull still pending on the other side.
          const failure = settled.find((outcome) => outcome?.kind === "failure")
          if (failure?.kind === "failure") {
            return await cleanupThenThrow(failure.error, [close])
          }
          if (
            settled.some(
              (outcome) => outcome?.kind === "result" && outcome.value.done
            )
          ) {
            await close()
            return done()
          }
          const [leftResult, rightResult] = settled
          if (leftResult?.kind === "result" && rightResult?.kind === "result") {
            for (const branch of branches) {
              branch.pending = undefined
              branch.settled = undefined
            }
            const leftValue = leftResult.value as IteratorYieldResult<LeftValue>
            const rightValue =
              rightResult.value as IteratorYieldResult<RightValue>
            return item([leftValue.value, rightValue.value] as const)
          }
          await Promise.race(
            branches.flatMap((branch) =>
              branch.settled === undefined && branch.pending !== undefined
                ? [branch.pending]
                : []
            )
          )
          // Let every completion from the same scheduler turn become visible
          // before the checks above choose.
          await settleMicrotasks()
        }
      },
      close,
    })
  })
}

export function merge<Environment, Failure, Value>(
  right: Stream<Environment, Failure, Value>,
  left: Stream<Environment, Failure, Value>
): Stream<Environment, Failure, Value> {
  return makeStream(async (environment, context) => {
    const leftBranch = await openMergeBranch(left, environment, context)
    const rightBranch = await openMergeBranch(
      right,
      environment,
      context
    ).catch((error) => cleanupThenThrow(error, [() => closeBranch(leftBranch)]))
    const branches = [leftBranch, rightBranch] as const
    let closed = false
    // Both sides are pulled at once, so a close can start while another is
    // still running; the second one waits for the first to finish.
    const close = onceAsync(async (): Promise<void> => {
      closed = true
      await closeBranches(branches)
    })
    return Object.freeze({
      async next() {
        if (closed) return done()
        while (true) {
          for (const branch of branches) startMergeDemand(branch)
          const pending = branches.flatMap((branch) =>
            branch.pending === undefined ? [] : [branch.pending]
          )
          if (pending.length === 0) {
            await close()
            return done()
          }
          await Promise.race(pending)
          // Let every completion from the same scheduler turn become visible;
          // the ordered scan below then gives left deterministic priority.
          await settleMicrotasks()
          const failure = branches.find(
            (branch) => branch.settled?.kind === "failure"
          )?.settled
          if (failure?.kind === "failure") {
            return await cleanupThenThrow(failure.error, [close])
          }
          const ready = branches.find(
            (branch) => branch.settled?.kind === "result"
          )
          if (ready?.settled?.kind === "result") {
            const result = ready.settled.value
            ready.pending = undefined
            ready.settled = undefined
            if (result.done) {
              ready.done = true
              continue
            }
            return result
          }
        }
      },
      close,
    })
  })
}

export function NonPositiveBufferCapacity(value: number): BufferCapacityError {
  return Object.freeze({ tag: "NonPositiveBufferCapacity", value })
}

export function bufferCapacity(
  value: number
): Either<BufferCapacityError, BufferCapacity> {
  return Number.isSafeInteger(value) && value > 0
    ? Right(Object.freeze({ value }) as BufferCapacity)
    : Left(NonPositiveBufferCapacity(value))
}

export function buffer<Environment, Failure, Value>(
  capacity: BufferCapacity,
  source: Stream<Environment, Failure, Value>
): Stream<Environment, Failure, Value> {
  return makeStream(async (environment, context) => {
    const branch = await openBranch(source, environment, context)
    const values: Value[] = []
    let terminal: "open" | "complete" = "open"
    let failure: unknown
    let change = deferredSignal()
    let space = deferredSignal()
    let closed = false
    const notifyChange = (): void => {
      change.resolve()
      change = deferredSignal()
    }
    const notifySpace = (): void => {
      space.resolve()
      space = deferredSignal()
    }
    const producer = (async () => {
      try {
        while (!closed) {
          while (!closed && values.length >= capacity.value) {
            await awaitEffectValue(space.promise, branch.execution.context)
          }
          if (closed) return
          const result = await awaitEffectValue(
            pullBranch(branch),
            branch.execution.context
          )
          if (result.done) {
            terminal = "complete"
            await settleBranch(branch, "close")
            notifyChange()
            return
          }
          values.push(result.value)
          notifyChange()
        }
      } catch (error) {
        if (!closed) {
          values.length = 0
          failure =
            terminal === "complete"
              ? error
              : await errorAfterCleanup(error, [() => closeBranch(branch)])
          notifyChange()
        }
      }
    })()
    void producer.catch(() => undefined)

    const close = async (): Promise<void> => {
      if (closed) return
      closed = true
      values.length = 0
      notifySpace()
      notifyChange()
      await runCleanups([() => closeBranch(branch), () => producer])
    }
    return Object.freeze({
      async next(pull = context) {
        while (true) {
          if (values.length > 0) {
            const value = values.shift() as Value
            notifySpace()
            return item(value)
          }
          if (failure !== undefined) throw failure
          if (terminal === "complete" || closed) return done()
          await awaitEffectValue(change.promise, pull)
        }
      },
      close,
    })
  })
}

export function runCollect<Environment, Failure, Value>(
  source: Stream<Environment, Failure, Value>
): Effect<Environment, Failure, ReadonlyArray<Value>> {
  return terminal(source, async (sourceCursor, _environment, context) => {
    const values: Value[] = []
    while (true) {
      const result = await awaitEffectValue(sourceCursor.next(), context)
      if (result.done) return values
      values.push(result.value)
    }
  })
}

export function runFold<Environment, Failure, Value, Result>(
  initial: Result,
  step: (result: Result) => (value: Value) => Result,
  source: Stream<Environment, Failure, Value>
): Effect<Environment, Failure, Result> {
  return terminal(source, async (sourceCursor, _environment, context) => {
    let result = initial
    while (true) {
      const next = await awaitEffectValue(sourceCursor.next(), context)
      if (next.done) return result
      result = step(result)(next.value)
    }
  })
}

export function runForEach<Environment, Failure, Value>(
  action: (value: Value) => Effect<Environment, Failure, Unit>,
  source: Stream<Environment, Failure, Value>
): Effect<Environment, Failure, Unit> {
  return terminal(source, async (sourceCursor, environment, context) => {
    while (true) {
      const result = await awaitEffectValue(sourceCursor.next(), context)
      if (result.done) return unit
      await awaitEffectValue(
        action(result.value)(environment, context),
        context
      )
    }
  })
}

// The dictionaries share these functions instead of spreading each other:
// a spread may run getters, so a bundler keeps an unused dictionary, and
// with it the stream code it refers to.
const streamMap =
  <Value, Result>(mapper: (value: Value) => Result) =>
  <Environment, Failure>(source: Stream<Environment, Failure, Value>) =>
    map(mapper, source)

const streamApply =
  <Environment, Failure, Value, Result>(
    functions: Stream<Environment, Failure, (value: Value) => Result>
  ) =>
  (values: Stream<Environment, Failure, Value>) =>
    flatMap((mapper) => map(mapper, values), functions)

export const streamFunctor = /* @__PURE__ */ Object.freeze({ map: streamMap })

export const streamApplicative = /* @__PURE__ */ Object.freeze({
  map: streamMap,
  pure: singleton,
  apply: streamApply,
})

export const streamMonad = /* @__PURE__ */ Object.freeze({
  map: streamMap,
  pure: singleton,
  apply: streamApply,
  flatMap:
    <Environment, Failure, Value, Result>(
      mapper: (value: Value) => Stream<Environment, Failure, Result>
    ) =>
    (source: Stream<Environment, Failure, Value>) =>
      flatMap(mapper, source),
})

function makeStream<Environment, Failure, Value>(
  open: Stream<Environment, Failure, Value>["open"]
): Stream<Environment, Failure, Value> {
  return Object.freeze({ open }) as Stream<Environment, Failure, Value>
}

function cursor<Value>(
  next: (context?: EffectContext) => Promise<IteratorResult<Value>>,
  close: () => Promise<void> = async () => undefined
): StreamCursor<Value> {
  return Object.freeze({ next, close: onceAsync(close) })
}

function transform<Environment, Failure, Value, Result>(
  source: Stream<Environment, Failure, Value>,
  next: (
    source: StreamCursor<Value>,
    context: EffectContext
  ) => Promise<IteratorResult<Result>>
): Stream<Environment, Failure, Result> {
  return makeStream(async (environment, context) => {
    const sourceCursor = await source.open(environment, context)
    return cursor(
      (pull = context) => next(sourceCursor, pull),
      sourceCursor.close
    )
  })
}

// A cursor opened during a pull, with what that pull holds for it.
type OwnedCursor<Value> = Readonly<{
  cursor: StreamCursor<Value>
  close: () => Promise<void>
}>

/*
 * Opens a stream during a pull, as `flatMap` opens its inner stream. When
 * the pull has a context of its own, the opening runs in a new execution:
 * stopping the pull stops the opening too, but once the stream is open, what
 * the opening acquired stays until its cursor has closed, and then as long as
 * `context` does. So the cursor's close can still use it.
 */
async function duringPull<Value>(
  context: EffectContext,
  pull: EffectContext,
  open: (context: EffectContext) => Promise<StreamCursor<Value> | undefined>
): Promise<OwnedCursor<Value> | undefined> {
  if (pull === context) {
    const opened = await open(context)
    return opened && { cursor: opened, close: opened.close }
  }
  const execution = createEffectExecution(context, true)
  const unlink = pull.onCancel(() => execution.cancel())
  // While opening, the execution belongs to the pull; afterwards, only the
  // pulls of the branches opened in it do.
  let opening = true
  const forget = runsPullsIn(context, () => [
    ...(opening ? [execution] : []),
    ...pullsUnder(execution.context),
  ])
  let opened: StreamCursor<Value> | undefined
  try {
    opened = await open(execution.context)
    opening = false
  } catch (error) {
    unlink()
    forget()
    return await cleanupThenThrow(error, [
      () => releaseOpening(execution, context),
    ])
  }
  unlink()
  if (opened === undefined) {
    forget()
    await releaseOpening(execution, context)
    return undefined
  }
  const cursor = opened
  return {
    cursor,
    close: () =>
      runCleanups([
        () => cursor.close(),
        () => {
          forget()
          return releaseOpening(execution, context)
        },
      ]),
  }
}

function terminal<Environment, Failure, Value, Result>(
  source: Stream<Environment, Failure, Value>,
  consume: (
    source: StreamCursor<Value>,
    environment: Environment,
    context: EffectContext
  ) => Promise<Result>
): Effect<Environment, Failure, Result> {
  return async (environment, parentContext) => {
    const parent = parentContext ?? createEffectExecution().context
    const execution = createEffectExecution(parent)
    let sourceCursor: StreamCursor<Value> | undefined
    let completed = false
    let result: Result | undefined
    let failure: { readonly error: unknown } | undefined
    try {
      sourceCursor = await source.open(environment, execution.context)
      result = await consume(sourceCursor, environment, execution.context)
      completed = true
    } catch (error) {
      failure = { error }
    }
    const cleanupDefects = await collectCleanupDefects([
      () => sourceCursor?.close(),
      () => (completed ? execution.close() : execution.cancel()),
    ])
    if (cleanupDefects.length > 0) {
      throw defectWithSuppressed([
        ...cleanupDefects,
        ...(failure === undefined ? [] : [failure.error]),
      ])
    }
    if (failure !== undefined) throw failure.error
    return result as Result
  }
}

async function releaseOpening(
  execution: EffectExecution,
  context: EffectContext
): Promise<void> {
  if (!handOverExecution(execution, context)) await execution.close()
}

/*
 * `zip`, `merge` and `buffer` pull each source in a branch, as native code
 * pulls it in a branch Fiber. The branch's execution holds what opening the
 * source acquired, and its cursor closes in that context. Each pull runs in
 * an execution of its own, a child of the branch's:
 *
 * - Stopping the branch cancels the pull still running, so it never resumes,
 *   and releases what it acquired. Then the cursor closes, while the branch's
 *   context is still live, and what opening acquired is released last. This
 *   is the native order. A pull still acquiring lets the cursor close while
 *   it stops, since the acquisition may be waiting for that close.
 * - What a finished pull acquired moves to the branch and stays until the
 *   branch ends. An execution that acquired nothing serves the next pull.
 * - Stopping a branch stops the pulls of the branches opened under it too,
 *   such as the sides of a nested `zip`, before any cursor closes.
 * - The pull's `onCancel` cleanups may share the source's close, so their
 *   failures are reported with the branch's, not absorbed.
 */
// A branch closes when its source has ended, stops when its consumer is
// done with it, and is cancelled with its parent.
type Settling = "close" | "stop" | "cancel"

type Branch<Value> = {
  readonly cursor: StreamCursor<Value>
  readonly execution: EffectExecution
  // The execution for the current pull, or the next one.
  pull: EffectExecution | undefined
  // Unlinks the branch from its parent.
  readonly detach: () => void
  // How the branch is being settled, once it is.
  settling: Settling | undefined
  // Set by the first settle, which every later one waits for.
  closing: Promise<void> | undefined
}

async function openBranch<Environment, Failure, Value>(
  source: Stream<Environment, Failure, Value>,
  environment: Environment,
  context: EffectContext
): Promise<Branch<Value>> {
  // The branch is linked to its parent here, not by createEffectExecution:
  // when the parent is cancelled, the branch settles in the native order,
  // like a branch its consumer stops.
  const execution = createEffectExecution()
  let cancel = (): Promise<void> => execution.cancel()
  const detachCancel = context.onCancel(() => cancel())
  let forget = (): void => undefined
  const detach = (): void => {
    detachCancel()
    forget()
  }
  try {
    const cursor = await source.open(environment, execution.context)
    const branch: Branch<Value> = {
      cursor,
      execution,
      pull: undefined,
      detach,
      settling: undefined,
      closing: undefined,
    }
    forget = runsPullsIn(context, () => branchPulls(branch))
    cancel = () => settleBranch(branch, "cancel")
    return branch
  } catch (error) {
    detach()
    return await cleanupThenThrow(error, [() => execution.cancel()])
  }
}

async function pullBranch<Value>(
  branch: Branch<Value>
): Promise<IteratorResult<Value>> {
  if (branch.pull === undefined || branch.pull.context.cancelled) {
    branch.pull = createEffectExecution(branch.execution.context, true)
  }
  const pull = branch.pull
  try {
    return await branch.cursor.next(pull.context)
  } finally {
    // What a finished pull acquired moves to the branch, and the next pull
    // gets a new execution.
    if (
      branch.pull === pull &&
      !pull.context.cancelled &&
      handOverExecution(pull, branch.execution.context)
    ) {
      branch.pull = undefined
    }
  }
}

// The pulls running in each context: those of the branches opened in it,
// and of the branches opened under those.
const pullsIn = new WeakMap<EffectContext, Set<() => EffectExecution[]>>()

function runsPullsIn(
  context: EffectContext,
  pulls: () => EffectExecution[]
): () => void {
  let entries = pullsIn.get(context)
  if (entries === undefined) {
    entries = new Set()
    pullsIn.set(context, entries)
  }
  entries.add(pulls)
  return (): void => {
    entries.delete(pulls)
  }
}

function pullsUnder(context: EffectContext): EffectExecution[] {
  return [...(pullsIn.get(context) ?? [])].flatMap((pulls) => pulls())
}

// The branch's pull and those of the branches opened under it.
function branchPulls(branch: Branch<unknown>): EffectExecution[] {
  return [
    ...(branch.pull === undefined ? [] : [branch.pull]),
    ...pullsUnder(branch.execution.context),
  ]
}

async function settleAll(
  work: ReadonlyArray<Promise<void> | undefined>
): Promise<void> {
  const outcomes = await Promise.allSettled(work)
  const defects = outcomes.flatMap((outcome) =>
    outcome.status === "rejected" ? [outcome.reason] : []
  )
  if (defects.length > 0) throw defectWithSuppressed(defects)
}

async function closeBranch(branch: Branch<unknown>): Promise<void> {
  await settleBranch(branch, "stop")
}

// A branch is settled once: a cancellation from a parent can start it while
// its consumer is closing it too.
function settleBranch(branch: Branch<unknown>, mode: Settling): Promise<void> {
  if (branch.closing === undefined) {
    branch.settling = mode
    branch.closing = runSettleBranch(branch, mode)
  } else if (
    (branch.settling === "close" && mode !== "close") ||
    (branch.settling === "stop" && mode === "cancel")
  ) {
    // The cursor closes only once. A branch closing normally that is then
    // stopped, or one closing that its parent then cancels, cancels its
    // execution, since the close may be waiting for that notice; what opening
    // acquired still waits for the close. The close then fails or not
    // depending on how far it had got, so its failure goes to those already
    // waiting for it, not to the one that came later.
    branch.settling = mode
    const closing = branch.closing
    branch.closing = runCleanups([
      () => branch.execution.cancel(),
      () => closing.catch(() => undefined),
    ])
  }
  return branch.closing
}

async function runSettleBranch(
  branch: Branch<unknown>,
  mode: Settling
): Promise<void> {
  // Until the cursor has closed, the branch's scope holds an acquisition, so
  // cancelling the branch meanwhile releases nothing that opening acquired.
  let closed = (): void => undefined
  if (!branch.execution.context.cancelled) {
    void withResourceAcquisition(
      branch.execution.context,
      () =>
        new Promise<void>((resolve) => {
          closed = resolve
        })
    )
  }
  const closeCursor = async (): Promise<void> => {
    try {
      await branch.cursor.close()
    } finally {
      closed()
    }
  }
  // The parent stays linked until the branch has settled, so its
  // cancellation still reaches a source that is closing.
  try {
    if (mode === "close") {
      await runCleanups([
        closeCursor,
        () => branch.pull?.close(),
        () => branch.execution.close(),
      ])
      return
    }
    // Every pull stops at once, so none resumes while another is stopping.
    const pulls = branchPulls(branch)
    // An acquisition cannot be stopped, and it may wait for the cursor to
    // close, so a pull still acquiring lets the cursor close meanwhile.
    const acquiring = pulls.some(executionAcquiring)
    const stopping = settleAll(pulls.map((pull) => pull.cancel()))
    // A parent's cancellation reaches the branch's execution as it reaches
    // any other, while the cursor closes.
    const closing =
      mode === "stop"
        ? closeCursor
        : () => settleAll([closeCursor(), branch.execution.cancel()])
    const steps: Cleanup[] = acquiring
      ? [() => settleAll([stopping, closing()])]
      : [() => stopping, closing]
    if (mode === "stop") steps.push(() => branch.execution.cancel())
    await runCleanups(steps)
  } finally {
    closed()
    branch.detach()
  }
}

async function closeBranches(
  branches: ReadonlyArray<Branch<unknown>>
): Promise<void> {
  const outcomes = await Promise.allSettled(branches.map(closeBranch))
  const defects = outcomes.flatMap((outcome) =>
    outcome.status === "rejected" ? [outcome.reason] : []
  )
  if (defects.length > 0) throw defectWithSuppressed(defects)
}

type MergeSettlement<Value> =
  | Readonly<{ kind: "result"; value: IteratorResult<Value> }>
  | Readonly<{ kind: "failure"; error: unknown }>

type MergeBranch<Value> = Branch<Value> & {
  pending: Promise<void> | undefined
  settled: MergeSettlement<Value> | undefined
  done: boolean
}

async function openMergeBranch<Environment, Failure, Value>(
  source: Stream<Environment, Failure, Value>,
  environment: Environment,
  context: EffectContext
): Promise<MergeBranch<Value>> {
  // The same object, not a copy: a parent's cancellation settles the branch
  // that openBranch returned.
  return Object.assign(await openBranch(source, environment, context), {
    pending: undefined,
    settled: undefined,
    done: false,
  })
}

function startMergeDemand<Value>(branch: MergeBranch<Value>): void {
  if (branch.done || branch.pending !== undefined) return
  branch.pending = pullBranch(branch).then(
    (value) => {
      branch.settled = { kind: "result", value }
    },
    (error: unknown) => {
      branch.settled = { kind: "failure", error }
    }
  )
}

async function closeAll(
  cursors: ReadonlyArray<Pick<StreamCursor<unknown>, "close"> | undefined>
): Promise<void> {
  const outcomes = await Promise.allSettled(
    cursors.map((source) => source?.close())
  )
  const defects = outcomes.flatMap((outcome) =>
    outcome.status === "rejected" ? [outcome.reason] : []
  )
  if (defects.length > 0) throw defectWithSuppressed(defects)
}

type Cleanup = () => void | Promise<void>

async function collectCleanupDefects(
  cleanups: ReadonlyArray<Cleanup>
): Promise<unknown[]> {
  const defects: unknown[] = []
  for (const cleanup of cleanups) {
    try {
      await cleanup()
    } catch (error) {
      defects.push(error)
    }
  }
  return defects
}

async function runCleanups(cleanups: ReadonlyArray<Cleanup>): Promise<void> {
  const defects = await collectCleanupDefects(cleanups)
  if (defects.length > 0) throw defectWithSuppressed(defects)
}

async function errorAfterCleanup(
  error: unknown,
  cleanups: ReadonlyArray<Cleanup>
): Promise<unknown> {
  const defects = await collectCleanupDefects(cleanups)
  return defects.length > 0 ? defectWithSuppressed([...defects, error]) : error
}

async function cleanupThenThrow(
  error: unknown,
  cleanups: ReadonlyArray<Cleanup>
): Promise<never> {
  throw await errorAfterCleanup(error, cleanups)
}

function onceAsync(action: () => Promise<void>): () => Promise<void> {
  let result: Promise<void> | undefined
  return () => {
    result ??= action()
    return result
  }
}

// Lets the completions a side's pull queues as microtasks run, however many
// layers of combinators it passes through, before a stream chooses between
// its sides. It does not wait for the next turn, so a side that is ready
// never waits for one that is not.
async function settleMicrotasks(): Promise<void> {
  for (let tick = 0; tick < 64; tick += 1) await undefined
}

function item<Value>(value: Value): IteratorResult<Value> {
  return { done: false, value }
}

function done<Value>(): IteratorResult<Value> {
  return { done: true, value: undefined }
}

function deferredSignal(): {
  readonly promise: Promise<void>
  readonly resolve: () => void
} {
  let resolve = (): void => undefined
  const promise = new Promise<void>((done) => {
    resolve = done
  })
  return { promise, resolve }
}
