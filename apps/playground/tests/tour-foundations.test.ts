import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const foundationIds = [
  "01-hello-world",
  "02-values-and-bindings",
  "if-expression",
] as const

describe("Tour foundation curriculum", () => {
  test("teaches execution, values, types and branching as a sequential foundation", () => {
    const foundations = tourLessons.slice(0, foundationIds.length)

    expect(foundations.map(({ id }) => id)).toEqual([...foundationIds])
    expect(
      foundations.every(({ deliveryIssue }) => deliveryIssue === 1348)
    ).toBe(true)
    expect(foundations.every(({ format }) => format !== undefined)).toBe(true)
    for (const [index, lesson] of foundations.entries()) {
      expect(lesson.prerequisites).toEqual(
        index === 0 ? [] : [foundationIds[index - 1]!]
      )
      expect(lesson.format?.next.lessonId).toBe(
        foundationIds[index + 1] ?? "03-function-definitions"
      )
    }
  })

  test("introduces templates, primitive bindings, inference and if expressions", () => {
    expect(introducedBy("01-hello-world")).toContain("string-template")
    expect(introducedBy("02-values-and-bindings")).toEqual(
      expect.arrayContaining([
        "int",
        "float",
        "bool",
        "unit",
        "let",
        "type-annotation",
        "type-inference",
      ])
    )
    expect(introducedBy("if-expression")).toEqual(["if-expression"])
  })

  test("keeps every foundation runnable, changeable and diagnosable", () => {
    for (const id of foundationIds) {
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
