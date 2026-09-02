import type { List, NonEmptyList } from "./list"

export type Eq<Value> = Readonly<{
  eq: (left: Value) => (right: Value) => boolean
}>

const strictEq = <Value>(): Eq<Value> =>
  Object.freeze({
    eq:
      (left: Value) =>
      (right: Value): boolean =>
        left === right,
  })

/** Standard `Eq<Int>` dictionary. */
export const intEq = strictEq<number>()

/** Standard `Eq<Bool>` dictionary. */
export const boolEq = strictEq<boolean>()

/** Standard `Eq<String>` dictionary. */
export const stringEq = strictEq<string>()

/** Standard `Eq<Char>` dictionary. */
export const charEq = strictEq<string>()

/** Standard `Eq<Unit>` dictionary. */
export const unitEq = strictEq<undefined>()

export const arrayEq = <Value>(element: Eq<Value>): Eq<ReadonlyArray<Value>> =>
  Object.freeze({
    eq:
      (left: ReadonlyArray<Value>) =>
      (right: ReadonlyArray<Value>): boolean => {
        if (left.length !== right.length) return false
        for (let index = 0; index < left.length; index += 1) {
          if (!element.eq(left[index] as Value)(right[index] as Value)) {
            return false
          }
        }
        return true
      },
  })

export const listEq = <Value>(element: Eq<Value>): Eq<List<Value>> =>
  Object.freeze({
    eq:
      (left: List<Value>) =>
      (right: List<Value>): boolean => {
        let leftCursor = left
        let rightCursor = right
        while (leftCursor.tag === "Cons" && rightCursor.tag === "Cons") {
          if (!element.eq(leftCursor.head)(rightCursor.head)) return false
          leftCursor = leftCursor.tail
          rightCursor = rightCursor.tail
        }
        return leftCursor.tag === "Empty" && rightCursor.tag === "Empty"
      },
  })

export const nonEmptyListEq = <Value>(
  element: Eq<Value>
): Eq<NonEmptyList<Value>> =>
  Object.freeze({
    eq:
      (left: NonEmptyList<Value>) =>
      (right: NonEmptyList<Value>): boolean =>
        element.eq(left.head)(right.head) &&
        listEq(element).eq(left.tail)(right.tail),
  })

export const tupleEq = <Value extends readonly unknown[]>(
  ...elements: ReadonlyArray<Eq<unknown>>
): Eq<Value> =>
  Object.freeze({
    eq:
      (left: Value) =>
      (right: Value): boolean =>
        left.length === right.length &&
        elements.length === left.length &&
        elements.every((element, index) =>
          element.eq(left[index])(right[index])
        ),
  })

export const recordEq = <Value extends object>(
  names: ReadonlyArray<string>,
  optional: ReadonlyArray<boolean>,
  ...fields: ReadonlyArray<Eq<unknown>>
): Eq<Value> => {
  if (names.length !== optional.length || names.length !== fields.length) {
    throw new Error("record Eq metadata length mismatch")
  }
  return Object.freeze({
    eq:
      (left: Value) =>
      (right: Value): boolean => {
        const leftRecord = left as Readonly<Record<string, unknown>>
        const rightRecord = right as Readonly<Record<string, unknown>>
        for (let index = 0; index < names.length; index += 1) {
          const name = names[index] as string
          const leftHas = Object.hasOwn(leftRecord, name)
          const rightHas = Object.hasOwn(rightRecord, name)
          if (!(optional[index] as boolean) && (!leftHas || !rightHas)) {
            return false
          }
          if (leftHas !== rightHas) return false
          if (
            leftHas &&
            !(fields[index] as Eq<unknown>).eq(leftRecord[name])(
              rightRecord[name]
            )
          ) {
            return false
          }
        }
        return true
      },
  })
}
