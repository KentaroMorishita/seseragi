import { describe, expect, test } from "bun:test"
import { arrayApplicative, arrayTraversable } from "../src/array"
import {
  type Effect,
  effectApplicative,
  fail,
  run,
  succeed,
} from "../src/effect"
import {
  fromArray,
  listTraversable,
  nonEmptyListTraversable,
} from "../src/list"
import {
  eitherApplicative,
  Just,
  Left,
  maybeApplicative,
  Nothing,
  Right,
} from "../src/sum"

describe("Traversable standard dictionaries", () => {
  test("traverse Array through Maybe preserving source order", () => {
    const visited: number[] = []
    const result = arrayTraversable.traverse((value: number) => {
      visited.push(value)
      return Just(value * 10)
    })([1, 2, 3])(maybeApplicative)

    expect(visited).toEqual([1, 2, 3])
    expect(result).toEqual(Just([10, 20, 30]))

    const failed = arrayTraversable.traverse((value: number) =>
      value === 2 ? Nothing : Just(value)
    )([1, 2, 3])(maybeApplicative)
    expect(failed).toEqual(Nothing)
  })

  test("delegate List failure behavior to Either Applicative", () => {
    const result = listTraversable.traverse((value: number) =>
      value >= 2 ? Left(`stopped at ${value}`) : Right(value)
    )(fromArray([1, 2, 3]))(eitherApplicative)

    expect(result).toEqual(Left("stopped at 2"))
    expect(
      listTraversable.traverse((value: number) => Right(value * 2))(
        fromArray([1, 2, 3])
      )(eitherApplicative)
    ).toEqual(Right(fromArray([2, 4, 6])))
  })

  test("preserve NonEmptyList shape and order", () => {
    const result = nonEmptyListTraversable.traverse((value: number) =>
      Just(value + 1)
    )({ tag: "NonEmpty", head: 1, tail: fromArray([2, 3]) })(maybeApplicative)

    expect(result).toEqual(
      Just({ tag: "NonEmpty", head: 2, tail: fromArray([3, 4]) })
    )
    expect(
      nonEmptyListTraversable.traverse(() => Nothing)({
        tag: "NonEmpty",
        head: 1,
        tail: fromArray([]),
      })(maybeApplicative)
    ).toEqual(Nothing)
  })

  test("lift empty collections without calling the callback", () => {
    const callback = () => {
      throw new Error("empty traversal called f")
    }
    expect(arrayTraversable.traverse(callback)([])(maybeApplicative)).toEqual(
      Just([])
    )
    expect(
      listTraversable.traverse(callback)(fromArray([]))(eitherApplicative)
    ).toEqual(Right(fromArray([])))
  })

  test("preserve independent branches of a nondeterministic Applicative", () => {
    const result = arrayTraversable.traverse((value: number) => [
      value,
      value + 10,
    ])([1, 2])(arrayApplicative) as number[][]
    expect(result).toEqual([
      [1, 2],
      [1, 12],
      [11, 2],
      [11, 12],
    ])
    expect(new Set(result).size).toBe(4)
  })

  test("traverse large inputs without recursive rebuilding or prefix copying", () => {
    const values = Array.from({ length: 50_000 }, (_, index) => index)
    expect(arrayTraversable.traverse(Just)(values)(maybeApplicative)).toEqual(
      Just(values)
    )
  })

  test("keep Effect cold and stop later effects at the target typed failure", async () => {
    const executed: number[] = []
    const effect = arrayTraversable.traverse(
      (value: number): Effect<unknown, string, number> =>
        async (environment, context) => {
          executed.push(value)
          if (value === 2) return fail("stopped")(environment, context)
          return value
        }
    )([1, 2, 3])(effectApplicative) as Effect<unknown, string, number[]>
    expect(executed).toEqual([])
    expect(await run(effect, {})).toEqual({ kind: "failure", error: "stopped" })
    expect(executed).toEqual([1, 2])
  })

  test("keep large deferred traversals stack-safe and repeatable", async () => {
    const values = Array.from({ length: 10_000 }, (_, index) => index)
    const effect = arrayTraversable.traverse(succeed)(values)(effectApplicative)
    const first = await run(effect, {})
    const second = await run(effect, {})
    expect(first).toEqual({ kind: "success", value: values })
    expect(second).toEqual(first)
    if (first.kind === "success" && second.kind === "success") {
      expect(first.value).not.toBe(second.value)
    }
  })
})
