import type { Unit } from "./effect"

/** Runtime dictionary for the standard `Semigroup<String>` instance. */
export const stringSemigroup = Object.freeze({
  append:
    (left: string) =>
    (right: string): string =>
      `${left}${right}`,
})

/** Runtime dictionary for the standard `Monoid<String>` instance. */
export const stringMonoid = Object.freeze({
  ...stringSemigroup,
  empty: (_unit: Unit): string => "",
})
