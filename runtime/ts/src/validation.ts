import { type Eq, nonEmptyListEq } from "./equality"
import { type NonEmptyList, nonEmptyListSemigroup, singleton } from "./list"
import {
  type Either,
  Left,
  type Left as LeftValue,
  Right,
  type Right as RightValue,
} from "./sum"
import type { RuntimeDictionary } from "./traversable"

export type Valid<A> = Readonly<{ tag: "Valid"; value: A }>
export type Invalid<E> = Readonly<{ tag: "Invalid"; value: NonEmptyList<E> }>
export type Validation<E, A> = Valid<A> | Invalid<E>

export function Valid<A>(value: A): Valid<A> {
  return { tag: "Valid", value }
}

export function Invalid<E>(value: NonEmptyList<E>): Invalid<E> {
  return { tag: "Invalid", value }
}

export const valid = Valid
export const invalidMany = Invalid
export const invalid = <E>(error: E): Invalid<E> => Invalid(singleton(error))

// Capture the complete input before distributing over the tags. This also
// preserves independent generic payloads in the compiler's structural union ABI.
export function fromEither<T extends Either<unknown, unknown>>(
  value: T
): T extends LeftValue<infer E>
  ? Invalid<E>
  : T extends RightValue<infer A>
    ? Valid<A>
    : never
export function fromEither<E, A>(value: Either<E, A>): Validation<E, A>
export function fromEither<E, A>(value: Either<E, A>): Validation<E, A> {
  return value.tag === "Left" ? invalid(value.value) : Valid(value.value)
}

export function toEither<T extends Validation<unknown, unknown>>(
  value: T
): T extends Invalid<infer E>
  ? LeftValue<NonEmptyList<E>>
  : T extends Valid<infer A>
    ? RightValue<A>
    : never
export function toEither<E, A>(
  value: Validation<E, A>
): Either<NonEmptyList<E>, A>
export function toEither<E, A>(
  value: Validation<E, A>
): Either<NonEmptyList<E>, A> {
  return value.tag === "Invalid" ? Left(value.value) : Right(value.value)
}

export const validationFunctor = Object.freeze({
  map:
    <A, B>(f: (value: A) => B) =>
    <E>(value: Validation<E, A>): Validation<E, B> =>
      value.tag === "Invalid" ? value : Valid(f(value.value)),
})

// A Valid function has no error payload from which TypeScript can infer E.
// Keep that parameter on the second application instead of fixing it to unknown.
function applyValidation<A, B>(
  wrapped: Valid<(value: A) => B>
): {
  (value: Valid<A>): Valid<B>
  <E>(value: Validation<E, A>): Validation<E, B>
}
function applyValidation<E>(
  wrapped: Invalid<E>
): <F, A>(value: Validation<F, A>) => Invalid<E | F>
function applyValidation<E, A, B>(
  wrapped: Validation<E, (value: A) => B>
): <F>(value: Validation<F, A>) => Validation<E | F, B>
function applyValidation<E, A, B>(wrapped: Validation<E, (value: A) => B>) {
  return <F>(value: Validation<F, A>): Validation<E | F, B> => {
    if (wrapped.tag === "Invalid") {
      return value.tag === "Invalid"
        ? Invalid(
            nonEmptyListSemigroup.append<E | F>(wrapped.value)(value.value)
          )
        : wrapped
    }
    return value.tag === "Invalid" ? value : Valid(wrapped.value(value.value))
  }
}

export const validationApplicative = Object.freeze({
  ...validationFunctor,
  pure: Valid,
  apply: applyValidation,
})

export function validationEq<E, A>(
  errorEvidence: Eq<E> | RuntimeDictionary,
  valueEvidence: Eq<A> | RuntimeDictionary
): Eq<Validation<E, A>> {
  const errors = nonEmptyListEq(errorEvidence as Eq<E>)
  const values = valueEvidence as Eq<A>
  return Object.freeze({
    eq:
      (left: Validation<E, A>) =>
      (right: Validation<E, A>): boolean => {
        if (left.tag === "Valid") {
          return right.tag === "Valid" && values.eq(left.value)(right.value)
        }
        return right.tag === "Invalid" && errors.eq(left.value)(right.value)
      },
  })
}
