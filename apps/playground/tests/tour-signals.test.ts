import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const signalIds = [
  "signals-value-difference",
  "signals-make",
  "signals-readonly-coercion",
  "signals-set",
  "signals-observable-update",
  "signals-map",
  "signals-functor-operator",
  "signals-constant-pure",
  "signals-combine",
  "signals-applicative-glitch-free",
  "signals-monad-boundary",
  "signals-switch-map",
  "signals-handler-boundary",
] as const

describe("Tour Signal curriculum", () => {
  test("stages one Signal concept at a time", () => {
    const lessons = tourLessons.slice(100, 100 + signalIds.length)

    expect(lessons.map(({ id }) => id)).toEqual([...signalIds])
    expect(lessons.every(({ deliveryIssue }) => deliveryIssue === 179)).toBe(
      true
    )
    for (const [index, lesson] of lessons.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "abstraction-custom-operator" : signalIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        signalIds[index + 1] ?? "13-components-and-web-ui"
      )
    }
  })

  test("keeps every Signal lesson runnable and diagnosable", () => {
    for (const id of signalIds) {
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

  test("connects the named APIs, operators, and ownership boundaries", () => {
    const set = lessonById("signals-set")
    expect(set.source).toContain("signals.set 42 state")

    const observable = lessonById("signals-observable-update")
    expect(observable.source).toContain("before <-")
    expect(observable.source).toContain("after <-")

    const functor = lessonById("signals-functor-operator")
    expect(functor.source).toContain("signals.map double source")
    expect(functor.source).toContain("double <$> source")

    const applicative = lessonById("signals-applicative-glitch-free")
    expect(applicative.source).toContain("<*>")
    expect(applicative.source).toContain("signals.transaction")
    expect(applicative.source).toContain("signals.planSet")

    const monad = lessonById("signals-monad-boundary")
    expect(monad.source).not.toContain(">>=")
    expect(monad.diagnosticSource).toContain(">>=")

    const switching = lessonById("signals-switch-map")
    expect(switching.source).toContain("signals.switchMap")

    const boundary = lessonById("signals-handler-boundary")
    expect(boundary.source).toContain("state: MutableSignal<Int>")
    expect(boundary.source).toContain("-> Signal<Int> = state")
  })

  test("keeps the Signal chapter independent of DOM and Web UI", () => {
    for (const id of signalIds) {
      const source = lessonById(id).source
      expect(source).not.toContain("std/web")
      expect(source).not.toContain("std/dom")
    }
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
