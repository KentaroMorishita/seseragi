import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const foundationIds = [
  "01-hello-world",
  "program-entry",
  "string-literal",
  "string-template",
  "int-literal",
  "float-literal",
  "bool-literal",
  "unit-literal",
  "02-values-and-bindings",
  "local-let",
  "type-annotations",
  "type-inference",
  "primitive-arithmetic",
  "primitive-comparison",
  "comments-and-tools",
] as const

describe("Tour foundation curriculum", () => {
  test("teaches execution, values, types and tools as a sequential foundation", () => {
    const foundations = tourLessons.slice(0, foundationIds.length)

    expect(foundations.map(({ id }) => id)).toEqual([...foundationIds])
    expect(
      foundations.every(({ deliveryIssue }) => deliveryIssue === 172)
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

  test("introduces each primitive literal and type concept in its own lesson", () => {
    expect(introducedBy("string-literal")).toEqual([
      "string-literal",
      "println",
    ])
    expect(introducedBy("int-literal")).toEqual(["int"])
    expect(introducedBy("float-literal")).toEqual(["float"])
    expect(introducedBy("bool-literal")).toEqual(["bool"])
    expect(introducedBy("unit-literal")).toEqual(["unit"])
    expect(introducedBy("type-annotations")).toEqual(["type-annotation"])
    expect(introducedBy("type-inference")).toEqual(["type-inference"])
    expect(introducedBy("primitive-arithmetic")).toEqual([
      "primitive-arithmetic",
    ])
    expect(introducedBy("primitive-comparison")).toEqual(["comparison"])
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
