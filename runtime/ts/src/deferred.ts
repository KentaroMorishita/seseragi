import {
  createEffectExecution,
  type Effect,
  EffectCancellation,
  fromEither,
} from "./effect"
import { type Either, type Maybe, Nothing, Right, Just, Left } from "./sum"

const deferredBrand: unique symbol = Symbol("seseragi.deferred")

export type Deferred<Failure, Success> = Readonly<{
  readonly [deferredBrand]: (failure: Failure) => Success
}>

type DeferredWaiter<Failure, Success> = {
  active: boolean
  detach: () => void
  readonly settle: (result: Either<Failure, Success>) => void
}

type DeferredState<Failure, Success> = {
  result: Either<Failure, Success> | undefined
  readonly waiters: DeferredWaiter<Failure, Success>[]
}

const deferredStates = new WeakMap<object, DeferredState<unknown, unknown>>()

function stateOf<Failure, Success>(
  deferred: Deferred<Failure, Success>
): DeferredState<Failure, Success> {
  const state = deferredStates.get(deferred)
  if (state === undefined) {
    throw new TypeError("Deferred value does not use the runtime brand")
  }
  return state as DeferredState<Failure, Success>
}

export function make<Failure, Success>(): Effect<
  unknown,
  never,
  Deferred<Failure, Success>
> {
  return () => {
    const deferred = Object.freeze({}) as Deferred<Failure, Success>
    deferredStates.set(deferred, { result: undefined, waiters: [] })
    return deferred
  }
}

export function awaitDeferred<Failure, Success>(
  deferred: Deferred<Failure, Success>
): Effect<unknown, Failure, Success> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    const state = stateOf(deferred)
    const completed = state.result
    if (completed !== undefined) {
      return (fromEither(completed) as Effect<unknown, Failure, Success>)(
        environment,
        activeContext
      )
    }
    const result = await new Promise<Either<Failure, Success>>(
      (resolve, reject) => {
        const waiter: DeferredWaiter<Failure, Success> = {
          active: true,
          detach: () => undefined,
          settle(value) {
            if (!waiter.active) return
            waiter.active = false
            waiter.detach()
            resolve(value)
          },
        }
        waiter.detach = activeContext.onCancel(() => {
          if (!waiter.active) return
          waiter.active = false
          const index = state.waiters.indexOf(waiter)
          if (index >= 0) state.waiters.splice(index, 1)
          reject(new EffectCancellation())
        })
        state.waiters.push(waiter)
      }
    )
    return (fromEither(result) as Effect<unknown, Failure, Success>)(
      environment,
      activeContext
    )
  }
}

export function poll<Failure, Success>(
  deferred: Deferred<Failure, Success>
): Effect<unknown, never, Maybe<Either<Failure, Success>>> {
  return () => {
    const result = stateOf<Failure, Success>(deferred).result
    return result === undefined ? Nothing : Just(result)
  }
}

export function complete<Failure, Success>(
  result: Either<Failure, Success>,
  deferred: Deferred<Failure, Success>
): Effect<unknown, never, boolean> {
  return () => completeNow(result, stateOf<Failure, Success>(deferred))
}

export function succeed<Failure, Success>(
  value: Success,
  deferred: Deferred<Failure, Success>
): Effect<unknown, never, boolean> {
  return () => completeNow(Right(value), stateOf<Failure, Success>(deferred))
}

export function fail<Failure, Success>(
  error: Failure,
  deferred: Deferred<Failure, Success>
): Effect<unknown, never, boolean> {
  return () => completeNow(Left(error), stateOf<Failure, Success>(deferred))
}

function completeNow<Failure, Success>(
  result: Either<Failure, Success>,
  state: DeferredState<Failure, Success>
): boolean {
  if (state.result !== undefined) return false
  state.result = result
  const waiters = state.waiters.splice(0)
  for (const waiter of waiters) waiter.settle(result)
  return true
}
