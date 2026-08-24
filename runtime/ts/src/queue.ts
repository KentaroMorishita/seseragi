import {
  createEffectExecution,
  type Effect,
  EffectCancellation,
  fail,
  fromEither,
  type Unit,
} from "./effect"
import { type Either, Just, Left, type Maybe, Nothing, Right } from "./sum"

export type QueueCreateError = Readonly<{
  tag: "NonPositiveCapacity"
  value: number
}>

export type QueueClosed = Readonly<{ tag: "QueueClosed" }>

export function NonPositiveCapacity(value: number): QueueCreateError {
  return Object.freeze({ tag: "NonPositiveCapacity", value })
}

export const QueueClosed: QueueClosed = Object.freeze({ tag: "QueueClosed" })

const queueBrand: unique symbol = Symbol("seseragi.queue")

export type Queue<Value> = Readonly<{
  readonly [queueBrand]: Value
}>

type TakeWaiter<Value> = {
  active: boolean
  detach: () => void
  readonly settle: (result: Either<QueueClosed, Value>) => void
}

type OfferWaiter<Value> = {
  active: boolean
  detach: () => void
  readonly value: Value
  readonly settle: (result: Either<QueueClosed, Unit>) => void
}

type QueueState<Value> = {
  readonly capacity: number
  readonly buffer: Value[]
  readonly takers: TakeWaiter<Value>[]
  readonly offerers: OfferWaiter<Value>[]
  closed: boolean
}

const queueStates = new WeakMap<object, QueueState<unknown>>()

function stateOf<Value>(queue: Queue<Value>): QueueState<Value> {
  const state = queueStates.get(queue)
  if (state === undefined) {
    throw new TypeError("Queue value does not use the runtime brand")
  }
  return state as QueueState<Value>
}

function createQueue<Value>(capacity: number): Queue<Value> {
  const queue = Object.freeze({}) as Queue<Value>
  queueStates.set(queue, {
    capacity,
    buffer: [],
    takers: [],
    offerers: [],
    closed: false,
  })
  return queue
}

function firstActive<Waiter extends { active: boolean }>(
  waiters: Waiter[]
): Waiter | undefined {
  while (waiters.length > 0 && waiters[0]?.active === false) waiters.shift()
  return waiters.shift()
}

function settleTake<Value>(
  waiter: TakeWaiter<Value>,
  result: Either<QueueClosed, Value>
): void {
  if (!waiter.active) return
  waiter.active = false
  waiter.detach()
  waiter.settle(result)
}

function settleOffer<Value>(
  waiter: OfferWaiter<Value>,
  result: Either<QueueClosed, Unit>
): void {
  if (!waiter.active) return
  waiter.active = false
  waiter.detach()
  waiter.settle(result)
}

function drain<Value>(state: QueueState<Value>): void {
  while (state.buffer.length > 0) {
    const taker = firstActive(state.takers)
    if (taker === undefined) break
    settleTake(taker, Right(state.buffer.shift() as Value))
  }

  if (state.closed) {
    let offerer = firstActive(state.offerers)
    while (offerer !== undefined) {
      settleOffer(offerer, Left(QueueClosed))
      offerer = firstActive(state.offerers)
    }
    if (state.buffer.length === 0) {
      let taker = firstActive(state.takers)
      while (taker !== undefined) {
        settleTake(taker, Left(QueueClosed))
        taker = firstActive(state.takers)
      }
    }
    return
  }

  while (true) {
    const offerer = firstActive(state.offerers)
    if (offerer === undefined) return
    const taker = firstActive(state.takers)
    if (taker !== undefined && state.buffer.length === 0) {
      settleTake(taker, Right(offerer.value))
      settleOffer(offerer, Right(undefined))
      continue
    }
    if (state.buffer.length < state.capacity) {
      state.buffer.push(offerer.value)
      settleOffer(offerer, Right(undefined))
      continue
    }
    state.offerers.unshift(offerer)
    return
  }
}

export function bounded<Value>(
  capacity: number
): Effect<unknown, QueueCreateError, Queue<Value>> {
  return (environment, context) =>
    Number.isSafeInteger(capacity) && capacity > 0
      ? createQueue(capacity)
      : fail(NonPositiveCapacity(capacity))(environment, context)
}

export function unbounded<Value>(): Effect<unknown, never, Queue<Value>> {
  return () => createQueue(Number.POSITIVE_INFINITY)
}

export function offer<Value>(
  value: Value,
  queue: Queue<Value>
): Effect<unknown, QueueClosed, Unit> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    const state = stateOf<Value>(queue)
    if (state.closed) return fail(QueueClosed)(environment, activeContext)
    const taker = firstActive(state.takers)
    if (taker !== undefined && state.buffer.length === 0) {
      settleTake(taker, Right(value))
      return undefined
    }
    if (state.buffer.length < state.capacity) {
      state.buffer.push(value)
      return undefined
    }
    const result = await new Promise<Either<QueueClosed, Unit>>(
      (resolve, reject) => {
        const waiter: OfferWaiter<Value> = {
          active: true,
          detach: () => undefined,
          value,
          settle: resolve,
        }
        waiter.detach = activeContext.onCancel(() => {
          if (!waiter.active) return
          waiter.active = false
          const index = state.offerers.indexOf(waiter)
          if (index >= 0) state.offerers.splice(index, 1)
          drain(state)
          reject(new EffectCancellation())
        })
        state.offerers.push(waiter)
      }
    )
    return fromEither(result)(environment, activeContext)
  }
}

export function take<Value>(
  queue: Queue<Value>
): Effect<unknown, QueueClosed, Value> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    const state = stateOf<Value>(queue)
    if (state.buffer.length > 0) {
      const value = state.buffer.shift() as Value
      drain(state)
      return value
    }
    if (state.closed) return fail(QueueClosed)(environment, activeContext)
    const offerer = firstActive(state.offerers)
    if (offerer !== undefined) {
      settleOffer(offerer, Right(undefined))
      return offerer.value
    }
    const result = await new Promise<Either<QueueClosed, Value>>(
      (resolve, reject) => {
        const waiter: TakeWaiter<Value> = {
          active: true,
          detach: () => undefined,
          settle: resolve,
        }
        waiter.detach = activeContext.onCancel(() => {
          if (!waiter.active) return
          waiter.active = false
          const index = state.takers.indexOf(waiter)
          if (index >= 0) state.takers.splice(index, 1)
          drain(state)
          reject(new EffectCancellation())
        })
        state.takers.push(waiter)
      }
    )
    return (fromEither(result) as Effect<unknown, QueueClosed, Value>)(
      environment,
      activeContext
    )
  }
}

export function tryOffer<Value>(
  value: Value,
  queue: Queue<Value>
): Effect<unknown, never, Either<QueueClosed, boolean>> {
  return () => {
    const state = stateOf<Value>(queue)
    if (state.closed) return Left(QueueClosed)
    const taker = firstActive(state.takers)
    if (taker !== undefined && state.buffer.length === 0) {
      settleTake(taker, Right(value))
      return Right(true)
    }
    if (state.buffer.length >= state.capacity) return Right(false)
    state.buffer.push(value)
    return Right(true)
  }
}

export function tryTake<Value>(
  queue: Queue<Value>
): Effect<unknown, never, Either<QueueClosed, Maybe<Value>>> {
  return () => {
    const state = stateOf<Value>(queue)
    if (state.buffer.length > 0) {
      const value = state.buffer.shift() as Value
      drain(state)
      return Right(Just(value))
    }
    if (state.closed) return Left(QueueClosed)
    const offerer = firstActive(state.offerers)
    if (offerer !== undefined) {
      settleOffer(offerer, Right(undefined))
      return Right(Just(offerer.value))
    }
    return Right(Nothing)
  }
}

export function size<Value>(
  queue: Queue<Value>
): Effect<unknown, never, number> {
  return () => stateOf<Value>(queue).buffer.length
}

export function close<Value>(
  queue: Queue<Value>
): Effect<unknown, never, Unit> {
  return () => {
    const state = stateOf<Value>(queue)
    if (!state.closed) {
      state.closed = true
      drain(state)
    }
    return undefined
  }
}
