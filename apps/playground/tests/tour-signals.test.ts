import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const coreSignalIds = [
  "signals-value-difference",
  "signals-write-operator",
  "signals-combine",
  "signals-handler-boundary",
] as const

const sideTripSignalId = "signals-switch-map"

describe("Tour Signal curriculum", () => {
  test("stages core Signal steps sequentially and keeps switchMap in side trips", () => {
    const lessons = tourLessons.slice(23, 23 + coreSignalIds.length)

    expect(lessons.map(({ id }) => id)).toEqual([...coreSignalIds])
    expect(lessons.every(({ deliveryIssue }) => deliveryIssue === 1348)).toBe(
      true
    )
    for (const [index, lesson] of lessons.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "abstraction-trait-operation" : coreSignalIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        coreSignalIds[index + 1] ?? "web-html-value"
      )
    }

    const switching = lessonById(sideTripSignalId)
    expect(switching.deliveryIssue).toBe(1348)
    expect(switching.prerequisites).toEqual(["abstraction-signal-contract"])
    expect(switching.format?.next.lessonId).toBe("abstraction-impl-method")
  })

  test("keeps every Signal lesson runnable and diagnosable", () => {
    for (const id of [...coreSignalIds, sideTripSignalId]) {
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
    const read = lessonById("signals-value-difference")
    expect(read.source).toContain("signals.make 42")
    expect(read.source).toContain("named <- signals.read view")
    expect(read.source).toContain("viaView <- *view")
    expect(read.source).toContain("viaState <- *state")

    const write = lessonById("signals-write-operator")
    expect(write.source).toContain("signals.set 20 state")
    expect(write.source).toContain("state := 30")
    expect(write.source).toContain("signals.update")
    expect(write.diagnosticSource).toContain("Signal<Int>")
    expect(write.diagnosticSource).toContain("view := 20")

    const combine = lessonById("signals-combine")
    expect(combine.source).toContain("signals.map")
    expect(combine.source).toContain("<$>")
    expect(combine.source).toContain("signals.combine")
    expect(combine.source).toContain("count := 3")

    const boundary = lessonById("signals-handler-boundary")
    expect(boundary.source).toContain("<*>")
    expect(boundary.source).toContain("signals.transaction")
    expect(boundary.source).toContain("signals.planSet")
    expect(boundary.source).toContain("MutableSignal<Int>")

    const switching = lessonById(sideTripSignalId)
    expect(switching.source).toContain("signals.switchMap")
  })

  test("keeps the Signal chapter independent of DOM and Web UI", () => {
    for (const id of [...coreSignalIds, sideTripSignalId]) {
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
