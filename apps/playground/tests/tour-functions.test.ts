import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const functionIds = ["03-function-definitions", "05-pipelines"] as const

describe("Tour function curriculum", () => {
  test("teaches definition, partial application, and pipelines sequentially", () => {
    const functions = tourLessons.slice(3, 3 + functionIds.length)

    expect(functions.map(({ id }) => id)).toEqual([...functionIds])
    expect(functions.every(({ deliveryIssue }) => deliveryIssue === 1348)).toBe(
      true
    )
    expect(functions.every(({ format }) => format !== undefined)).toBe(true)
    for (const [index, lesson] of functions.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "if-expression" : functionIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        functionIds[index + 1] ?? "tuple-and-record"
      )
    }
  })

  test("introduces partial application, dollar application and pipelines", () => {
    expect(introducedBy("03-function-definitions")).toEqual(
      expect.arrayContaining(["fn", "currying", "partial-application"])
    )
    expect(introducedBy("05-pipelines")).toEqual(
      expect.arrayContaining(["dollar-application", "pipeline"])
    )
  })

  test("keeps every function lesson runnable, changeable and diagnosable", () => {
    for (const id of functionIds) {
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
})

function introducedBy(id: string): readonly string[] {
  return lessonById(id).introducedSurfaces
}

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
