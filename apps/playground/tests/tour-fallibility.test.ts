import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const fallibilityIds = [
  "maybe-role",
  "maybe-constructors",
  "maybe-match",
  "maybe-default",
  "maybe-map",
  "maybe-combine",
  "maybe-short-circuit",
  "either-role",
  "either-constructors",
  "either-match",
  "either-map-error",
  "09-maybe-and-either",
  "effect-failure-bridge",
] as const

describe("Tour fallibility curriculum", () => {
  test("stages Maybe and Either before the Effect chapter", () => {
    const lessons = tourLessons.slice(56, 56 + fallibilityIds.length)

    expect(lessons.map(({ id }) => id)).toEqual([...fallibilityIds])
    expect(lessons.every(({ deliveryIssue }) => deliveryIssue === 176)).toBe(
      true
    )
    for (const [index, lesson] of lessons.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "08-collections-and-ranges" : fallibilityIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        fallibilityIds[index + 1] ?? "10-effects-and-do"
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

  test("shows Nothing short-circuiting before the pure Effect bridge", () => {
    const shortCircuit = lessonById("maybe-short-circuit")
    expect(shortCircuit.source).toContain("let source: Maybe<Int> = Nothing")
    expect(shortCircuit.source).toContain("flatMap")
    expect(shortCircuit.expectedOutput.trim()).toBe("result: Nothing")

    const bridge = lessonById("effect-failure-bridge")
    expect(bridge.source).toContain(
      'Either<String, Int> = Left "invalid value"'
    )
    expect(bridge.source).not.toContain("Effect<")
    expect(bridge.source).not.toContain("fails ")
    expect(bridge.source).not.toContain("with ")
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
