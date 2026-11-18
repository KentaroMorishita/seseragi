import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const fallibilityIds = [
  "maybe-match",
  "maybe-short-circuit",
  "09-maybe-and-either",
  "either-map-error",
] as const

describe("Tour fallibility curriculum", () => {
  test("stages Maybe and Either before the Effect chapter", () => {
    const lessons = tourLessons.slice(12, 12 + fallibilityIds.length)

    expect(lessons.map(({ id }) => id)).toEqual([...fallibilityIds])
    expect(lessons.every(({ deliveryIssue }) => deliveryIssue === 1348)).toBe(
      true
    )
    for (const [index, lesson] of lessons.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "collection-pipeline" : fallibilityIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        fallibilityIds[index + 1] ?? "effect-pure-vs-effect"
      )
    }
  })

  test("keeps every fallibility lesson runnable and diagnosable", () => {
    for (const id of fallibilityIds) {
      const lesson = lessonById(id)
      expect(lesson.source.trim()).not.toBe("")
      expect(lesson.expectedOutput.trim()).not.toBe("")
      expect(lesson.exerciseSource.trim()).not.toBe("")
      expect(lesson.exerciseExpectedOutput.trim()).not.toBe("")
      expect(lesson.diagnosticSource.trim()).not.toBe("")
      expect(lesson.diagnosticOutput).toContain("error[")
      expect(lesson.format?.walkthrough.length).toBeGreaterThan(0)
      expect(lesson.format?.recap.length).toBeGreaterThan(0)
    }
  })

  test("introduces Maybe and Either operations before the Effect chapter", () => {
    const maybeMatch = lessonById("maybe-match")
    expect(maybeMatch.source).toContain('primary ?? "Guest"')
    expect(maybeMatch.source).toContain("match value {")

    const combined = lessonById("maybe-short-circuit")
    expect(combined.source).toContain("map double (Just 21)")
    expect(combined.source).toContain("double <$> Just 21")
    expect(combined.source).toContain("add <$> Just 20 <*> Just 22")
    expect(combined.source).toContain("flatMap next input")
    expect(combined.source).toContain("input >>= next")
    expect(combined.source).toContain("missing >>= next")

    const compare = lessonById("09-maybe-and-either")
    expect(compare.source).toContain("Maybe<String>")
    expect(compare.source).toContain("Either<String, Int>")

    const either = lessonById("either-map-error")
    expect(either.source).toContain("map increment success")
    expect(either.source).toContain("increment <$> success")
    expect(either.source).toContain("increment <$> failure")
    expect(either.source).toContain("either.mapLeft")
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
