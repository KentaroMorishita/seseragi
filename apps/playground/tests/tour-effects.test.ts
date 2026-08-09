import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const effectIds = [
  "effect-pure-vs-effect",
  "effect-single-operation",
  "effect-do-block",
  "effect-sequence",
  "effect-bind-success",
  "effect-succeed-value",
  "effect-failure-short-circuit",
  "effect-fails-type",
  "effect-with-capability",
  "effect-inferred-contract",
  "effect-explicit-contract",
  "effect-map-error",
  "effect-value-boundary",
  "10-effects-and-do",
] as const

describe("Tour Effect curriculum", () => {
  test("stages execution, failure and contracts sequentially", () => {
    const lessons = tourLessons.slice(69, 69 + effectIds.length)

    expect(lessons.map(({ id }) => id)).toEqual([...effectIds])
    expect(lessons.every(({ deliveryIssue }) => deliveryIssue === 177)).toBe(
      true
    )
    for (const [index, lesson] of lessons.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "effect-failure-bridge" : effectIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        effectIds[index + 1] ?? "abstraction-concrete-generic"
      )
    }
  })

  test("keeps every Effect lesson runnable or intentionally failing", () => {
    for (const id of effectIds) {
      const lesson = lessonById(id)
      expect(lesson.source.trim()).not.toBe("")
      expect(
        lesson.expectedOutput.trim() !== "" ||
          lesson.expectedFailure.trim() !== ""
      ).toBe(true)
      expect(lesson.exerciseSource.trim()).not.toBe("")
      expect(lesson.exerciseExpectedOutput.trim()).not.toBe("")
      expect(lesson.diagnosticSource.trim()).not.toBe("")
      expect(lesson.diagnosticOutput).toContain("error[")
      expect(lesson.format?.walkthrough.length).toBeGreaterThan(0)
      expect(lesson.format?.recap.length).toBeGreaterThan(0)
    }
  })

  test("fixes the short-circuit and application failure contracts", () => {
    const shortCircuit = lessonById("effect-failure-short-circuit")
    expect(shortCircuit.source).toContain('println "before"')
    expect(shortCircuit.source).toContain('println "unreachable"')
    expect(shortCircuit.expectedOutput).toBe("before")
    expect(shortCircuit.expectedFailure).toBe("Rejected")

    const mapped = lessonById("effect-map-error")
    expect(mapped.source).toContain("|> mapError InvalidInputFailure")
    expect(mapped.expectedOutput).toBe("")
    expect(mapped.expectedFailure).toBe("InvalidInputFailure InvalidInput bad")
  })

  test("shows inferred and explicit contracts as separate forms", () => {
    const inferred = lessonById("effect-inferred-contract")
    expect(inferred.source).toContain("pub effect fn main = println")
    expect(inferred.source).not.toContain("with Console")
    expect(inferred.source).not.toContain("fails ConsoleError")

    const explicit = lessonById("effect-explicit-contract")
    expect(explicit.source).toContain("-> Unit")
    expect(explicit.source).toContain("with Console")
    expect(explicit.source).toContain("fails ConsoleError")
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
