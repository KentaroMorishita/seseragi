import { describe, expect, test } from "bun:test"
import curriculumJson from "../../../examples/tour/curriculum.json"
import {
  type CanonicalTourContent,
  parseTourCurriculum,
  type TourCurriculum,
  type TourSampleRole,
  tourCurriculumLessons,
  validateTourCurriculum,
} from "../../../scripts/tour-curriculum"

const curriculum = parseTourCurriculum(curriculumJson)
const lessons = tourCurriculumLessons(curriculum)
const content = lessons.map(
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
  test("accepts nested categories, chapters, canonical content and sample audit", () => {
    expect(() =>
      validateTourCurriculum(curriculum, content, samples)
    ).not.toThrow()
  })

  test("accepts another category, chapter and stable lesson id without a count limit", () => {
    const value = mutableCurriculum()
    const currentLessons = mutableLessons(value)
    const previous = currentLessons.at(-1)!
    const appended = {
      ...structuredClone(previous),
      id: "open-ended-lesson",
      order: currentLessons.length + 1,
      title: "追加lesson",
      summary: "件数上限なしの検証lessonです。",
      goal: "manifest追加だけでcurriculumへ参加できる。",
      introducedSurfaces: ["open-ended-surface"],
      requiredSurfaces: [],
      prerequisites: [previous.id],
      content: "lessons/open-ended-lesson/lesson.json",
    }
    value.categories.push({
      id: "open-ended-category",
      order: value.categories.length + 1,
      title: "追加category",
      summary: "dataだけで追加します。",
      chapters: [
        {
          id: "open-ended-chapter",
          order: 1,
          title: "追加chapter",
          summary: "switch文を必要としません。",
          lessons: [appended],
        },
      ],
    })
    value.requiredTopics.push("open-ended-surface")
    const audit = value.sampleAudit.find(
      ({ sampleId }) => sampleId === appended.seedSamples[0]
    )!
    audit.tourLessons.push(appended.id)
    const parsed = parseTourCurriculum(value)

    expect(() =>
      validateTourCurriculum(
        parsed,
        [
          ...content,
          {
            id: appended.id,
            interactive: appended.capabilities.includes("dom"),
            hasExpectedOutput: !appended.capabilities.includes("dom"),
            source: 'pub effect fn main = println "ok"',
            guide: "guide",
          },
        ],
        samples
      )
    ).not.toThrow()
  })

  test("rejects duplicate ids and display-order contradictions", () => {
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          value.categories[1]!.id = value.categories[0]!.id
        }),
        content,
        samples
      )
    ).toThrow("Duplicate Tour category id")
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          value.categories[1]!.chapters[0]!.id =
            value.categories[0]!.chapters[0]!.id
        }),
        content,
        samples
      )
    ).toThrow("Duplicate Tour chapter id")
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          mutableLessons(value)[1]!.id = mutableLessons(value)[0]!.id
        }),
        content,
        samples
      )
    ).toThrow("Duplicate Tour lesson id")
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          value.categories[1]!.order = 20
        }),
        content,
        samples
      )
    ).toThrow("has order 20; expected 2")
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          mutableLessons(value)[4]!.order = 200
        }),
        content,
        samples
      )
    ).toThrow("has order 200; expected 5")
  })

  test("rejects missing prerequisites, cycles and forward-only prerequisites", () => {
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          mutableLessons(value)[1]!.prerequisites = ["missing-lesson"]
        }),
        content,
        samples
      )
    ).toThrow("references missing prerequisite missing-lesson")
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          const values = mutableLessons(value)
          values[0]!.prerequisites = [values[1]!.id]
          values[1]!.prerequisites = [values[0]!.id]
        }),
        content,
        samples
      )
    ).toThrow("Tour prerequisite cycle")
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          const values = mutableLessons(value)
          values[0]!.prerequisites = [values[1]!.id]
          values[1]!.prerequisites = []
        }),
        content,
        samples
      )
    ).toThrow("must appear earlier")
  })

  test("requires every introduced surface exactly once and on the checklist", () => {
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          const lesson = mutableLessons(value).find(({ introducedSurfaces }) =>
            introducedSurfaces.includes("main")
          )!
          lesson.introducedSurfaces = lesson.introducedSurfaces.filter(
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
          mutableLessons(value)[1]!.introducedSurfaces.push("main")
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
    expect(() =>
      validateTourCurriculum(
        curriculum,
        content.map((lesson, index) =>
          index === 0
            ? { ...lesson, formatNextLessonId: "missing-lesson" }
            : lesson
        ),
        samples
      )
    ).toThrow(
      `next connection must be ${tourCurriculumLessons(curriculum)[1]!.id}`
    )
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
  const value = mutableCurriculum()
  change(value)
  return parseTourCurriculum(value)
}

function mutableCurriculum(): MutableCurriculum {
  return structuredClone(curriculumJson) as unknown as MutableCurriculum
}

function mutableLessons(value: MutableCurriculum): MutableLesson[] {
  return value.categories.flatMap(({ chapters }) =>
    chapters.flatMap(({ lessons }) => lessons)
  )
}

type MutableLesson = {
  id: string
  order: number
  title: string
  summary: string
  goal: string
  introducedSurfaces: string[]
  requiredSurfaces: string[]
  prerequisites: string[]
  capabilities: string[]
  content: string
  seedSamples: string[]
}

type MutableCurriculum = {
  requiredTopics: string[]
  categories: Array<{
    id: string
    order: number
    title: string
    summary: string
    chapters: Array<{
      id: string
      order: number
      title: string
      summary: string
      lessons: MutableLesson[]
    }>
  }>
  sampleAudit: Array<{
    sampleId: string
    decision: string
    tourLessons: string[]
  }>
} & Record<string, unknown>
