import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const collectionIds = [
  "array-values",
  "array-access",
  "list-values",
  "list-decomposition",
  "range-values",
  "collection-map",
  "collection-filter",
  "collection-reduce-step",
  "collection-reduce",
  "collection-append-concat",
  "collection-comprehension",
  "collection-pipeline",
  "empty-collections",
  "08-collections-and-ranges",
] as const

describe("Tour collection curriculum", () => {
  test("teaches construction, transformation and reduction sequentially", () => {
    const lessons = tourLessons.slice(42, 42 + collectionIds.length)

    expect(lessons.map(({ id }) => id)).toEqual([...collectionIds])
    expect(
      lessons.every(({ id, deliveryIssue }) =>
        id === "collection-map" ? deliveryIssue === 262 : deliveryIssue === 175
      )
    ).toBe(true)
    for (const [index, lesson] of lessons.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "data-shape-selection" : collectionIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        collectionIds[index + 1] ?? "maybe-role"
      )
    }
  })

  test("keeps every collection lesson runnable and diagnosable", () => {
    for (const id of collectionIds) {
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

  test("keeps callback-first and collection-last flow visible", () => {
    const map = lessonById("collection-map")
    expect(map.source).toContain("map double [")
    expect(map.source).toContain("double <$> [")
    expect(map.source).toContain("map double `[1, 2, 3]")
    expect(map.source).toContain("double <$> `[1, 2, 3]")
    expect(map.introducedSurfaces).toContain("collection-functor-operator")
    expect(lessonById("collection-filter").source).toContain(
      "arrays.filter even ["
    )
    expect(lessonById("collection-pipeline").source).toContain(
      "|> arrays.filter even"
    )
    expect(lessonById("collection-pipeline").source).toContain("|> map double")
    expect(lessonById("collection-pipeline").source).toContain(
      "|> reduce 0 (+)"
    )
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
