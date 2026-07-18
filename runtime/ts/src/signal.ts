import { type Effect, type Unit, unit } from "./effect"

const signalBrand = Symbol("seseragi.signal")
const mutableSignalBrand = Symbol("seseragi.mutable-signal")
const signalChangeBrand = Symbol("seseragi.signal-change")

export type Signal<Value> = {
  readonly [signalBrand]: true
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

type UnknownMutableSignal = MutableSignal<unknown>
type StagedValues = Map<UnknownMutableSignal, unknown>

function mutable<Value>(initial: Value): MutableSignal<Value> {
  let current = initial
  return Object.freeze({
    [signalBrand]: true as const,
    [mutableSignalBrand]: true as const,
    current: () => current,
    commit: (value: Value) => {
      current = value
    },
  })
}

function derived<Value>(current: () => Value): Signal<Value> {
  return Object.freeze({
    [signalBrand]: true as const,
    current,
  })
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
    const values: StagedValues = new Map()
    for (const planned of changes) {
      planned.stage(values)
    }
    for (const [target, value] of values) {
      target.commit(value)
    }
    return unit
  }
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
  return derived(() => mapper(source.current()))
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
  return derived(() => mapper(left.current())(right.current()))
}

export function constant<Value>(value: Value): Signal<Value> {
  return derived(() => value)
}
