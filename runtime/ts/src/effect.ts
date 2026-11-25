import { defectWithSuppressed } from "./cleanup-defect"
import { createDuration, type Duration } from "./clock-value"
import type { Iterable, RuntimeDictionary } from "./collection"
import { locateRuntimeDefect, type RuntimeSource } from "./runtime-diagnostic"
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

type Keepable = { kept: boolean }

/**
 * Tracks what is registered while streams open. Once an open has finished,
 * what it registered is kept: a partial release leaves it for the cursor's
 * close. An open still in progress belongs to the pull being cancelled, so
 * what it registered, such as a wait it may be blocked on, is not kept.
 */
class OpeningBatches {
  private readonly batches = new Set<Keepable[]>()

  add(entry: Keepable): void {
    for (const batch of this.batches) batch.push(entry)
  }

  start(): (opened: boolean) => void {
    const batch: Keepable[] = []
    this.batches.add(batch)
    return (opened) => {
      if (!this.batches.delete(batch)) return
      if (opened) for (const entry of batch) entry.kept = true
    }
  }
}

type ResourceFinalizerEntry = Keepable & {
  active: boolean
  readonly order: number
  readonly finalizer: EffectResourceFinalizer
}

class ResourceScope {
  private readonly finalizers: ResourceFinalizerEntry[] = []
  private readonly acquisitions = new Set<Promise<void>>()
  private state: "open" | "closing" | "closed" = "open"
  private closing: Promise<void> | undefined
  private registrations = 0
  private readonly openings = new OpeningBatches()
  private partial: Promise<void> | undefined

  register(finalizer: EffectResourceFinalizer): EffectResourceRegistration {
    if (this.state === "closed") {
      return Object.freeze({
        ready: runResourceFinalizer(finalizer),
        unregister: () => undefined,
      })
    }
    this.registrations += 1
    const entry: ResourceFinalizerEntry = {
      active: true,
      order: this.registrations,
      kept: false,
      finalizer,
    }
    this.openings.add(entry)
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
    // A partial release may be running a finalizer; the rest wait for it,
    // so nothing older is released first.
    const partial = this.partial
    this.closing =
      partial === undefined
        ? this.drain()
        : partial.then(
            () => this.drain(),
            () => this.drain()
          )
    return this.closing
  }

  /** Starts tracking a stream's open; see `OpeningBatches`. */
  startOpening(): (opened: boolean) => void {
    return this.openings.start()
  }

  /** How many finalizers have been registered so far. */
  mark(): number {
    return this.registrations
  }

  /**
   * Releases, newest first, the finalizers registered after `mark`. It
   * stops at the first kept one, so nothing older is released before what
   * was acquired after it. The scope stays open, so the rest wait for
   * `close`.
   */
  async releaseAfter(mark: number): Promise<void> {
    if (this.state !== "open") return await this.close()
    const partial = this.releaseNewest(mark)
    this.partial = partial
    try {
      await partial
    } finally {
      if (this.partial === partial) this.partial = undefined
    }
  }

  private async releaseNewest(mark: number): Promise<void> {
    const defects: unknown[] = []
    while (true) {
      if (this.acquisitions.size > 0) {
        await Promise.all([...this.acquisitions])
        continue
      }
      const entry = this.finalizers.at(-1)
      if (entry === undefined) break
      if (!entry.active) {
        this.finalizers.pop()
        continue
      }
      if (entry.kept || entry.order <= mark) break
      this.finalizers.pop()
      entry.active = false
      try {
        await entry.finalizer()
      } catch (error) {
        defects.push(normalizeFinalizerDefect(error))
      }
    }
    if (defects.length > 0) throw defectWithSuppressed(defects)
  }

  async acquire<Resource>(
    action: () => Resource | Promise<Resource>
  ): Promise<Resource> {
    let completed = (): void => undefined
    const pending = new Promise<void>((resolve) => {
      completed = resolve
    })
    // Register before calling the host: it may request cancellation
    // synchronously, or finish after this scope starts closing.
    this.acquisitions.add(pending)
    try {
      return await action()
    } finally {
      this.acquisitions.delete(pending)
      completed()
    }
  }

  private async drain(): Promise<void> {
    const defects: unknown[] = []
    while (this.finalizers.length > 0 || this.acquisitions.size > 0) {
      // A late acquisition must install its finalizer before this scope
      // releases older resources or announces completion.
      if (this.acquisitions.size > 0) {
        await Promise.all([...this.acquisitions])
        continue
      }
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

/** Supervises host acquisition through its atomic resource handoff. */
export function withResourceAcquisition<Resource>(
  context: EffectContext,
  action: () => Resource | Promise<Resource>
): Promise<Resource> {
  throwIfCancelled(context)
  return resourceScopeOf(context).acquire(action)
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
 * How far an execution's cleanups and resources had been registered at one
 * moment; `releaseExecutionAfter` releases what came later.
 */
export type EffectExecutionMark = Readonly<{
  cleanups: number
  resources: number
}>

type ExecutionControl = Readonly<{
  startOpening: () => (opened: boolean) => void
  interrupt: () => void
  mark: () => EffectExecutionMark
  releaseAfter: (mark: EffectExecutionMark) => Promise<void>
}>

const executionControls = new WeakMap<EffectExecution, ExecutionControl>()
// Contexts derived from an execution share its signal, so the signal finds
// the execution that owns a cleanup registered through any of them.
const signalControls = new WeakMap<AbortSignal, ExecutionControl>()

/**
 * Creates an isolated cancellation scope for exactly one root Effect run.
 * Cancellation has two steps. Aborting the signal stops the work and reaches
 * every child execution at once; releasing runs the `onCancel` cleanups and
 * closes the resource scope. `cancel` does both, and a child is released when
 * its parent is. Cleanup registered while cancellation is settling joins the
 * same drain, including cleanup registered by another cleanup. The drain
 * closes only after one quiet event-loop turn with no tracked work.
 * Cleanup failures are intentionally absorbed: cancellation must not surface
 * as a stale host exception after the owning UI has moved on.
 */
export function createEffectExecution(
  parentContext?: EffectContext
): EffectExecution {
  const controller = new AbortController()
  const rootScope = new ResourceScope()
  // Each cleanup keeps its registration order, so a partial release can
  // tell which cleanups came after a mark, and whether it must keep them.
  const cleanups = new Map<
    EffectCancellationCleanup,
    Keepable & { readonly order: number }
  >()
  const started = new Map<EffectCancellationCleanup, Promise<void>>()
  const pending = new Set<Promise<void>>()
  let cancellation: Promise<void> | undefined
  let cleanupEpoch = 0
  let cleanupRegistrations = 0
  const openings = new OpeningBatches()
  // Cleanups started while a partial release runs; it waits for them too.
  let releasing: Set<Promise<void>> | undefined
  const startCleanup = (cleanup: EffectCancellationCleanup): Promise<void> => {
    const running = started.get(cleanup)
    if (running !== undefined) return running
    cleanupEpoch += 1
    cleanups.delete(cleanup)
    // Record the start before calling the cleanup: it may register itself
    // again synchronously, and that must not run it twice.
    let finish = (): void => undefined
    const task = new Promise<void>((resolve) => {
      finish = resolve
    })
    started.set(cleanup, task)
    pending.add(task)
    void task.then(() => pending.delete(task))
    void runCancellationCleanup(cleanup).then(finish)
    return task
  }
  const drainCleanups = async (): Promise<void> => {
    while (true) {
      for (const cleanup of [...cleanups.keys()]) startCleanup(cleanup)
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
  const interrupt = (): void => {
    if (!controller.signal.aborted) controller.abort()
  }
  const context: EffectContext = Object.freeze({
    signal: controller.signal,
    get cancelled() {
      return controller.signal.aborted
    },
    onCancel(cleanup) {
      if (started.has(cleanup)) return () => undefined
      // An aborted execution that is not releasing yet keeps the cleanup for
      // its release, so a later explicit close still runs first.
      if (cancellation !== undefined) {
        startCleanup(cleanup)
        return () => undefined
      }
      // A cleanup registered by a cleanup that a partial release runs may be
      // awaited by it, so it starts at once.
      if (releasing !== undefined) {
        releasing.add(startCleanup(cleanup))
        return () => undefined
      }
      // Registering the same cleanup again keeps its first place.
      if (!cleanups.has(cleanup)) {
        cleanupRegistrations += 1
        const entry = { order: cleanupRegistrations, kept: false }
        openings.add(entry)
        cleanups.set(cleanup, entry)
      }
      return () => {
        cleanups.delete(cleanup)
      }
    },
  })
  resourceScopes.set(context, rootScope)
  let detachParent = (): void => undefined
  const execution: EffectExecution = Object.freeze({
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
      interrupt()
      for (const cleanup of [...cleanups.keys()]) startCleanup(cleanup)
      void (async () => {
        await drainCleanups()
        await rootScope.close()
      })().then(resolveCancellation, rejectCancellation)
      return cancellation
    },
    close() {
      // Once aborted, the cleanups registered with `onCancel` still have to
      // run, so closing releases like a cancellation.
      if (controller.signal.aborted) return execution.cancel()
      detachParent()
      return rootScope.close()
    },
  })
  const control: ExecutionControl = Object.freeze({
    startOpening: () => openings.start(),
    interrupt,
    mark: () =>
      Object.freeze({
        cleanups: cleanupRegistrations,
        resources: rootScope.mark(),
      }),
    async releaseAfter(mark: EffectExecutionMark) {
      if (cancellation !== undefined) return await cancellation
      const owner = releasing === undefined
      const tasks = releasing ?? new Set<Promise<void>>()
      releasing = tasks
      const finished = new Set<Promise<void>>()
      const settle = async (): Promise<void> => {
        while (true) {
          const running = [...tasks].filter((task) => !finished.has(task))
          if (running.length === 0) return
          await Promise.all(running)
          for (const task of running) finished.add(task)
        }
      }
      try {
        // As with resources, nothing at or before the newest kept cleanup
        // runs early.
        let floor = mark.cleanups
        for (const entry of cleanups.values()) {
          if (entry.kept && entry.order > floor) floor = entry.order
        }
        for (const [cleanup, entry] of [...cleanups]) {
          if (entry.order > floor) tasks.add(startCleanup(cleanup))
        }
        await settle()
        // An acquisition the scope waits for may still register a cleanup
        // and wait for it, so new cleanups keep starting at once until the
        // resources are released.
        await rootScope.releaseAfter(mark.resources)
        await settle()
      } finally {
        if (owner) releasing = undefined
      }
    },
  })
  executionControls.set(execution, control)
  signalControls.set(controller.signal, control)
  if (parentContext !== undefined) {
    // The parent's abort reaches this execution at once; its release waits
    // until the parent releases.
    const signal = parentContext.signal
    signal.addEventListener("abort", interrupt, { once: true })
    const detachCleanup = parentContext.onCancel(() => execution.cancel())
    detachParent = () => {
      signal.removeEventListener("abort", interrupt)
      detachCleanup()
    }
    if (signal.aborted) interrupt()
  }
  return execution
}

function executionControl(execution: EffectExecution): ExecutionControl {
  const control = executionControls.get(execution)
  if (control === undefined) {
    throw new TypeError("unknown Effect execution")
  }
  return control
}

/**
 * Internal: aborts an execution and its children without releasing anything.
 * Work waiting on the signal stops; cleanups and resources wait for a release.
 */
export function interruptExecution(execution: EffectExecution): void {
  executionControl(execution).interrupt()
}

/**
 * Internal: runs a stream's `open`. Once it has opened, a partial release
 * keeps what it registered, because the cursor's close may still use it,
 * even when the stream opened during a pull, as the inner stream of
 * `flatMap` does.
 */
export async function openingStream<Result>(
  context: EffectContext,
  open: () => Result | Promise<Result>
): Promise<Result> {
  const finishCleanups = signalControls.get(context.signal)?.startOpening()
  const finishResources = resourceScopes.get(context)?.startOpening()
  let opened = false
  try {
    const result = await open()
    opened = true
    return result
  } finally {
    finishResources?.(opened)
    finishCleanups?.(opened)
  }
}

/** Internal: marks what an execution has registered so far. */
export function markExecution(execution: EffectExecution): EffectExecutionMark {
  return executionControl(execution).mark()
}

/**
 * Internal: runs the `onCancel` cleanups and releases the resources that an
 * execution registered after `mark`, newest first. The older ones stay for
 * its `cancel` or `close`.
 */
export function releaseExecutionAfter(
  execution: EffectExecution,
  mark: EffectExecutionMark
): Promise<void> {
  return executionControl(execution).releaseAfter(mark)
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
  const deferred: Effect<Environment, Failure, Success> = (
    environment,
    context
  ) =>
    interpretBinds(
      deferred as ErasedEffect,
      environment,
      context ?? createEffectExecution().context
    ) as Promise<Success>
  Object.defineProperty(deferred, deferPlan, {
    value: thunk as (unit: Unit) => ErasedEffect,
  })
  return deferred
}

// Bind plans are internal and owned by their callable Effect value.
// Interpret binds and deferred construction in one loop. Neither association
// nor engine-specific tail calls may determine stack safety.
type ErasedEffect = Effect<unknown, unknown, unknown>
type BindPlan = {
  readonly source: ErasedEffect
  readonly next: (value: unknown) => ErasedEffect
}
const bindPlan = Symbol("seseragi.effect-bind")
const deferPlan = Symbol("seseragi.effect-defer")
const sourcePlan = Symbol("seseragi.effect-source")
type PlannedEffect = ErasedEffect & {
  readonly [bindPlan]?: BindPlan
  readonly [deferPlan]?: (unit: Unit) => ErasedEffect
  readonly [sourcePlan]?: Readonly<{
    readonly effect: ErasedEffect
    readonly source: RuntimeSource
  }>
}
export function atSource<Value>(
  source: RuntimeSource,
  action: () => Value,
  effect: boolean
): Value {
  try {
    const value = action()
    if (!effect || typeof value !== "function") return value
    if ((value as PlannedEffect)[sourcePlan] !== undefined) return value
    const located: ErasedEffect = (environment, context) =>
      interpretBinds(
        located,
        environment,
        context ?? createEffectExecution().context
      )
    Object.defineProperty(located, sourcePlan, {
      value: { effect: value as ErasedEffect, source },
    })
    return located as Value
  } catch (error) {
    throw locateEffectDefect(error, source)
  }
}

function locateEffectDefect(
  error: unknown,
  source: RuntimeSource | undefined
): unknown {
  return error instanceof TypedFailureSignal || isEffectCancellation(error)
    ? error
    : locateRuntimeDefect(error, source)
}

async function interpretBinds(
  initial: ErasedEffect,
  environment: unknown,
  context: EffectContext
): Promise<unknown> {
  const continuations: BindPlan["next"][] = []
  const sources: (RuntimeSource | undefined)[] = []
  let current = initial
  let source: RuntimeSource | undefined
  try {
    while (true) {
      while (true) {
        const located = (current as PlannedEffect)[sourcePlan]
        if (located !== undefined) {
          source = located.source
          current = located.effect
          continue
        }
        const plan = (current as PlannedEffect)[bindPlan]
        if (plan !== undefined) {
          continuations.push(plan.next)
          sources.push(source)
          current = plan.source
          continue
        }
        const thunk = (current as PlannedEffect)[deferPlan]
        if (thunk !== undefined) {
          current = thunk(unit)
          continue
        }
        break
      }
      const value = await awaitWithCancellation(
        current(environment, context),
        context
      )
      const next = continuations.pop()
      if (next === undefined) return value
      source = sources.pop()
      current = next(value)
    }
  } catch (error) {
    throw locateEffectDefect(error, source)
  }
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
  const bound: Effect<
    Environment & NextEnvironment,
    Failure | NextFailure,
    NextSuccess
  > = (environment, context) =>
    interpretBinds(
      bound as ErasedEffect,
      environment,
      context ?? createEffectExecution().context
    ) as Promise<NextSuccess>
  Object.defineProperty(bound, bindPlan, {
    value: {
      source: effect as ErasedEffect,
      next: next as (value: unknown) => ErasedEffect,
    } satisfies BindPlan,
  })
  return bound
}

export const effectFunctor = /* @__PURE__ */ Object.freeze({
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

export const effectApplicative = /* @__PURE__ */ Object.freeze({
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

export const effectMonad = /* @__PURE__ */ Object.freeze({
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

export function die(message: string): Effect<unknown, never, never> {
  return () => {
    throw new Error(message)
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
    return withResourceAcquisition(activeContext, async () => {
      const resource = await acquire(environment, activeContext)
      const scope = resourceScopeOf(activeContext)
      const finalizerContext = maskedContext(scope)
      const registration = scope.register(() =>
        release(resource)(environment, finalizerContext)
      )
      await registration.ready
      throwIfCancelled(activeContext)
      return resource
    })
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
      // A parent cancellation must observe this same lexical close, including
      // its failure. Unregister only after success; otherwise an early abort
      // can consume the source rejection before the parent drains cleanup.
      await innerScope.close()
      registration.unregister()
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

export const FiberCancelled: FiberCancelled = /* @__PURE__ */ Object.freeze({
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

// Waiters for the end of the current scheduler turn. When a `yieldNow` timer
// fires while some wait, they are released first and the yielded Fiber
// resumes in a later macrotask, as native code resumes a yielded Fiber only
// after the others have reached a wait.
const turnEndWaiters = new Set<() => void>()

export function awaitTurnEnd(): Promise<void> {
  return new Promise<void>((resolve) => {
    const release = (): void => {
      if (turnEndWaiters.delete(release)) resolve()
    }
    turnEndWaiters.add(release)
    setTimeout(release, 0)
  })
}

function releaseTurnEndWaiters(): boolean {
  if (turnEndWaiters.size === 0) return false
  for (const release of [...turnEndWaiters]) release()
  return true
}

export function yieldNow(_unit?: Unit): Effect<unknown, never, Unit> {
  return async (_environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    await awaitWithCancellation(
      new Promise<void>((resolve) => {
        const resume = (): void => {
          if (releaseTurnEndWaiters()) setTimeout(resume, 0)
          else resolve()
        }
        setTimeout(resume, 0)
      }),
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
export const Continue: LoopControl = /* @__PURE__ */ Object.freeze({
  tag: "Continue",
})
export const Break: LoopControl = /* @__PURE__ */ Object.freeze({
  tag: "Break",
})

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

export const ScheduleStop: ScheduleStop = /* @__PURE__ */ Object.freeze({
  tag: "ScheduleStop",
})

export function ScheduleContinue(value: Duration): ScheduleContinue {
  return Object.freeze({ tag: "ScheduleContinue", value })
}

export function NegativeRecurrences(value: number): ScheduleError {
  return Object.freeze({ tag: "NegativeRecurrences", value })
}

function scheduleOf<Input>(
  decide: (observation: number, input: Input) => ScheduleDecision
): Schedule<Input> {
  return Object.freeze({ decide }) as Schedule<Input>
}

/** `effects.schedule`. A Seseragi decide takes its arguments one at a time. */
export function schedule<Input>(
  decide: (observation: number) => (input: Input) => ScheduleDecision
): Schedule<Input> {
  return scheduleOf((observation, input: Input) => decide(observation)(input))
}

export function recurs<Input>(
  additionalRuns: number
): Either<ScheduleError, Schedule<Input>> {
  if (!Number.isSafeInteger(additionalRuns) || additionalRuns < 0) {
    return Left(NegativeRecurrences(additionalRuns))
  }
  return Right(
    scheduleOf((observation) =>
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
    scheduleOf((observation) =>
      observation <= additionalRuns ? ScheduleContinue(delay) : ScheduleStop
    )
  )
}

export function whileInput<Input>(
  predicate: (input: Input) => boolean
): Schedule<Input> {
  return scheduleOf((_observation, input: Input) =>
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
