import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const abstractionIds = [
  "abstraction-concrete-generic",
  "abstraction-type-parameter",
  "abstraction-generic-adt-definition",
  "abstraction-generic-adt-use",
  "abstraction-multiple-type-parameters",
  "abstraction-type-constructor-parameter",
  "abstraction-trait-operation",
  "abstraction-user-instance",
  "abstraction-where-constraint",
  "abstraction-instance-selection",
  "abstraction-functor-map",
  "abstraction-applicative-apply",
  "abstraction-monad-bind",
  "abstraction-type-comparison",
  "abstraction-signal-contract",
  "abstraction-impl-method",
  "abstraction-custom-operator",
] as const

describe("Tour generic and Trait curriculum", () => {
  test("stages one abstraction concept at a time", () => {
    const lessons = tourLessons.slice(83, 83 + abstractionIds.length)

    expect(lessons.map(({ id }) => id)).toEqual([...abstractionIds])
    expect(
      lessons.slice(10, 13).every(({ deliveryIssue }) => deliveryIssue === 262)
    ).toBe(true)
    expect(
      [...lessons.slice(0, 10), ...lessons.slice(13)].every(
        ({ deliveryIssue }) => deliveryIssue === 178
      )
    ).toBe(true)
    for (const [index, lesson] of lessons.entries()) {
      expect(lesson.prerequisites).toContain(
        index === 0 ? "10-effects-and-do" : abstractionIds[index - 1]!
      )
      expect(lesson.format?.next.lessonId).toBe(
        abstractionIds[index + 1] ?? "signals-value-difference"
      )
    }
  })

  test("keeps every abstraction lesson runnable and diagnosable", () => {
    for (const id of abstractionIds) {
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
    const functor = lessonById("abstraction-functor-map")
    expect(functor.source).toContain("let source: Maybe<Int>")
    expect(functor.source).toContain(
      "let named: Maybe<String> = map label source"
    )
    expect(functor.source).toContain("label <$> source")
    expect(functor.format?.walkthrough).toHaveLength(4)
    expect(functor.prerequisites).toEqual([
      "abstraction-instance-selection",
      "collection-map",
      "maybe-map",
      "either-map-error",
    ])

    const applicative = lessonById("abstraction-applicative-apply")
    expect(applicative.source).toContain("flatMap (\\value: Int -> map")
    expect(applicative.source).toContain("let partial: Maybe<Int -> Int>")
    expect(applicative.source).toContain("apply partial right")
    expect(applicative.source).toContain("operatorPartial <*> right")
    expect(applicative.format?.walkthrough).toHaveLength(4)
    expect(applicative.prerequisites).toContain("maybe-combine")

    const monad = lessonById("abstraction-monad-bind")
    expect(monad.source).toContain("flatMap halfIfEven input")
    expect(monad.source).toContain(">>= halfIfEven >>= decrementIfPositive")
    expect(monad.source).toContain("current <- value")
    expect(monad.format?.walkthrough).toHaveLength(4)
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
