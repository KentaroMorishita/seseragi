import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const functionIds = [
  "03-function-definitions",
  "one-parameter-function",
  "function-return-type",
  "04-function-calls",
  "multiple-parameter-function",
  "function-value",
  "currying-from-code",
  "partial-application",
  "application-grouping",
  "dollar-application",
  "pipeline-application",
  "mixed-application",
  "05-pipelines",
] as const

describe("Tour function curriculum", () => {
  test("teaches definition, application and value flow one step at a time", () => {
    const functions = tourLessons.slice(15, 15 + functionIds.length)

    expect(functions.map(({ id }) => id)).toEqual([...functionIds])
    expect(functions.every(({ deliveryIssue }) => deliveryIssue === 173)).toBe(
      true
    )
    expect(functions.every(({ format }) => format !== undefined)).toBe(true)
    for (const [index, lesson] of functions.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "comments-and-tools" : functionIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        functionIds[index + 1] ?? "06-records-and-structs"
      )
    }
  })

  test("keeps currying, partial application and both operators separate", () => {
    expect(introducedBy("currying-from-code")).toEqual(["currying"])
    expect(introducedBy("partial-application")).toEqual([
      "partial-application",
    ])
    expect(introducedBy("application-grouping")).toEqual([
      "application-grouping",
    ])
    expect(introducedBy("dollar-application")).toEqual([
      "dollar-application",
    ])
    expect(introducedBy("pipeline-application")).toEqual(["pipeline"])
    expect(introducedBy("mixed-application")).toEqual(["mixed-application"])
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
