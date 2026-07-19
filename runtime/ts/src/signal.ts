import { type Effect, type Unit, unit } from "./effect"

const signalBrand = Symbol("seseragi.signal")
const mutableSignalBrand = Symbol("seseragi.mutable-signal")
const signalChangeBrand = Symbol("seseragi.signal-change")
const signalStateBrand = Symbol("seseragi.signal-state")
const subscriptionBrand = Symbol("seseragi.subscription")
const cancelSubscriptionBrand = Symbol("seseragi.cancel-subscription")

type SignalState = {
  readonly current: () => unknown
  readonly dependencies: readonly SignalState[]
  readonly depth: number
  readonly subscribers: Set<Subscriber>
}

type Subscriber = {
  active: boolean
  readonly environment: unknown
  readonly observer: (value: unknown) => Effect<unknown, never, Unit>
  readonly source: SignalState
}

export type Signal<Value> = {
  readonly [signalBrand]: true
  readonly [signalStateBrand]: SignalState
  readonly current: () => Value
}

export type MutableSignal<Value> = Signal<Value> & {
  readonly [mutableSignalBrand]: true
  readonly commit: (value: Value) => void
}

export type SignalChange = {
  readonly [signalChangeBrand]: true
  readonly stage: (values: StagedValues) => void
}

export type Subscription = {
  readonly [subscriptionBrand]: true
  readonly [cancelSubscriptionBrand]: () => void
}

type UnknownMutableSignal = MutableSignal<unknown>
type StagedValues = Map<UnknownMutableSignal, unknown>
type QueuedTransaction = {
  readonly changes: readonly SignalChange[]
}

let signalEpoch = 0
let publicationActive = false
const subscribedSignals = new Set<SignalState>()
const queuedTransactions: QueuedTransaction[] = []

function mutable<Value>(initial: Value): MutableSignal<Value> {
  let current = initial
  const subscribers = new Set<Subscriber>()
  const state: SignalState = {
    current: () => current,
    dependencies: [],
    depth: 0,
    subscribers,
  }
  return Object.freeze({
    [signalBrand]: true as const,
    [mutableSignalBrand]: true as const,
    [signalStateBrand]: state,
    current: () => current,
    commit: (value: Value) => {
      current = value
    },
  })
}

function derived<Value>(
  evaluate: () => Value,
  dependencies: readonly Signal<unknown>[]
): Signal<Value> {
  let cachedEpoch = -1
  let cached: Value
  const current = () => {
    if (cachedEpoch !== signalEpoch) {
      cached = evaluate()
      cachedEpoch = signalEpoch
    }
    return cached
  }
  const dependencyStates = dependencies.map(signalState)
  const state: SignalState = {
    current,
    dependencies: dependencyStates,
    depth:
      dependencyStates.length === 0
        ? 0
        : Math.max(...dependencyStates.map((dependency) => dependency.depth)) +
          1,
    subscribers: new Set(),
  }
  return Object.freeze({
    [signalBrand]: true as const,
    [signalStateBrand]: state,
    current,
  })
}

function signalState(source: Signal<unknown>): SignalState {
  return source[signalStateBrand]
}

function change(stage: (values: StagedValues) => void): SignalChange {
  return Object.freeze({
    [signalChangeBrand]: true as const,
    stage,
  })
}

function stagedValue<Value>(
  values: StagedValues,
  target: MutableSignal<Value>
): Value {
  if (values.has(target as UnknownMutableSignal)) {
    return values.get(target as UnknownMutableSignal) as Value
  }
  return target.current()
}

export function make<Value>(
  initial: Value
): Effect<unknown, never, MutableSignal<Value>> {
  return () => mutable(initial)
}

export function read<Value>(
  source: Signal<Value>
): Effect<unknown, never, Value> {
  return () => source.current()
}

export function set<Value>(
  value: Value
): (target: MutableSignal<Value>) => Effect<unknown, never, Unit>
export function set<Value>(
  value: Value,
  target: MutableSignal<Value>
): Effect<unknown, never, Unit>
export function set<Value>(
  value: Value,
  target?: MutableSignal<Value>
):
  | Effect<unknown, never, Unit>
  | ((target: MutableSignal<Value>) => Effect<unknown, never, Unit>) {
  if (target === undefined) {
    return (target: MutableSignal<Value>) => set(value, target)
  }
  return transaction([planSet(value, target)])
}

export function update<Value>(
  mapper: (value: Value) => Value
): (target: MutableSignal<Value>) => Effect<unknown, never, Unit>
export function update<Value>(
  mapper: (value: Value) => Value,
  target: MutableSignal<Value>
): Effect<unknown, never, Unit>
export function update<Value>(
  mapper: (value: Value) => Value,
  target?: MutableSignal<Value>
):
  | Effect<unknown, never, Unit>
  | ((target: MutableSignal<Value>) => Effect<unknown, never, Unit>) {
  if (target === undefined) {
    return (target: MutableSignal<Value>) => update(mapper, target)
  }
  return transaction([planUpdate(mapper, target)])
}

export function planSet<Value>(
  value: Value
): (target: MutableSignal<Value>) => SignalChange
export function planSet<Value>(
  value: Value,
  target: MutableSignal<Value>
): SignalChange
export function planSet<Value>(
  value: Value,
  target?: MutableSignal<Value>
): SignalChange | ((target: MutableSignal<Value>) => SignalChange) {
  if (target === undefined) {
    return (target) => planSet(value, target)
  }
  return change((values) => {
    values.set(target as UnknownMutableSignal, value)
  })
}

export function planUpdate<Value>(
  mapper: (value: Value) => Value
): (target: MutableSignal<Value>) => SignalChange
export function planUpdate<Value>(
  mapper: (value: Value) => Value,
  target: MutableSignal<Value>
): SignalChange
export function planUpdate<Value>(
  mapper: (value: Value) => Value,
  target?: MutableSignal<Value>
): SignalChange | ((target: MutableSignal<Value>) => SignalChange) {
  if (target === undefined) {
    return (target) => planUpdate(mapper, target)
  }
  return change((values) => {
    values.set(
      target as UnknownMutableSignal,
      mapper(stagedValue(values, target))
    )
  })
}

export function transaction(
  changes: readonly SignalChange[]
): Effect<unknown, never, Unit> {
  return () => {
    if (publicationActive) {
      queuedTransactions.push({ changes })
      return unit
    }

    const changed = commitChanges(changes)
    const notification = publish(changed)
    return notification === undefined ? unit : finishPublication(notification)
  }
}

function commitChanges(changes: readonly SignalChange[]): Set<SignalState> {
  const values: StagedValues = new Map()
  for (const planned of changes) {
    planned.stage(values)
  }
  if (values.size === 0) {
    return new Set()
  }
  const changed = new Set<SignalState>()
  for (const [target, value] of values) {
    target.commit(value)
    changed.add(signalState(target))
  }
  signalEpoch += 1
  return changed
}

function publish(changed: ReadonlySet<SignalState>): Promise<Unit> | undefined {
  if (changed.size === 0 || subscribedSignals.size === 0) {
    return undefined
  }

  const affected = new Map<SignalState, boolean>()
  const signals = [...subscribedSignals]
    .filter((source) => signalWasAffected(source, changed, affected))
    .sort((left, right) => left.depth - right.depth)
  if (signals.length === 0) {
    return undefined
  }

  publicationActive = true
  return notifySubscribers(signals)
}

function signalWasAffected(
  source: SignalState,
  changed: ReadonlySet<SignalState>,
  memo: Map<SignalState, boolean>
): boolean {
  if (changed.has(source)) {
    return true
  }
  const cached = memo.get(source)
  if (cached !== undefined) {
    return cached
  }
  const affected = source.dependencies.some((dependency) =>
    signalWasAffected(dependency, changed, memo)
  )
  memo.set(source, affected)
  return affected
}

async function notifySubscribers(
  signals: readonly SignalState[]
): Promise<Unit> {
  let firstDefect: unknown
  for (const source of signals) {
    const value = source.current()
    for (const subscriber of [...source.subscribers]) {
      if (!subscriber.active) {
        continue
      }
      try {
        await subscriber.observer(value)(subscriber.environment)
      } catch (error) {
        cancelSubscriber(subscriber)
        firstDefect ??= error
      }
    }
  }
  if (firstDefect !== undefined) {
    throw firstDefect
  }
  return unit
}

async function finishPublication(initial: Promise<Unit>): Promise<Unit> {
  let firstDefect: unknown
  try {
    await initial
  } catch (error) {
    firstDefect = error
  } finally {
    publicationActive = false
  }

  while (queuedTransactions.length > 0) {
    const queued = queuedTransactions.shift()
    if (queued === undefined) {
      break
    }
    try {
      const changed = commitChanges(queued.changes)
      const notification = publish(changed)
      if (notification !== undefined) {
        await notification
      }
    } catch (error) {
      firstDefect ??= error
    } finally {
      publicationActive = false
    }
  }

  if (firstDefect !== undefined) {
    throw firstDefect
  }
  return unit
}

export function map<Value, Result>(
  mapper: (value: Value) => Result
): (source: Signal<Value>) => Signal<Result>
export function map<Value, Result>(
  mapper: (value: Value) => Result,
  source: Signal<Value>
): Signal<Result>
export function map<Value, Result>(
  mapper: (value: Value) => Result,
  source?: Signal<Value>
): Signal<Result> | ((source: Signal<Value>) => Signal<Result>) {
  if (source === undefined) {
    return (source) => map(mapper, source)
  }
  return derived(() => mapper(source.current()), [source])
}

export function combine<Left, Right, Result>(
  mapper: (left: Left) => (right: Right) => Result
): (left: Signal<Left>) => (right: Signal<Right>) => Signal<Result>
export function combine<Left, Right, Result>(
  mapper: (left: Left) => (right: Right) => Result,
  left: Signal<Left>
): (right: Signal<Right>) => Signal<Result>
export function combine<Left, Right, Result>(
  mapper: (left: Left) => (right: Right) => Result,
  left: Signal<Left>,
  right: Signal<Right>
): Signal<Result>
export function combine<Left, Right, Result>(
  mapper: (left: Left) => (right: Right) => Result,
  left?: Signal<Left>,
  right?: Signal<Right>
):
  | Signal<Result>
  | ((left: Signal<Left>) => (right: Signal<Right>) => Signal<Result>)
  | ((right: Signal<Right>) => Signal<Result>) {
  if (left === undefined) {
    return (left: Signal<Left>) => (right: Signal<Right>) =>
      combine(mapper, left, right)
  }
  if (right === undefined) {
    return (right: Signal<Right>) => combine(mapper, left, right)
  }
  return derived(() => mapper(left.current())(right.current()), [left, right])
}

export function constant<Value>(value: Value): Signal<Value> {
  return derived(() => value, [])
}

export function subscribe<Environment, Value>(
  observer: (value: Value) => Effect<Environment, never, Unit>
): (source: Signal<Value>) => Effect<Environment, never, Subscription>
export function subscribe<Environment, Value>(
  observer: (value: Value) => Effect<Environment, never, Unit>,
  source: Signal<Value>
): Effect<Environment, never, Subscription>
export function subscribe<Environment, Value>(
  observer: (value: Value) => Effect<Environment, never, Unit>,
  source?: Signal<Value>
):
  | Effect<Environment, never, Subscription>
  | ((source: Signal<Value>) => Effect<Environment, never, Subscription>) {
  if (source === undefined) {
    return (source: Signal<Value>) => subscribe(observer, source)
  }
  return async (environment: Environment) => {
    await observer(source.current())(environment)
    const state = signalState(source)
    const subscriber: Subscriber = {
      active: true,
      environment,
      observer: observer as (value: unknown) => Effect<unknown, never, Unit>,
      source: state,
    }
    state.subscribers.add(subscriber)
    subscribedSignals.add(state)
    return Object.freeze({
      [subscriptionBrand]: true as const,
      [cancelSubscriptionBrand]: () => cancelSubscriber(subscriber),
    })
  }
}

export function unsubscribe(
  subscription: Subscription
): Effect<unknown, never, Unit> {
  return () => {
    subscription[cancelSubscriptionBrand]()
    return unit
  }
}

function cancelSubscriber(subscriber: Subscriber): void {
  if (!subscriber.active) {
    return
  }
  subscriber.active = false
  subscriber.source.subscribers.delete(subscriber)
  if (subscriber.source.subscribers.size === 0) {
    subscribedSignals.delete(subscriber.source)
  }
}
