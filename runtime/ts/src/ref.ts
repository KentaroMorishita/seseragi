import type { Effect, Unit } from "./effect"

const refBrand: unique symbol = Symbol("seseragi.ref")

export type Ref<Value> = Readonly<{ readonly [refBrand]: Value }>

const cells = new WeakMap<object, unknown>()

export function make<Value>(
  initial: Value
): Effect<unknown, never, Ref<Value>> {
  return () => {
    const reference = Object.freeze({}) as Ref<Value>
    cells.set(reference, initial)
    return reference
  }
}

export function get<Value>(
  reference: Ref<Value>
): Effect<unknown, never, Value> {
  return () => read(reference)
}

export function set<Value>(
  value: Value,
  reference: Ref<Value>
): Effect<unknown, never, Unit> {
  return () => {
    ensureReference(reference)
    cells.set(reference, value)
    return undefined
  }
}

export function update<Value>(
  transform: (value: Value) => Value,
  reference: Ref<Value>
): Effect<unknown, never, Unit> {
  return () => {
    const next = transform(read(reference))
    cells.set(reference, next)
    return undefined
  }
}

export function modify<Value, Result>(
  transform: (value: Value) => readonly [Result, Value],
  reference: Ref<Value>
): Effect<unknown, never, Result> {
  return () => {
    const [result, next] = transform(read(reference))
    cells.set(reference, next)
    return result
  }
}

function read<Value>(reference: Ref<Value>): Value {
  ensureReference(reference)
  return cells.get(reference) as Value
}

function ensureReference(reference: object): void {
  if (!cells.has(reference)) {
    throw new TypeError("Ref value does not use the runtime brand")
  }
}
