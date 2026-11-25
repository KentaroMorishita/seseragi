import { defectWithSuppressed } from "./cleanup-defect"
import { type Iterable, NonPositiveSize, type SizeError } from "./collection"
import {
  awaitEffectValue,
  awaitTurnEnd,
  createEffectExecution,
  type Effect,
  type EffectContext,
  type EffectExecution,
  type EffectExecutionMark,
  interruptExecution,
  mapError as mapEffectError,
  markExecution,
  openingStream,
  releaseExecutionAfter,
  throwIfCancelled,
  type Unit,
  unit,
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

export type StreamCursor<Value> = Readonly<{
  next: () => Promise<IteratorResult<Value>>
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
      async next() {
        if (closed) return done()
        throwIfCancelled(context)
        return awaitEffectValue(source.pull(context), context)
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
    return cursor(async () => {
      if (state === "emitted") return done()
      state = "emitted"
      return item(await awaitEffectValue(effect(environment, context), context))
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
  return transform(source, async (sourceCursor) => {
    const result = await sourceCursor.next()
    return result.done ? done() : item(mapper(result.value))
  })
}

export function filter<Environment, Failure, Value>(
  predicate: (value: Value) => boolean,
  source: Stream<Environment, Failure, Value>
): Stream<Environment, Failure, Value> {
  return transform(source, async (sourceCursor) => {
    while (true) {
      const result = await sourceCursor.next()
      if (result.done || predicate(result.value)) return result
    }
  })
}

export function filterMap<Environment, Failure, Value, Result>(
  mapper: (value: Value) => Maybe<Result>,
  source: Stream<Environment, Failure, Value>
): Stream<Environment, Failure, Result> {
  return transform(source, async (sourceCursor) => {
    while (true) {
      const result = await sourceCursor.next()
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
      next: async () =>
        await mapEffectError(mapper, (() => sourceCursor.next()) as Effect<
          Environment,
          Failure,
          IteratorResult<Value>
        >)(environment, context),
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
    let inner: StreamCursor<Result> | undefined
    return cursor(
      async () => {
        while (true) {
          if (inner !== undefined) {
            const result = await inner.next()
            if (!result.done) return result
            await inner.close()
            inner = undefined
          }
          const result = await outer.next()
          if (result.done) return done()
          inner = await mapper(result.value).open(environment, context)
        }
      },
      async () => {
        await closeAll([inner, outer])
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
      async next() {
        if (remaining <= 0) {
          await close()
          return done()
        }
        const result = await sourceCursor.next()
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
    return cursor(async () => {
      while (remaining > 0) {
        const skipped = await sourceCursor.next()
        if (skipped.done) return skipped
        remaining -= 1
      }
      return sourceCursor.next()
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
      return cursor(async () => {
        const values: Value[] = []
        while (!finished && values.length < size) {
          const result = await sourceCursor.next()
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
    let current = await prefix.open(environment, context)
    let suffixStarted = false
    return cursor(
      async () => {
        const result = await current.next()
        if (!result.done || suffixStarted) return result
        await current.close()
        suffixStarted = true
        current = await suffix.open(environment, context)
        return current.next()
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
          await Promise.resolve()
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
          await Promise.resolve()
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
            branch.cursor.next(),
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
      async next() {
        while (true) {
          if (values.length > 0) {
            const value = values.shift() as Value
            notifySpace()
            return item(value)
          }
          if (failure !== undefined) throw failure
          if (terminal === "complete" || closed) return done()
          await awaitEffectValue(change.promise, context)
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

// What a stream registers while it opens stays until its cursor has closed,
// even when it opens during a pull of another stream.
function makeStream<Environment, Failure, Value>(
  open: Stream<Environment, Failure, Value>["open"]
): Stream<Environment, Failure, Value> {
  return Object.freeze({
    open: (environment: Environment, context: EffectContext) =>
      openingStream(context, () => open(environment, context)),
  }) as Stream<Environment, Failure, Value>
}

function cursor<Value>(
  next: () => Promise<IteratorResult<Value>>,
  close: () => Promise<void> = async () => undefined
): StreamCursor<Value> {
  return Object.freeze({ next, close: onceAsync(close) })
}

function transform<Environment, Failure, Value, Result>(
  source: Stream<Environment, Failure, Value>,
  next: (source: StreamCursor<Value>) => Promise<IteratorResult<Result>>
): Stream<Environment, Failure, Result> {
  return makeStream(async (environment, context) => {
    const sourceCursor = await source.open(environment, context)
    return cursor(() => next(sourceCursor), sourceCursor.close)
  })
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

type Branch<Value> = {
  readonly cursor: StreamCursor<Value>
  readonly execution: EffectExecution
  // What the branch had registered once its source was open. Later
  // registrations come from its pulls.
  readonly opened: EffectExecutionMark
  // Stops passing the parent's interrupt on to the branch.
  readonly detach: () => void
  // Set by the first close, which every later close waits for.
  closing: Promise<void> | undefined
}

async function openBranch<Environment, Failure, Value>(
  source: Stream<Environment, Failure, Value>,
  environment: Environment,
  context: EffectContext
): Promise<Branch<Value>> {
  const execution = createEffectExecution(context)
  // When an outer branch is stopped, a pull it waits on in this branch stops
  // at once too. Releasing this branch still waits for its own settle.
  const interrupt = (): void => interruptExecution(execution)
  context.signal.addEventListener("abort", interrupt, { once: true })
  const detach = (): void =>
    context.signal.removeEventListener("abort", interrupt)
  if (context.signal.aborted) interrupt()
  try {
    const cursor = await source.open(environment, execution.context)
    return {
      cursor,
      execution,
      opened: markExecution(execution),
      detach,
      closing: undefined,
    }
  } catch (error) {
    detach()
    return await cleanupThenThrow(error, [() => execution.cancel()])
  }
}

async function closeBranch(branch: Branch<unknown>): Promise<void> {
  await settleBranch(branch, "cancel")
}

// Cancelling follows the native order. It stops the branch first, so a
// pending pull never resumes, and releases what the pulls acquired. Then it
// closes the cursor, which may still use what opening a stream acquired,
// and releases that last. A stream opened during a pull, such as the inner
// stream of flatMap, counts as opening, not as the pull. A branch is settled
// once: a cancellation from a parent can start it while its consumer is
// closing it too.
function settleBranch(
  branch: Branch<unknown>,
  mode: "cancel" | "close"
): Promise<void> {
  branch.closing ??= runSettleBranch(branch, mode)
  return branch.closing
}

async function runSettleBranch(
  branch: Branch<unknown>,
  mode: "cancel" | "close"
): Promise<void> {
  branch.detach()
  // A branch stopped with its outer branch cancels, so its cleanups run.
  if (mode === "close" && !branch.execution.context.cancelled) {
    await runCleanups([
      () => branch.cursor.close(),
      () => branch.execution.close(),
    ])
    return
  }
  interruptExecution(branch.execution)
  await runCleanups([
    () => releaseExecutionAfter(branch.execution, branch.opened),
    () => branch.cursor.close(),
    () => branch.execution.cancel(),
  ])
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
  return {
    ...(await openBranch(source, environment, context)),
    pending: undefined,
    settled: undefined,
    done: false,
  }
}

function startMergeDemand<Value>(branch: MergeBranch<Value>): void {
  if (branch.done || branch.pending !== undefined) return
  branch.pending = branch.cursor.next().then(
    (value) => {
      branch.settled = { kind: "result", value }
    },
    (error: unknown) => {
      branch.settled = { kind: "failure", error }
    }
  )
}

async function closeAll(
  cursors: ReadonlyArray<StreamCursor<unknown> | undefined>
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
