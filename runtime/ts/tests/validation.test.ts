import { expect, test } from "bun:test"
import { intEq, stringEq } from "../src/equality"
import {
  consNonEmpty,
  fromArray,
  singleton,
  toArray,
  toListNonEmpty,
} from "../src/list"
import {
  renderDebug,
  renderShow,
  stringDebug,
  stringShow,
  validationDebug,
  validationShow,
} from "../src/show"
import { Left, Right } from "../src/sum"
import * as v from "../src/validation"

const toArrayNonEmpty = <A>(value: import("../src/list").NonEmptyList<A>) =>
  toArray(toListNonEmpty(value))

test("Validation helpers preserve payloads and all errors across explicit Either conversion", () => {
  const payload = () => 42
  expect(v.valid(payload).value).toBe(payload)
  const errors = consNonEmpty("first", fromArray(["second", "first"]))
  expect(v.invalidMany(errors).value).toBe(errors)
  expect(v.fromEither(Left("bad"))).toEqual(v.Invalid(singleton("bad")))
  expect(v.fromEither(Right(payload)).value).toBe(payload)
  expect(v.toEither(v.Valid(payload))).toEqual(Right(payload))
  expect(v.toEither(v.Invalid(errors))).toEqual(Left(errors))
})

test("Functor maps only Valid once and keeps Invalid identity", () => {
  const calls: number[] = []
  const f = (n: number) => {
    calls.push(n)
    return n + 1
  }
  const invalid = v.invalid("bad")
  expect(v.validationFunctor.map(f)(invalid)).toBe(invalid)
  expect(v.validationFunctor.map(f)(v.valid(3))).toEqual(v.Valid(4))
  expect(calls).toEqual([3])
})

test("Applicative covers all branches and accumulates non-empty errors left to right", () => {
  const calls: number[] = []
  const wrapped = v.Valid((n: number) => {
    calls.push(n)
    return n + 1
  })
  const left = v.invalidMany(consNonEmpty("a", fromArray(["b"])))
  const right = v.invalidMany(consNonEmpty("c", fromArray(["d"])))
  const apply = v.validationApplicative.apply
  expect(apply(wrapped)(v.Valid(3))).toEqual(v.Valid(4))
  expect(apply(left)(v.Valid(3))).toBe(left)
  expect(apply(wrapped)(right)).toBe(right)
  const accumulated = apply(left)(right)
  expect(accumulated.tag).toBe("Invalid")
  if (accumulated.tag !== "Invalid") throw new Error("expected errors")
  expect(toArrayNonEmpty(accumulated.value)).toEqual(["a", "b", "c", "d"])
  expect(toArrayNonEmpty(left.value)).toEqual(["a", "b"])
  expect(toArrayNonEmpty(right.value)).toEqual(["c", "d"])
  expect(calls).toEqual([3])
})

test("curried independent inputs keep every error and do not manufacture Monad", () => {
  const apply = v.validationApplicative.apply
  const make = (a: number) => (b: number) => (c: number) => a + b + c
  const result = apply(
    apply(apply(v.validationApplicative.pure(make))(v.invalid("first")))(
      v.invalid("second")
    )
  )(v.invalid("third"))
  expect(result.tag).toBe("Invalid")
  if (result.tag !== "Invalid") throw new Error("expected errors")
  expect(toArrayNonEmpty(result.value)).toEqual(["first", "second", "third"])
  expect("flatMap" in v.validationApplicative).toBe(false)
  expect("validationMonad" in v).toBe(false)
})

test("conditional Eq distinguishes constructors, payloads, and ordered errors", () => {
  const eq = v.validationEq(stringEq, intEq).eq
  expect(eq(v.Valid(1))(v.Valid(1))).toBe(true)
  expect(eq(v.Valid(1))(v.Valid(2))).toBe(false)
  expect(eq(v.Valid(1))(v.invalid("bad"))).toBe(false)
  expect(eq(v.invalid("bad"))(v.Valid(1))).toBe(false)
  const errors = v.Invalid(consNonEmpty("a", fromArray(["b"])))
  expect(eq(errors)(v.Invalid(consNonEmpty("a", fromArray(["b"]))))).toBe(true)
  expect(eq(errors)(v.Invalid(consNonEmpty("b", fromArray(["a"]))))).toBe(false)
})

test("Show and Debug compose payload dictionaries with NonEmptyList documents", () => {
  const show = validationShow(stringShow, stringShow).show
  const debug = validationDebug(stringDebug, stringDebug).debug
  expect(show(v.Valid("ok"))).toBe("Valid ok")
  expect(debug(v.Valid("ok\n"))).toBe('Valid "ok\\n"')
  const errors = v.Invalid(consNonEmpty("a", fromArray(["b\n"])))
  expect(show(errors)).toBe("Invalid `[a, b\n]")
  expect(debug(errors)).toBe('Invalid `["a", "b\\n"]')
  const options = { layout: "multiline" as const, indentWidth: 4 }
  expect(
    renderShow(validationShow(stringShow, stringShow), errors, options)
  ).toContain("\n")
  expect(
    renderDebug(validationDebug(stringDebug, stringDebug), errors, {
      layout: "auto",
      maxWidth: 1,
    })
  ).toContain("\n")
  expect(
    renderDebug(validationDebug(stringDebug, stringDebug), errors, {
      layout: "auto",
      maxWidth: 100,
    })
  ).toBe(debug(errors))
})

test("Functor and Applicative laws preserve successes and ordered failures", () => {
  const { map, pure, apply } = v.validationApplicative
  const values: v.Validation<string, number>[] = [
    v.Valid(3),
    v.invalid("x"),
    v.Invalid(consNonEmpty("x", fromArray(["y"]))),
  ]
  const f = (n: number) => n + 2
  const g = (n: number) => n * 3
  for (const value of values) {
    expect(map((n: number) => n)(value)).toEqual(value)
    expect(map((n: number) => f(g(n)))(value)).toEqual(map(f)(map(g)(value)))
    expect(apply(pure((n: number) => n))(value)).toEqual(value)
    expect(apply(pure(f))(value)).toEqual(map(f)(value))
  }
  expect(apply(pure(f))(pure(3))).toEqual(pure(f(3)))
  const functions: v.Validation<string, (n: number) => number>[] = [
    pure(f),
    v.invalid("f"),
  ]
  for (const wrapped of functions) {
    expect(apply(wrapped)(pure(3))).toEqual(
      apply(pure((fn: (n: number) => number) => fn(3)))(wrapped)
    )
    for (const inner of [pure(g), v.invalid("g")]) {
      for (const value of values) {
        const compose =
          (outer: (n: number) => number) =>
          (inside: (n: number) => number) =>
          (n: number) =>
            outer(inside(n))
        expect(
          apply(apply(apply(pure(compose))(wrapped))(inner))(value)
        ).toEqual(apply(wrapped)(apply(inner)(value)))
      }
    }
  }
})
