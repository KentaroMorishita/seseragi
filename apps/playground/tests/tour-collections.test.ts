import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const coreCollectionIds = [
  "08-collections-and-ranges",
  "collection-pipeline",
] as const

const sideTripCollectionIds = [
  "list-values",
  "list-decomposition",
  "collection-reduce-step",
  "collection-append-concat",
  "collection-comprehension",
  "empty-collections",
] as const

const collectionIds = [...coreCollectionIds, ...sideTripCollectionIds] as const

describe("Tour collection curriculum", () => {
  test("teaches core Array/Range pipelines and keeps List/detail lessons in side trips", () => {
    const coreLessons = tourLessons.slice(10, 10 + coreCollectionIds.length)
    const sideTripLessons = tourLessons.slice(-sideTripCollectionIds.length)

    expect(coreLessons.map(({ id }) => id)).toEqual([...coreCollectionIds])
    expect(sideTripLessons.map(({ id }) => id)).toEqual([
      ...sideTripCollectionIds,
    ])
    expect(
      [...coreLessons, ...sideTripLessons].every(
        ({ deliveryIssue }) => deliveryIssue === 1348
      )
    ).toBe(true)
    for (const [index, lesson] of coreLessons.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "exhaustive-match" : coreCollectionIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        coreCollectionIds[index + 1] ?? "maybe-match"
      )
    }
    for (const [index, lesson] of sideTripLessons.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "collection-pipeline" : sideTripCollectionIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        sideTripCollectionIds[index + 1] ?? null
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
    const pipeline = lessonById("collection-pipeline")
    expect(pipeline.source).toContain("|> arrays.filter even")
    expect(pipeline.source).toContain("|> map double")
    expect(pipeline.source).toContain("|> reduce 0 (+)")
    expect(pipeline.source).toContain("double <$> numbers")
    expect(pipeline.introducedSurfaces).toContain("collection-functor-operator")
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
