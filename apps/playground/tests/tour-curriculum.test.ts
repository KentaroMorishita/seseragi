import { describe, expect, test } from "bun:test"
import curriculumJson from "../../../examples/tour/curriculum.json"
import {
  type CanonicalTourContent,
  parseTourCurriculum,
  type TourCurriculum,
  type TourSampleRole,
  validateTourCurriculum,
} from "../../../scripts/tour-curriculum"

const curriculum = parseTourCurriculum(curriculumJson)
const content = curriculum.lessons.map(
  (lesson): CanonicalTourContent => ({
    id: lesson.id,
    interactive: lesson.capabilities.includes("dom"),
    hasExpectedOutput: !lesson.capabilities.includes("dom"),
    source: 'pub effect fn main = println "ok"',
    guide: "guide",
  })
)
const samples = curriculum.sampleAudit.map(
  ({ sampleId: id, currentKind: kind }): TourSampleRole => ({ id, kind })
)

describe("Tour curriculum coverage", () => {
  test("accepts the canonical order, topics, content and sample audit", () => {
    expect(() =>
      validateTourCurriculum(curriculum, content, samples)
    ).not.toThrow()
  })

  test("rejects duplicate lesson ids and order contradictions", () => {
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          value.lessons[1]!.id = value.lessons[0]!.id
        }),
        content,
        samples
      )
    ).toThrow("Duplicate Tour lesson id")
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          value.lessons[4]!.order = 15
        }),
        content,
        samples
      )
    ).toThrow("has order 15; expected 5")
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          value.lessons[4]!.prerequisites = [value.lessons[0]!.id]
        }),
        content,
        samples
      )
    ).toThrow("prerequisites must be")
  })

  test("requires every topic exactly once and on the independent checklist", () => {
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          value.lessons[0]!.introduces = value.lessons[0]!.introduces.filter(
            (topic) => topic !== "main"
          )
        }),
        content,
        samples
      )
    ).toThrow("required topic(s) missing: main")
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          value.lessons[1]!.introduces.push("main")
        }),
        content,
        samples
      )
    ).toThrow("introduced by both")
  })

  test("requires canonical content and expected output for every lesson", () => {
    expect(() =>
      validateTourCurriculum(curriculum, content.slice(1), samples)
    ).toThrow("content ids must exactly match")
    expect(() =>
      validateTourCurriculum(
        curriculum,
        content.map((lesson, index) =>
          index === 0 ? { ...lesson, hasExpectedOutput: false } : lesson
        ),
        samples
      )
    ).toThrow("requires expected output")
  })

  test("rejects excluded imports and sample audit drift", () => {
    expect(() =>
      validateTourCurriculum(
        curriculum,
        content.map((lesson, index) =>
          index === 0
            ? { ...lesson, source: 'import * as bytes from "std/bytes"' }
            : lesson
        ),
        samples
      )
    ).toThrow("imports excluded")
    expect(() =>
      validateTourCurriculum(curriculum, content, [
        { ...samples[0]!, kind: "showcase" },
        ...samples.slice(1),
      ])
    ).toThrow("actual kind is showcase")
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          value.sampleAudit[0]!.tourLessons = []
        }),
        content,
        samples
      )
    ).toThrow("tourLessons must be")
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          value.sampleAudit[0]!.decision = "discover-recipe"
        }),
        content,
        samples
      )
    ).toThrow("decision must be tour-seed-and-recipe")
  })
})

function mutate(change: (value: MutableCurriculum) => void): TourCurriculum {
  const value = structuredClone(curriculumJson) as MutableCurriculum
  change(value)
  return parseTourCurriculum(value)
}

type MutableCurriculum = {
  lessons: Array<{
    id: string
    order: number
    prerequisites: string[]
    introduces: string[]
  }>
  sampleAudit: Array<{
    decision: string
    tourLessons: string[]
  }>
} & Record<string, unknown>
