import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const dataPatternIds = [
  "tuple-values",
  "tuple-pattern-binding",
  "record-values",
  "record-access-update",
  "06-records-and-structs",
  "struct-field-access",
  "simple-adt",
  "payload-adt",
  "single-constructor-match",
  "07-adts-and-patterns",
  "payload-pattern-binding",
  "nested-pattern-wildcard",
  "exhaustive-match",
  "data-shape-selection",
] as const

describe("Tour data and pattern curriculum", () => {
  test("teaches construction and decomposition one step at a time", () => {
    const lessons = tourLessons.slice(28, 28 + dataPatternIds.length)

    expect(lessons.map(({ id }) => id)).toEqual([...dataPatternIds])
    expect(lessons.every(({ deliveryIssue }) => deliveryIssue === 174)).toBe(
      true
    )
    expect(lessons.every(({ format }) => format !== undefined)).toBe(true)

    for (const [index, lesson] of lessons.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "05-pipelines" : dataPatternIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        dataPatternIds[index + 1] ?? "08-collections-and-ranges"
      )
    }
  })

  test("keeps every lesson runnable, changeable and diagnosable", () => {
    for (const id of dataPatternIds) {
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

  test("keeps the Shipped Osaka walkthrough traceable", () => {
    const source = lessonById("payload-pattern-binding").source
    const payloadTemplate = ["Shipped to ", "$", "{city}"].join("")

    expect(source).toContain('Shipped "Osaka"')
    expect(source).toContain(payloadTemplate)
    expect(lessonById("payload-pattern-binding").expectedOutput).toContain(
      "Shipped to Osaka"
    )
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
