import { defectWithSuppressed } from "./cleanup-defect"
import { createDuration, type Duration } from "./clock-value"
import type { Iterable, RuntimeDictionary } from "./collection"
import { type Either, Just, Left, type Maybe, Nothing, Right } from "./sum"

export type Unit = undefined

export type Effect<Environment, Failure, Success> = ((
  environment: Environment,
  context?: EffectContext
) => Promise<Success> | Success) & {
  readonly __failure?: Failure
}

export type EffectCancellationCleanup = () => void | Promise<void>
export type EffectResourceFinalizer = () => void | Promise<void>

export type EffectResourceRegistration = Readonly<{
  readonly ready: Promise<void>
  readonly unregister: () => void
}>

/**
 * Runner-owned lifecycle state shared by every operation in one Effect run.
 *
 * It is deliberately separate from Seseragi's typed failure channel. A
 * cancelled execution is a host lifecycle event, not a value user code can
 * catch with `mapError`.
 */
export type EffectContext = Readonly<{
  readonly signal: AbortSignal
  readonly cancelled: boolean
  readonly onCancel: (cleanup: EffectCancellationCleanup) => () => void
}>

export type EffectExecution = Readonly<{
  readonly context: EffectContext
  readonly cancel: () => Promise<void>
  readonly close: () => Promise<void>
}>

const effectContext = Symbol("seseragi.effect-context")

type ResourceFinalizerEntry = {
  active: boolean
  readonly finalizer: EffectResourceFinalizer
}

class ResourceScope {
  private readonly finalizers: ResourceFinalizerEntry[] = []
  private state: "open" | "closing" | "closed" = "open"
  private closing: Promise<void> | undefined

  register(finalizer: EffectResourceFinalizer): EffectResourceRegistration {
    if (this.state === "closed") {
      return Object.freeze({
        ready: runResourceFinalizer(finalizer),
        unregister: () => undefined,
      })
    }
    const entry: ResourceFinalizerEntry = { active: true, finalizer }
    this.finalizers.push(entry)
    return Object.freeze({
      ready: Promise.resolve(),
      unregister: () => {
        entry.active = false
      },
    })
  }

  close(): Promise<void> {
    if (this.closing !== undefined) return this.closing
    this.state = "closing"
    this.closing = this.drain()
    return this.closing
  }

  private async drain(): Promise<void> {
    const defects: unknown[] = []
    while (this.finalizers.length > 0) {
      const entry = this.finalizers.pop()
      if (entry === undefined || !entry.active) continue
      entry.active = false
      try {
        await entry.finalizer()
      } catch (error) {
        defects.push(normalizeFinalizerDefect(error))
      }
    }
    this.state = "closed"
    if (defects.length > 0) throw defectWithSuppressed(defects)
  }
}

const resourceScopes = new WeakMap<EffectContext, ResourceScope>()

function resourceScopeOf(context: EffectContext): ResourceScope {
  const existing = resourceScopes.get(context)
  if (existing !== undefined) return existing
  const scope = new ResourceScope()
  resourceScopes.set(context, scope)
  context.onCancel(() => scope.close())
  return scope
}

function contextWithResourceScope(
  context: EffectContext,
  scope: ResourceScope
): EffectContext {
  const child = Object.freeze({
    signal: context.signal,
    get cancelled() {
      return context.signal.aborted
    },
    onCancel: context.onCancel,
  })
  resourceScopes.set(child, scope)
  return child
}

function maskedContext(scope: ResourceScope): EffectContext {
  const controller = new AbortController()
  const context = Object.freeze({
    signal: controller.signal,
    cancelled: false,
    onCancel: (_cleanup: EffectCancellationCleanup) => () => undefined,
  })
  resourceScopes.set(context, scope)
  return context
}

/** Registers cleanup in the current lexical Effect resource scope. */
export function registerResourceFinalizer(
  context: EffectContext,
  finalizer: EffectResourceFinalizer
): EffectResourceRegistration {
  return resourceScopeOf(context).register(finalizer)
}

/** Attaches the shared lifecycle context to a host-owned environment record. */
export function attachEffectContext<Environment extends object>(
  environment: Environment,
  context: EffectContext
): Environment {
  Object.defineProperty(environment, effectContext, {
    configurable: false,
    enumerable: false,
    value: context,
    writable: false,
  })
  return environment
}

/** Retrieves the lifecycle context supplied to a host-owned environment. */
export function effectContextOf(
  environment: unknown
): EffectContext | undefined {
  if (typeof environment !== "object" || environment === null) {
    return undefined
  }
  return (environment as { readonly [effectContext]?: EffectContext })[
    effectContext
  ]
}

/** A runner-level cancellation signal; never a Seseragi typed failure. */
export class EffectCancellation extends Error {
  constructor() {
    super("effect execution cancelled")
    this.name = "EffectCancellation"
  }
}

export function isEffectCancellation(
  error: unknown
): error is EffectCancellation {
  return error instanceof EffectCancellation
}

export function throwIfCancelled(context: EffectContext): void {
  if (context.signal.aborted) {
    throw new EffectCancellation()
  }
}

/**
 * Creates an isolated cancellation scope for exactly one root Effect run.
 * Cleanup registered after abort joins the same drain while cancellation is
 * settling, including cleanup registered by another cleanup. The drain closes
 * only after one quiet event-loop turn with no tracked work.
 * Cleanup failures are intentionally absorbed: cancellation must not surface
 * as a stale host exception after the owning UI has moved on.
 */
export function createEffectExecution(
  parentContext?: EffectContext
): EffectExecution {
  const controller = new AbortController()
  const rootScope = new ResourceScope()
  const cleanups = new Set<EffectCancellationCleanup>()
  const started = new Set<EffectCancellationCleanup>()
  const pending = new Set<Promise<void>>()
  let cancellation: Promise<void> | undefined
  let cleanupEpoch = 0
  const startCleanup = (cleanup: EffectCancellationCleanup): void => {
    if (started.has(cleanup)) return
    started.add(cleanup)
    cleanupEpoch += 1
    cleanups.delete(cleanup)
    const task = runCancellationCleanup(cleanup)
    pending.add(task)
    void task.then(() => pending.delete(task))
  }
  const drainCleanups = async (): Promise<void> => {
    while (true) {
      for (const cleanup of [...cleanups]) startCleanup(cleanup)
      const snapshot = [...pending]
      if (snapshot.length > 0) {
        await Promise.all(snapshot)
        continue
      }
      const quietEpoch = cleanupEpoch
      await new Promise<void>((resolve) => setTimeout(resolve, 0))
      if (
        cleanups.size === 0 &&
        pending.size === 0 &&
        cleanupEpoch === quietEpoch
      ) {
        return
      }
    }
  }
  const context: EffectContext = Object.freeze({
    signal: controller.signal,
    get cancelled() {
      return controller.signal.aborted
    },
    onCancel(cleanup) {
      if (started.has(cleanup)) return () => undefined
      if (controller.signal.aborted) {
        startCleanup(cleanup)
        return () => undefined
      }
      cleanups.add(cleanup)
      return () => {
        cleanups.delete(cleanup)
      }
    },
  })
  resourceScopes.set(context, rootScope)
  let detachParent = (): void => undefined
  const execution = Object.freeze({
    context,
    cancel() {
      if (cancellation !== undefined) return cancellation
      detachParent()
      let resolveCancellation = (): void => undefined
      let rejectCancellation = (_error: unknown): void => undefined
      cancellation = new Promise<void>((resolve, reject) => {
        resolveCancellation = resolve
        rejectCancellation = reject
      })
      controller.abort()
      for (const cleanup of [...cleanups]) startCleanup(cleanup)
      void (async () => {
        await drainCleanups()
        await rootScope.close()
      })().then(resolveCancellation, rejectCancellation)
      return cancellation
    },
    close() {
      detachParent()
      return rootScope.close()
    },
  })
  if (parentContext !== undefined) {
    detachParent = parentContext.onCancel(() => execution.cancel())
  }
  return execution
}

export type EffectFailure<Failure> = {
  readonly kind: "failure"
  readonly error: Failure
}

export type EffectSuccess<Success> = {
  readonly kind: "success"
  readonly value: Success
}

export type EffectResult<Failure, Success> =
  | EffectFailure<Failure>
  | EffectSuccess<Success>

class TypedFailureSignal<Failure> {
  readonly error: Failure

  constructor(error: Failure) {
    this.error = error
  }
}

async function runResourceFinalizer(
  finalizer: EffectResourceFinalizer
): Promise<void> {
  try {
    await finalizer()
  } catch (error) {
    throw normalizeFinalizerDefect(error)
  }
}

function normalizeFinalizerDefect(error: unknown): unknown {
  if (error instanceof TypedFailureSignal) {
    return new TypeError("Effect finalizer produced a typed failure", {
      cause: error.error,
    })
  }
  return error
}

export const unit: Unit = undefined

export function succeed<Success>(
  value: Success
): Effect<unknown, never, Success> {
  return () => value
}

export function defer<Environment, Failure, Success>(
  thunk: (unit: Unit) => Effect<Environment, Failure, Success>
): Effect<Environment, Failure, Success> {
  return (environment, context) => thunk(unit)(environment, context)
}

export function flatMap<
  Environment,
  Failure,
  Success,
  NextEnvironment,
  NextFailure,
  NextSuccess,
>(
  effect: Effect<Environment, Failure, Success>,
  next: (value: Success) => Effect<NextEnvironment, NextFailure, NextSuccess>
): Effect<Environment & NextEnvironment, Failure | NextFailure, NextSuccess> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    const value = await awaitWithCancellation(
      effect(environment, activeContext),
      activeContext
    )
    return awaitWithCancellation(
      next(value)(environment, activeContext),
      activeContext
    )
  }
}

export const effectFunctor = Object.freeze({
  map:
    <Value, Result>(f: (value: Value) => Result) =>
    <Environment, Failure>(
      effect: Effect<Environment, Failure, Value>
    ): Effect<Environment, Failure, Result> =>
    async (environment, context) => {
      const activeContext = context ?? createEffectExecution().context
      return f(
        await awaitWithCancellation(
          effect(environment, activeContext),
          activeContext
        )
      )
    },
})

export const effectApplicative = Object.freeze({
  ...effectFunctor,
  pure: succeed,
  apply:
    <Environment, Failure, Value, Result>(
      functions: Effect<Environment, Failure, (value: Value) => Result>
    ) =>
    (
      values: Effect<Environment, Failure, Value>
    ): Effect<Environment, Failure, Result> =>
      flatMap(functions, (f) => effectFunctor.map(f)(values)),
})

export const effectMonad = Object.freeze({
  ...effectApplicative,
  flatMap:
    <Value, NextEnvironment, NextFailure, Result>(
      f: (value: Value) => Effect<NextEnvironment, NextFailure, Result>
    ) =>
    <Environment, Failure>(effect: Effect<Environment, Failure, Value>) =>
      flatMap(effect, f),
})

export function fail<Failure>(error: Failure): Effect<unknown, Failure, never> {
  return () => {
    throw new TypedFailureSignal(error)
  }
}

type EitherFailure<Value> = Value extends Left<infer Failure> ? Failure : never

type EitherSuccess<Value> = Value extends Right<infer Success> ? Success : never

export function fromEither<Value extends Either<unknown, unknown>>(
  value: Value
): Effect<unknown, EitherFailure<Value>, EitherSuccess<Value>>
export function fromEither(
  value: Either<unknown, unknown>
): Effect<unknown, unknown, unknown> {
  if (value.tag === "Right") {
    const success = value.value
    return succeed(success)
  }
  const failure = value.value
  return fail(failure)
}

export function fromMaybe<Failure, Success>(
  error: Failure,
  value: Maybe<Success>
): Effect<unknown, Failure, Success> {
  return value.tag === "Nothing" ? fail(error) : succeed(value.value)
}

export function attempt<Environment, Failure, Success>(
  effect: Effect<Environment, Failure, Success>
): Effect<Environment, never, Either<Failure, Success>> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    try {
      return Right(
        await awaitWithCancellation(
          effect(environment, activeContext),
          activeContext
        )
      )
    } catch (error) {
      if (isEffectCancellation(error)) throw error
      if (error instanceof TypedFailureSignal) {
        return Left(error.error as Failure)
      }
      throw error
    }
  }
}

export function mapError<Environment, Failure, NextFailure, Success>(
  mapper: (error: Failure) => NextFailure,
  effect: Effect<Environment, Failure, Success>
): Effect<Environment, NextFailure, Success> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    try {
      return await awaitWithCancellation(
        effect(environment, activeContext),
        activeContext
      )
    } catch (error) {
      if (isEffectCancellation(error)) {
        throw error
      }
      if (error instanceof TypedFailureSignal) {
        throw new TypedFailureSignal(mapper(error.error as Failure))
      }
      throw error
    }
  }
}

export function recover<Environment, Failure, NextFailure, Success>(
  handler: (error: Failure) => Effect<Environment, NextFailure, Success>,
  effect: Effect<Environment, Failure, Success>
): Effect<Environment, NextFailure, Success> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    try {
      return await awaitWithCancellation(
        effect(environment, activeContext),
        activeContext
      )
    } catch (error) {
      if (isEffectCancellation(error)) throw error
      if (error instanceof TypedFailureSignal) {
        return awaitWithCancellation(
          handler(error.error as Failure)(environment, activeContext),
          activeContext
        )
      }
      throw error
    }
  }
}

export function provide<Environment, Failure, Success>(
  environment: Environment,
  effect: Effect<Environment, Failure, Success>
): Effect<unknown, Failure, Success> {
  return (_outer, context) => effect(environment, context)
}

export function service<Environment, Success>(
  select: (environment: Environment) => Success
): Effect<Environment, never, Success> {
  return (environment) => select(environment)
}

export function provideSome<OuterEnvironment, Environment, Failure, Success>(
  project: (environment: OuterEnvironment) => Environment,
  effect: Effect<Environment, Failure, Success>
): Effect<OuterEnvironment, Failure, Success> {
  return (environment, context) => effect(project(environment), context)
}

export function acquireRelease<Environment, Failure, Resource>(
  acquire: Effect<Environment, Failure, Resource>,
  release: (resource: Resource) => Effect<Environment, never, Unit>
): Effect<Environment, Failure, Resource> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    throwIfCancelled(activeContext)
    // Acquisition and registration deliberately do not use
    // awaitWithCancellation. Once acquisition starts, cancellation is observed
    // only after a successful value has an owning finalizer.
    const resource = await acquire(environment, activeContext)
    const scope = resourceScopeOf(activeContext)
    const finalizerContext = maskedContext(scope)
    const registration = scope.register(() =>
      release(resource)(environment, finalizerContext)
    )
    await registration.ready
    throwIfCancelled(activeContext)
    return resource
  }
}

export function scoped<Environment, Failure, Success>(
  effect: Effect<Environment, Failure, Success>
): Effect<Environment, Failure, Success> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    const outerScope = resourceScopeOf(activeContext)
    const innerScope = new ResourceScope()
    const innerContext = contextWithResourceScope(activeContext, innerScope)
    const registration = outerScope.register(() => innerScope.close())
    await registration.ready
    try {
      return await awaitWithCancellation(
        effect(environment, innerContext),
        innerContext
      )
    } finally {
      registration.unregister()
      await innerScope.close()
    }
  }
}

export type FiberSucceeded<Success> = Readonly<{
  tag: "FiberSucceeded"
  value: Success
}>

export type FiberFailed<Failure> = Readonly<{
  tag: "FiberFailed"
  value: Failure
}>

export type FiberCancelled = Readonly<{ tag: "FiberCancelled" }>

export type FiberExit<Failure, Success> =
  | FiberSucceeded<Success>
  | FiberFailed<Failure>
  | FiberCancelled

const fiberBrand: unique symbol = Symbol("seseragi.fiber")

export type Fiber<Failure, Success> = Readonly<{
  readonly [fiberBrand]: (failure: Failure) => Success
}>

type FiberState<Failure, Success> = {
  readonly execution: EffectExecution
  readonly completion: Promise<FiberExit<Failure, Success>>
  exit: FiberExit<Failure, Success> | undefined
}

const fibers = new WeakMap<object, FiberState<unknown, unknown>>()

export function FiberSucceeded<Success>(
  value: Success
): FiberSucceeded<Success> {
  return Object.freeze({ tag: "FiberSucceeded", value })
}

export function FiberFailed<Failure>(value: Failure): FiberFailed<Failure> {
  return Object.freeze({ tag: "FiberFailed", value })
}

export const FiberCancelled: FiberCancelled = Object.freeze({
  tag: "FiberCancelled",
})

function fiberState<Failure, Success>(
  fiber: Fiber<Failure, Success>
): FiberState<Failure, Success> {
  const state = fibers.get(fiber)
  if (state === undefined) {
    throw new TypeError("Fiber value does not use the runtime brand")
  }
  return state as FiberState<Failure, Success>
}

async function interruptFiberState<Failure, Success>(
  state: FiberState<Failure, Success>
): Promise<void> {
  if (state.exit === undefined) await state.execution.cancel()
  await state.completion
}

function spawnFiber<Environment, Failure, Success>(
  effect: Effect<Environment, Failure, Success>,
  environment: Environment,
  context: EffectContext
): Fiber<Failure, Success> {
  throwIfCancelled(context)
  const execution = createEffectExecution(context)
  let resolveCompletion = (_exit: FiberExit<Failure, Success>): void =>
    undefined
  let rejectCompletion = (_error: unknown): void => undefined
  const completion = new Promise<FiberExit<Failure, Success>>(
    (resolve, reject) => {
      resolveCompletion = resolve
      rejectCompletion = reject
    }
  )
  void completion.catch(() => undefined)
  const state: FiberState<Failure, Success> = {
    execution,
    completion,
    exit: undefined,
  }
  const fiber = Object.freeze({}) as Fiber<Failure, Success>
  fibers.set(fiber, state as FiberState<unknown, unknown>)
  const registration = resourceScopeOf(context).register(() =>
    interruptFiberState(state)
  )

  void registration.ready.then(async () => {
    try {
      const value = await awaitWithCancellation(
        effect(environment, execution.context),
        execution.context
      )
      await execution.close()
      state.exit = FiberSucceeded(value)
      resolveCompletion(state.exit)
    } catch (error) {
      if (isEffectCancellation(error)) {
        try {
          await execution.cancel()
          state.exit = FiberCancelled
          resolveCompletion(state.exit)
        } catch (cleanupDefect) {
          rejectCompletion(cleanupDefect)
        }
      } else if (error instanceof TypedFailureSignal) {
        try {
          await execution.close()
          state.exit = FiberFailed(error.error as Failure)
          resolveCompletion(state.exit)
        } catch (cleanupDefect) {
          rejectCompletion(cleanupDefect)
        }
      } else {
        try {
          await execution.close()
          rejectCompletion(error)
        } catch (cleanupDefect) {
          rejectCompletion(cleanupDefect)
        }
      }
    } finally {
      registration.unregister()
    }
  }, rejectCompletion)
  return fiber
}

export function fork<Environment, Failure, Success>(
  effect: Effect<Environment, Failure, Success>
): Effect<Environment, never, Fiber<Failure, Success>> {
  return (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    return spawnFiber(effect, environment, activeContext)
  }
}

export function awaitFiber<Failure, Success>(
  fiber: Fiber<Failure, Success>
): Effect<unknown, never, FiberExit<Failure, Success>> {
  return (_environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    return awaitWithCancellation(
      fiberState<Failure, Success>(fiber).completion,
      activeContext
    )
  }
}

export function poll<Failure, Success>(
  fiber: Fiber<Failure, Success>
): Effect<unknown, never, Maybe<FiberExit<Failure, Success>>> {
  return () => {
    const exit = fiberState<Failure, Success>(fiber).exit
    return exit === undefined ? Nothing : Just(exit)
  }
}

export function join<Failure, Success>(
  fiber: Fiber<Failure, Success>
): Effect<unknown, Failure, Success> {
  return async (_environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    const exit = await awaitWithCancellation(
      fiberState<Failure, Success>(fiber).completion,
      activeContext
    )
    switch (exit.tag) {
      case "FiberSucceeded":
        return exit.value
      case "FiberFailed":
        throw new TypedFailureSignal(exit.value)
      case "FiberCancelled":
        throw new EffectCancellation()
    }
  }
}

export function interrupt<Failure, Success>(
  fiber: Fiber<Failure, Success>
): Effect<unknown, never, Unit> {
  return async (_environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    await awaitWithCancellation(
      interruptFiberState(fiberState<Failure, Success>(fiber)),
      activeContext
    )
    return undefined
  }
}

export function yieldNow(): Effect<unknown, never, Unit> {
  return async (_environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    await awaitWithCancellation(
      new Promise<void>((resolve) => setTimeout(resolve, 0)),
      activeContext
    )
    return undefined
  }
}

function resumeFiberExit<Failure, Success>(
  exit: FiberExit<Failure, Success>
): Success {
  switch (exit.tag) {
    case "FiberSucceeded":
      return exit.value
    case "FiberFailed":
      throw new TypedFailureSignal(exit.value)
    case "FiberCancelled":
      throw new EffectCancellation()
  }
}

export function race<Environment, Failure, Success>(
  left: Effect<Environment, Failure, Success>,
  right: Effect<Environment, Failure, Success>
): Effect<Environment, Failure, Success> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    const leftFiber = spawnFiber(left, environment, activeContext)
    const rightFiber = spawnFiber(right, environment, activeContext)
    const leftState = fiberState<Failure, Success>(leftFiber)
    const rightState = fiberState<Failure, Success>(rightFiber)
    try {
      const winner = await awaitWithCancellation(
        Promise.race([
          leftState.completion.then((exit) => ({
            side: "left" as const,
            exit,
          })),
          rightState.completion.then((exit) => ({
            side: "right" as const,
            exit,
          })),
        ]),
        activeContext
      )
      if (winner.side === "left") {
        await interruptFiberState(rightState)
      } else {
        await interruptFiberState(leftState)
      }
      return resumeFiberExit(winner.exit)
    } catch (error) {
      await Promise.allSettled([
        interruptFiberState(leftState),
        interruptFiberState(rightState),
      ])
      throw error
    }
  }
}

export function parallel<Environment, Failure, Success>(
  effects: ReadonlyArray<Effect<Environment, Failure, Success>>
): Effect<Environment, Failure, ReadonlyArray<Success>> {
  return (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    return traverseParallelValues(
      Number.POSITIVE_INFINITY,
      (effect) => effect,
      effects,
      environment,
      activeContext
    )
  }
}

export type ParallelismError = Readonly<{
  tag: "NonPositiveParallelism"
  value: number
}>

const parallelismBrand: unique symbol = Symbol("seseragi.parallelism")

export type Parallelism = Readonly<{
  readonly [parallelismBrand]: true
  readonly limit: number
}>

export function NonPositiveParallelism(value: number): ParallelismError {
  return Object.freeze({ tag: "NonPositiveParallelism", value })
}

export function parallelism(
  value: number
): Either<ParallelismError, Parallelism> {
  return Number.isSafeInteger(value) && value > 0
    ? Right(Object.freeze({ limit: value }) as Parallelism)
    : Left(NonPositiveParallelism(value))
}

const unlimitedParallelism = Object.freeze({
  limit: Number.POSITIVE_INFINITY,
}) as Parallelism

export function unboundedParallelism(_unit?: Unit): Parallelism {
  return unlimitedParallelism
}

type RuntimeReducible<Collection, Element> = Readonly<{
  reduce: <Accumulator>(
    initial: Accumulator
  ) => (
    step: (accumulator: Accumulator) => (value: Element) => Accumulator
  ) => (values: Collection) => Accumulator
}>

function reducibleValues<Collection, Element>(
  dictionary: RuntimeReducible<Collection, Element>,
  values: Collection
): ReadonlyArray<Element> {
  return dictionary.reduce<Element[]>([])((items) => (value) => {
    items.push(value)
    return items
  })(values)
}

async function traverseParallelValues<Environment, Failure, Element, Success>(
  limit: number,
  action: (value: Element) => Effect<Environment, Failure, Success>,
  values: ReadonlyArray<Element>,
  environment: Environment,
  context: EffectContext
): Promise<ReadonlyArray<Success>> {
  const results: Success[] = new Array(values.length)
  const active = new Map<number, FiberState<Failure, Success>>()
  let nextIndex = 0
  const start = (index: number): void => {
    const fiber = spawnFiber(
      action(values[index] as Element),
      environment,
      context
    )
    active.set(index, fiberState<Failure, Success>(fiber))
  }
  const fill = (): void => {
    while (nextIndex < values.length && active.size < limit) {
      start(nextIndex)
      nextIndex += 1
    }
  }

  try {
    fill()
    while (active.size > 0) {
      await awaitWithCancellation(
        Promise.race([...active.values()].map((state) => state.completion)),
        context
      )
      await Promise.resolve()
      const settled = [...active.entries()]
        .filter(([, state]) => state.exit !== undefined)
        .sort(([leftIndex], [rightIndex]) => leftIndex - rightIndex)
      const failed = settled.find(
        ([, state]) => state.exit?.tag === "FiberFailed"
      )
      if (failed !== undefined) {
        const exit = failed[1].exit as FiberFailed<Failure>
        throw new TypedFailureSignal(exit.value)
      }
      const cancelled = settled.find(
        ([, state]) => state.exit?.tag === "FiberCancelled"
      )
      if (cancelled !== undefined) throw new EffectCancellation()
      for (const [index, state] of settled) {
        const exit = state.exit as FiberSucceeded<Success>
        results[index] = exit.value
        active.delete(index)
      }
      fill()
    }
    return results
  } catch (error) {
    await Promise.allSettled(
      [...active.values()].map((state) => interruptFiberState(state))
    )
    throw error
  }
}

/** Explicit normal-success control for sequential Effect traversal. */
export type LoopControl = Readonly<{ tag: "Continue" } | { tag: "Break" }>
export const Continue: LoopControl = Object.freeze({ tag: "Continue" })
export const Break: LoopControl = Object.freeze({ tag: "Break" })

export function forEachUntil<Collection, Environment, Failure, Element>(
  action: (value: Element) => Effect<Environment, Failure, LoopControl>,
  values: Collection,
  dictionary: RuntimeDictionary
): Effect<Environment, Failure, Unit> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    throwIfCancelled(activeContext)
    const iterable = dictionary as Iterable<Collection, Element>
    let iterator = iterable.iterate(values)
    while (true) {
      throwIfCancelled(activeContext)
      const step = iterator.next()
      if (step.tag === "Nothing") return undefined
      const [value, rest] = step.value
      throwIfCancelled(activeContext)
      const control = await awaitWithCancellation(
        action(value)(environment, activeContext),
        activeContext
      )
      if (control.tag === "Break") return undefined
      iterator = rest
    }
  }
}

export function forEachParallel<Collection, Environment, Failure, Element>(
  concurrency: Parallelism,
  action: (value: Element) => Effect<Environment, Failure, Unit>,
  values: Collection,
  dictionary: RuntimeReducible<Collection, Element>
): Effect<Environment, Failure, Unit> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    await traverseParallelValues(
      concurrency.limit,
      action,
      reducibleValues(dictionary, values),
      environment,
      activeContext
    )
    return undefined
  }
}

export function traverseParallel<
  Collection,
  Environment,
  Failure,
  Element,
  Success,
>(
  concurrency: Parallelism,
  action: (value: Element) => Effect<Environment, Failure, Success>,
  values: Collection,
  dictionary: RuntimeReducible<Collection, Element>
): Effect<Environment, Failure, ReadonlyArray<Success>> {
  return (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    return traverseParallelValues(
      concurrency.limit,
      action,
      reducibleValues(dictionary, values),
      environment,
      activeContext
    )
  }
}

export type ClockRequirement = Readonly<{
  clock: Readonly<{
    sleep: (duration: Duration, context: EffectContext) => Promise<Unit>
  }>
}>

export type ScheduleStop = Readonly<{ tag: "ScheduleStop" }>
export type ScheduleContinue = Readonly<{
  tag: "ScheduleContinue"
  value: Duration
}>
export type ScheduleDecision = ScheduleStop | ScheduleContinue
export type ScheduleError = Readonly<{
  tag: "NegativeRecurrences"
  value: number
}>

const scheduleBrand: unique symbol = Symbol("seseragi.schedule")
export type Schedule<Input> = Readonly<{
  readonly [scheduleBrand]: true
  readonly decide: (observation: number, input: Input) => ScheduleDecision
}>

export const ScheduleStop: ScheduleStop = Object.freeze({
  tag: "ScheduleStop",
})

export function ScheduleContinue(value: Duration): ScheduleContinue {
  return Object.freeze({ tag: "ScheduleContinue", value })
}

export function NegativeRecurrences(value: number): ScheduleError {
  return Object.freeze({ tag: "NegativeRecurrences", value })
}

export function schedule<Input>(
  decide: (observation: number, input: Input) => ScheduleDecision
): Schedule<Input> {
  return Object.freeze({ decide }) as Schedule<Input>
}

export function recurs<Input>(
  additionalRuns: number
): Either<ScheduleError, Schedule<Input>> {
  if (!Number.isSafeInteger(additionalRuns) || additionalRuns < 0) {
    return Left(NegativeRecurrences(additionalRuns))
  }
  return Right(
    schedule((observation) =>
      observation <= additionalRuns
        ? ScheduleContinue(createDuration(0n))
        : ScheduleStop
    )
  )
}

export function spaced<Input>(
  additionalRuns: number,
  delay: Duration
): Either<ScheduleError, Schedule<Input>> {
  if (!Number.isSafeInteger(additionalRuns) || additionalRuns < 0) {
    return Left(NegativeRecurrences(additionalRuns))
  }
  return Right(
    schedule((observation) =>
      observation <= additionalRuns ? ScheduleContinue(delay) : ScheduleStop
    )
  )
}

export function whileInput<Input>(
  predicate: (input: Input) => boolean
): Schedule<Input> {
  return schedule((_observation, input) =>
    predicate(input) ? ScheduleContinue(createDuration(0n)) : ScheduleStop
  )
}

export function retry<Environment, Failure, Success>(
  policy: Schedule<Failure>,
  effect: Effect<Environment, Failure, Success>
): Effect<Environment & ClockRequirement, Failure, Success> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    let observation = 0
    while (true) {
      try {
        return await awaitWithCancellation(
          effect(environment, activeContext),
          activeContext
        )
      } catch (error) {
        if (isEffectCancellation(error)) throw error
        if (!(error instanceof TypedFailureSignal)) throw error
        observation += 1
        const decision = policy.decide(observation, error.error as Failure)
        if (decision.tag === "ScheduleStop") throw error
        await sleepWithClock(environment, decision.value, activeContext)
      }
    }
  }
}

export function repeat<Environment, Failure, Success>(
  policy: Schedule<Success>,
  effect: Effect<Environment, Failure, Success>
): Effect<Environment & ClockRequirement, Failure, Success> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    let observation = 0
    while (true) {
      const value = await awaitWithCancellation(
        effect(environment, activeContext),
        activeContext
      )
      observation += 1
      const decision = policy.decide(observation, value)
      if (decision.tag === "ScheduleStop") return value
      await sleepWithClock(environment, decision.value, activeContext)
    }
  }
}

export function timeout<Environment, Failure, Success>(
  duration: Duration,
  effect: Effect<Environment, Failure, Success>
): Effect<Environment & ClockRequirement, Failure, Maybe<Success>> {
  return raceWithTimeout<Environment, Failure, Success, Maybe<Success>>(
    duration,
    effect,
    (value) => Just(value),
    () => Nothing
  )
}

export function timeoutFail<Environment, Failure, Success>(
  error: Failure,
  duration: Duration,
  effect: Effect<Environment, Failure, Success>
): Effect<Environment & ClockRequirement, Failure, Success> {
  return raceWithTimeout(
    duration,
    effect,
    (value) => value,
    () => {
      throw new TypedFailureSignal(error)
    }
  )
}

async function sleepWithClock(
  environment: ClockRequirement,
  duration: Duration,
  context: EffectContext
): Promise<Unit> {
  return awaitWithCancellation(
    environment.clock.sleep(duration, context),
    context
  )
}

function raceWithTimeout<Environment, Failure, Success, Result>(
  duration: Duration,
  effect: Effect<Environment, Failure, Success>,
  completed: (value: Success) => Result,
  expired: () => Result
): Effect<Environment & ClockRequirement, Failure, Result> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    const sourceExecution = createEffectExecution()
    const timerExecution = createEffectExecution()
    const cancelChildren = activeContext.onCancel(async () => {
      await Promise.all([sourceExecution.cancel(), timerExecution.cancel()])
    })
    const source = Promise.resolve()
      .then(() => effect(environment, sourceExecution.context))
      .then(
        (value) => ({ tag: "source" as const, value }),
        (error: unknown) => ({ tag: "source-error" as const, error })
      )
    const timer = Promise.resolve()
      .then(() => environment.clock.sleep(duration, timerExecution.context))
      .then(
        () => ({ tag: "timer" as const }),
        (error: unknown) => ({ tag: "timer-error" as const, error })
      )
    try {
      const outcome = await awaitWithCancellation(
        Promise.race([source, timer]),
        activeContext
      )
      if (outcome.tag === "source") {
        await timerExecution.cancel()
        return completed(outcome.value)
      }
      if (outcome.tag === "source-error") {
        await timerExecution.cancel()
        throw outcome.error
      }
      if (outcome.tag === "timer-error") {
        await sourceExecution.cancel()
        throw outcome.error
      }
      await sourceExecution.cancel()
      return expired()
    } finally {
      cancelChildren()
    }
  }
}

export async function run<Environment, Failure, Success>(
  effect: Effect<Environment, Failure, Success>,
  environment: Environment,
  context?: EffectContext
): Promise<EffectResult<Failure, Success>> {
  const ownedExecution =
    context === undefined ? createEffectExecution() : undefined
  const activeContext = context ?? ownedExecution?.context
  if (activeContext === undefined) {
    throw new TypeError("Effect execution context is unavailable")
  }
  try {
    throwIfCancelled(activeContext)
    return {
      kind: "success",
      value: await awaitWithCancellation(
        effect(environment, activeContext),
        activeContext
      ),
    }
  } catch (error) {
    if (isEffectCancellation(error)) {
      throw error
    }
    if (error instanceof TypedFailureSignal) {
      return { kind: "failure", error: error.error as Failure }
    }
    throw error
  } finally {
    await ownedExecution?.close()
  }
}

async function runCancellationCleanup(
  cleanup: EffectCancellationCleanup
): Promise<void> {
  try {
    await cleanup()
  } catch {
    // A cancelled UI must not be resurrected by cleanup defects.
  }
}

function awaitWithCancellation<Success>(
  value: Promise<Success> | Success,
  context: EffectContext
): Promise<Success> {
  return new Promise((resolve, reject) => {
    let settled = false
    const finish = (settle: () => void): void => {
      if (settled) return
      settled = true
      context.signal.removeEventListener("abort", abort)
      settle()
    }
    const abort = (): void => finish(() => reject(new EffectCancellation()))
    context.signal.addEventListener("abort", abort, { once: true })
    void Promise.resolve(value).then(
      (success) => {
        if (context.signal.aborted) {
          abort()
          return
        }
        finish(() => resolve(success))
      },
      (error: unknown) => finish(() => reject(error))
    )
    // Observe an already-cancelled context only after consuming rejection
    // from the value, which may have cancelled synchronously during creation.
    if (context.signal.aborted) abort()
  })
}

/** Internal checkpoint used by sibling runtime modules. */
export function awaitEffectValue<Success>(
  value: Promise<Success> | Success,
  context: EffectContext
): Promise<Success> {
  return awaitWithCancellation(value, context)
}
