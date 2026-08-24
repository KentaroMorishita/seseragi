import {
  awaitEffectValue,
  createEffectExecution,
  type Effect,
  EffectCancellation,
  fail,
  type Unit,
} from "./effect"

export type SemaphoreCreateError = Readonly<{
  tag: "NonPositivePermits"
  value: number
}>

export function NonPositivePermits(value: number): SemaphoreCreateError {
  return Object.freeze({ tag: "NonPositivePermits", value })
}

const semaphoreBrand: unique symbol = Symbol("seseragi.semaphore")
const permitBrand: unique symbol = Symbol("seseragi.permit")

export type Semaphore = Readonly<{ readonly [semaphoreBrand]: true }>
export type Permit = Readonly<{ readonly [permitBrand]: true }>

type AcquireWaiter = {
  active: boolean
  detach: () => void
  readonly settle: (permit: Permit) => void
}

type SemaphoreState = {
  available: number
  readonly waiters: AcquireWaiter[]
}

type PermitState = {
  readonly semaphore: SemaphoreState
  released: boolean
}

const semaphoreStates = new WeakMap<object, SemaphoreState>()
const permitStates = new WeakMap<object, PermitState>()

function semaphoreState(semaphore: Semaphore): SemaphoreState {
  const state = semaphoreStates.get(semaphore)
  if (state === undefined) {
    throw new TypeError("Semaphore value does not use the runtime brand")
  }
  return state
}

function permitState(permit: Permit): PermitState {
  const state = permitStates.get(permit)
  if (state === undefined) {
    throw new TypeError("Permit value does not use the runtime brand")
  }
  return state
}

function newPermit(semaphore: SemaphoreState): Permit {
  const permit = Object.freeze({}) as Permit
  permitStates.set(permit, { semaphore, released: false })
  return permit
}

function firstWaiter(state: SemaphoreState): AcquireWaiter | undefined {
  while (state.waiters.length > 0 && state.waiters[0]?.active === false) {
    state.waiters.shift()
  }
  return state.waiters.shift()
}

function releaseNow(permit: Permit): void {
  const owned = permitState(permit)
  if (owned.released) return
  owned.released = true
  const waiter = firstWaiter(owned.semaphore)
  if (waiter === undefined) {
    owned.semaphore.available += 1
    return
  }
  waiter.active = false
  waiter.detach()
  waiter.settle(newPermit(owned.semaphore))
}

export function make(
  permits: number
): Effect<unknown, SemaphoreCreateError, Semaphore> {
  return (environment, context) => {
    if (!Number.isSafeInteger(permits) || permits <= 0) {
      return fail(NonPositivePermits(permits))(environment, context)
    }
    const semaphore = Object.freeze({}) as Semaphore
    semaphoreStates.set(semaphore, { available: permits, waiters: [] })
    return semaphore
  }
}

export function acquire(semaphore: Semaphore): Effect<unknown, never, Permit> {
  return (_environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    const state = semaphoreState(semaphore)
    if (state.available > 0 && state.waiters.length === 0) {
      state.available -= 1
      return newPermit(state)
    }
    return new Promise<Permit>((resolve, reject) => {
      const waiter: AcquireWaiter = {
        active: true,
        detach: () => undefined,
        settle: resolve,
      }
      waiter.detach = activeContext.onCancel(() => {
        if (!waiter.active) return
        waiter.active = false
        const index = state.waiters.indexOf(waiter)
        if (index >= 0) state.waiters.splice(index, 1)
        reject(new EffectCancellation())
      })
      state.waiters.push(waiter)
    })
  }
}

export function release(permit: Permit): Effect<unknown, never, Unit> {
  return () => {
    releaseNow(permit)
    return undefined
  }
}

export function withPermit<Environment, Failure, Success>(
  semaphore: Semaphore,
  effect: Effect<Environment, Failure, Success>
): Effect<Environment, Failure, Success> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    const permit = await acquire(semaphore)(environment, activeContext)
    try {
      return await awaitEffectValue(
        effect(environment, activeContext),
        activeContext
      )
    } finally {
      releaseNow(permit)
    }
  }
}

export function available(
  semaphore: Semaphore
): Effect<unknown, never, number> {
  return () => semaphoreState(semaphore).available
}
