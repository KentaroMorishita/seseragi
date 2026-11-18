import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const effectIds = [
  "effect-pure-vs-effect",
  "10-effects-and-do",
  "effect-failure-short-circuit",
  "effect-map-error",
  "effect-with-capability",
] as const

describe("Tour Effect curriculum", () => {
  test("stages execution, failure and contracts sequentially", () => {
    const lessons = tourLessons.slice(16, 16 + effectIds.length)

    expect(lessons.map(({ id }) => id)).toEqual([...effectIds])
    expect(lessons.every(({ deliveryIssue }) => deliveryIssue === 1348)).toBe(
      true
    )
    for (const [index, lesson] of lessons.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "either-map-error" : effectIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        effectIds[index + 1] ?? "abstraction-type-parameter"
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

  test("fixes the short-circuit, recovery and capability contracts", () => {
    const shortCircuit = lessonById("effect-failure-short-circuit")
    expect(shortCircuit.source).toContain('println "before"')
    expect(shortCircuit.source).toContain("fail Rejected")
    expect(shortCircuit.expectedOutput).toBe("before")
    expect(shortCircuit.expectedFailure).toBe("Rejected")

    const mapped = lessonById("effect-map-error")
    expect(mapped.source).toContain("|> mapError ParseFailure")
    expect(mapped.source).toContain("|> recover recoverOrder")
    expect(mapped.expectedFailure).toBe("")
    expect(mapped.expectedOutput).toContain("fallback (no stock: coffee)")

    const capability = lessonById("effect-with-capability")
    expect(capability.source).toContain(
      "-> Unit with Console fails ConsoleError"
    )
    expect(capability.source).toContain("effect fn reportPair")
  })

  test("relates Effect flatMap, >>=, and do after the Maybe lessons", () => {
    const lesson = lessonById("10-effects-and-do")
    expect(lesson.source).toContain("flatMap double source")
    expect(lesson.source).toContain("source >>= double")
    expect(lesson.source).toContain("value <- source")
    expect(lesson.expectedOutput).toBe("flatMap: 42\n>>=: 42\ndo: 42")
    expect(lesson.requiredSurfaces).toContain("maybe-monad-operator")
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
