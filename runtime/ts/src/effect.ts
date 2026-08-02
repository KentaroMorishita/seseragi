import type { Either, Left, Right } from "./sum"

export type Unit = undefined

export type Effect<Environment, Failure, Success> = ((
  environment: Environment,
  context?: EffectContext
) => Promise<Success> | Success) & {
  readonly __failure?: Failure
}

export type EffectCancellationCleanup = () => void | Promise<void>

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
}>

const effectContext = Symbol("seseragi.effect-context")

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
 * Cleanup failures are intentionally absorbed: cancellation must not surface
 * as a stale host exception after the owning UI has moved on.
 */
export function createEffectExecution(): EffectExecution {
  const controller = new AbortController()
  const cleanups = new Set<EffectCancellationCleanup>()
  let cancellation: Promise<void> | undefined
  const context: EffectContext = Object.freeze({
    signal: controller.signal,
    get cancelled() {
      return controller.signal.aborted
    },
    onCancel(cleanup) {
      if (controller.signal.aborted) {
        void runCancellationCleanup(cleanup)
        return () => undefined
      }
      cleanups.add(cleanup)
      return () => {
        cleanups.delete(cleanup)
      }
    },
  })
  return Object.freeze({
    context,
    cancel() {
      if (cancellation !== undefined) return cancellation
      controller.abort()
      const pending = [...cleanups]
      cleanups.clear()
      cancellation = Promise.allSettled(
        pending.map((cleanup) => runCancellationCleanup(cleanup))
      ).then(() => undefined)
      return cancellation
    },
  })
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

export const unit: Unit = undefined

export function succeed<Success>(
  value: Success
): Effect<unknown, never, Success> {
  return () => value
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

export async function run<Environment, Failure, Success>(
  effect: Effect<Environment, Failure, Success>,
  environment: Environment,
  context: EffectContext = createEffectExecution().context
): Promise<EffectResult<Failure, Success>> {
  try {
    return {
      kind: "success",
      value: await awaitWithCancellation(effect(environment, context), context),
    }
  } catch (error) {
    if (isEffectCancellation(error)) {
      throw error
    }
    if (error instanceof TypedFailureSignal) {
      return { kind: "failure", error: error.error as Failure }
    }
    throw error
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
  throwIfCancelled(context)
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
  })
}
