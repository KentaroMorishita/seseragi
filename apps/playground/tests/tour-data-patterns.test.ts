import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const dataPatternIds = [
  "tuple-and-record",
  "06-records-and-structs",
  "adt-definition",
  "07-adts-and-patterns",
  "exhaustive-match",
] as const

describe("Tour data and pattern curriculum", () => {
  test("teaches records, structs, ADTs, match, and exhaustiveness one step at a time", () => {
    const lessons = tourLessons.slice(5, 5 + dataPatternIds.length)

    expect(lessons.map(({ id }) => id)).toEqual([...dataPatternIds])
    expect(lessons.every(({ deliveryIssue }) => deliveryIssue === 1348)).toBe(
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
    const lesson = lessonById("07-adts-and-patterns")
    const payloadTemplate = ["Shipped to ", "$", "{city}"].join("")

    expect(lesson.source).toContain('Shipped "Osaka"')
    expect(lesson.source).toContain(payloadTemplate)
    expect(lesson.expectedOutput).toContain("Shipped to Osaka")
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
