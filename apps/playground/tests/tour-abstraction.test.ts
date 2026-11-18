import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const coreAbstractionIds = [
  "abstraction-type-parameter",
  "abstraction-trait-operation",
] as const

const sideTripAbstractionIds = [
  "abstraction-functor-map",
  "abstraction-applicative-apply",
  "abstraction-monad-bind",
  "abstraction-type-comparison",
  "abstraction-signal-contract",
  "abstraction-impl-method",
  "abstraction-custom-operator",
] as const

describe("Tour generic and Trait curriculum", () => {
  test("stages generics and traits before signals and places laws in side trips", () => {
    const core = tourLessons.slice(21, 21 + coreAbstractionIds.length)
    expect(core.map(({ id }) => id)).toEqual([...coreAbstractionIds])
    expect(core.every(({ deliveryIssue }) => deliveryIssue === 1348)).toBe(true)
    for (const [index, lesson] of core.entries()) {
      expect(lesson.prerequisites).toContain(
        index === 0 ? "effect-with-capability" : coreAbstractionIds[index - 1]!
      )
      expect(lesson.format?.next.lessonId).toBe(
        coreAbstractionIds[index + 1] ?? "signals-value-difference"
      )
    }

    const sideTrips = tourLessons.slice(67, 67 + sideTripAbstractionIds.length)
    expect(sideTrips.map(({ id }) => id)).toEqual([...sideTripAbstractionIds])
    expect(sideTrips.every(({ deliveryIssue }) => deliveryIssue === 1348)).toBe(
      true
    )
    for (const [index, lesson] of sideTrips.entries()) {
      expect(lesson.prerequisites).toContain(
        index === 0
          ? "abstraction-trait-operation"
          : sideTripAbstractionIds[index - 1]!
      )
      expect(lesson.format?.next.lessonId).toBe(
        sideTripAbstractionIds[index + 1] ?? "design-learning-map"
      )
    }
  })

  test("keeps every abstraction lesson runnable and diagnosable", () => {
    for (const id of [...coreAbstractionIds, ...sideTripAbstractionIds]) {
      const lesson = lessonById(id)
      expect(lesson.source.trim()).not.toBe("")
      expect(lesson.expectedOutput.trim()).not.toBe("")
      expect(lesson.exerciseSource.trim()).not.toBe("")
      expect(lesson.exerciseExpectedOutput.trim()).not.toBe("")
      expect(lesson.diagnosticSource.trim()).not.toBe("")
      expect(lesson.diagnosticOutput).toContain("error[")
      expect(lesson.format?.walkthrough.length).toBeGreaterThan(0)
      expect(lesson.format?.introduced.length).toBeGreaterThan(0)
      expect(lesson.format?.recap.length).toBeGreaterThan(0)
    }
  })

  test("connects named operations to operators and preserves the Signal boundary", () => {
    const typeParams = lessonById("abstraction-type-parameter")
    expect(typeParams.source).toContain("type Box<A>")
    expect(typeParams.source).toContain("type Pair<A, B>")

    const traitLesson = lessonById("abstraction-trait-operation")
    expect(traitLesson.source).toContain("deriving Eq, Show")
    expect(traitLesson.source).toContain("trait Label<A>")
    expect(traitLesson.source).toContain("where Label<A>")

    const functor = lessonById("abstraction-functor-map")
    expect(functor.source).toContain("let source: Maybe<Int>")
    expect(functor.source).toContain(
      "let named: Maybe<String> = map label source"
    )
    expect(functor.source).toContain("label <$> source")
    expect(functor.format?.walkthrough).toHaveLength(3)
    expect(functor.prerequisites).toEqual([
      "abstraction-trait-operation",
      "collection-pipeline",
      "maybe-short-circuit",
      "either-map-error",
    ])

    const applicative = lessonById("abstraction-applicative-apply")
    expect(applicative.source).toContain("flatMap (\\value: Int -> map")
    expect(applicative.source).toContain("let partial: Maybe<Int -> Int>")
    expect(applicative.source).toContain("apply partial right")
    expect(applicative.source).toContain("operatorPartial <*> right")
    expect(applicative.format?.walkthrough).toHaveLength(3)
    expect(applicative.prerequisites).toContain("maybe-short-circuit")

    const monad = lessonById("abstraction-monad-bind")
    expect(monad.source).toContain("flatMap halfIfEven input")
    expect(monad.source).toContain(">>= halfIfEven >>= decrementIfPositive")
    expect(monad.source).toContain("current <- value")
    expect(monad.format?.walkthrough).toHaveLength(3)
    expect(monad.prerequisites).toContain("maybe-short-circuit")
    expect(monad.prerequisites).toContain("10-effects-and-do")

    const signal = lessonById("abstraction-signal-contract")
    expect(signal.source).toContain("<$>")
    expect(signal.source).toContain("<*>")
    expect(signal.source).not.toContain(">>=")
    expect(signal.diagnosticSource).toContain(">>=")
  })

  test("compares all requested standard containers", () => {
    const lesson = lessonById("abstraction-type-comparison")
    expect(lesson.source).toContain("Maybe<Int>")
    expect(lesson.source).toContain("Either<String, Int>")
    expect(lesson.source).toContain("[1, 2]")
    expect(lesson.source).toContain("`[3, 4]")
    expect(lesson.source).toContain("succeed 4")
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
