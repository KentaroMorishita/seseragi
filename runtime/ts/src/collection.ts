import type { Effect, Unit } from "./effect"
import type { Iterator } from "./iterator"

export type Reducible<Collection, Element> = Readonly<{
  reduce: <Accumulator>(
    initial: Accumulator
  ) => (
    step: (accumulator: Accumulator) => (value: Element) => Accumulator
  ) => (values: Collection) => Accumulator
}>

export type Iterable<Collection, Element> = Readonly<{
  iterate: (values: Collection) => Iterator<Element>
}>

type RuntimeDictionary = Readonly<Record<string, (...arguments_: any[]) => any>>

/** Join a reducible collection without depending on its concrete representation. */
export function join<Collection>(
  dictionary: RuntimeDictionary,
  separator: string,
  values: Collection
): string {
  const reducible = dictionary as Reducible<Collection, string>
  const [, result] = reducible.reduce<readonly [boolean, string]>([true, ""])(
    ([first, output]) =>
      (value) =>
        [false, first ? value : `${output}${separator}${value}`] as const
  )(values)
  return result
}

/** Sum a reducible collection using only its selected algebra dictionaries. */
export function sum<Collection, Element>(
  reducibleDictionary: RuntimeDictionary,
  zeroDictionary: RuntimeDictionary,
  addDictionary: RuntimeDictionary,
  values: Collection
): Element {
  const reducible = reducibleDictionary as Reducible<Collection, Element>
  const zero = zeroDictionary as Readonly<{
    zero: (unit: Unit) => Element
  }>
  const add = addDictionary as Readonly<{
    add: (left: Element) => (right: Element) => Element
  }>
  return reducible.reduce(zero.zero(undefined))(add.add)(values)
}

/** Multiply a reducible collection using its selected algebra dictionaries. */
export function product<Collection, Element>(
  reducibleDictionary: RuntimeDictionary,
  oneDictionary: RuntimeDictionary,
  mulDictionary: RuntimeDictionary,
  values: Collection
): Element {
  const reducible = reducibleDictionary as Reducible<Collection, Element>
  const one = oneDictionary as Readonly<{
    one: (unit: Unit) => Element
  }>
  const mul = mulDictionary as Readonly<{
    mul: (left: Element) => (right: Element) => Element
  }>
  return reducible.reduce(one.one(undefined))(mul.mul)(values)
}

/** Combine a reducible collection using only its selected Monoid dictionary. */
export function combine<Collection, Element>(
  reducibleDictionary: RuntimeDictionary,
  monoidDictionary: RuntimeDictionary,
  values: Collection
): Element {
  const reducible = reducibleDictionary as Reducible<Collection, Element>
  const monoid = monoidDictionary as Readonly<{
    append: (left: Element) => (right: Element) => Element
    empty: (unit: Unit) => Element
  }>
  return reducible.reduce(monoid.empty(undefined))(monoid.append)(values)
}

/** Stop at the first element accepted by the predicate. */
export function any<Collection, Element>(
  dictionary: RuntimeDictionary,
  predicate: (value: Element) => boolean,
  values: Collection
): boolean {
  const iterable = dictionary as Iterable<Collection, Element>
  let iterator = iterable.iterate(values)
  while (true) {
    const step = iterator.next()
    if (step.tag === "Nothing") return false
    const [value, rest] = step.value
    if (predicate(value)) return true
    iterator = rest
  }
}

/** Stop at the first element rejected by the predicate. */
export function all<Collection, Element>(
  dictionary: RuntimeDictionary,
  predicate: (value: Element) => boolean,
  values: Collection
): boolean {
  const iterable = dictionary as Iterable<Collection, Element>
  let iterator = iterable.iterate(values)
  while (true) {
    const step = iterator.next()
    if (step.tag === "Nothing") return true
    const [value, rest] = step.value
    if (!predicate(value)) return false
    iterator = rest
  }
}

/** Run one cold Effect at a time in the collection's declared iteration order. */
export function forEach<Collection, Environment, Failure, Element>(
  dictionary: RuntimeDictionary,
  action: (value: Element) => Effect<Environment, Failure, Unit>,
  values: Collection
): Effect<Environment, Failure, Unit> {
  const iterable = dictionary as Iterable<Collection, Element>
  return async (environment) => {
    let iterator = iterable.iterate(values)
    while (true) {
      const step = iterator.next()
      if (step.tag === "Nothing") {
        return undefined
      }
      const [value, rest] = step.value
      await action(value)(environment)
      iterator = rest
    }
  }
}
