import { describe, expect, test } from "bun:test"
import curriculumJson from "../../../examples/tour/curriculum.json"
import {
  type CanonicalTourContent,
  parseTourCurriculum,
  type TourCurriculum,
  type TourSampleRole,
  tourCoverageReport,
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
    hasExpectedFailure: false,
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

  test("keeps curriculum labels and lesson summaries plain text", () => {
    const lessonMarker = mutableCurriculum()
    mutableLessons(lessonMarker)[0]!.summary = "`rich` summary"
    expect(() => parseTourCurriculum(lessonMarker)).toThrow(
      "must remain plain text"
    )

    const categoryMarker = mutableCurriculum()
    categoryMarker.categories[0]!.title = "## Heading"
    expect(() => parseTourCurriculum(categoryMarker)).toThrow(
      "must remain plain text"
    )
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
            hasExpectedFailure: false,
            source: 'pub effect fn main = println "ok"',
            guide: "guide",
          },
        ],
        samples
      )
    ).not.toThrow()
  })

  test("accepts a lesson inserted between existing stable ids", () => {
    const value = mutableCurriculum()
    const chapter = value.categories
      .flatMap(({ chapters }) => chapters)
      .find(({ lessons }) => lessons.length >= 3)!
    const previous = chapter.lessons[0]!
    const next = chapter.lessons[1]!
    const inserted = {
      ...structuredClone(previous),
      id: "inserted-middle-lesson",
      order: previous.order + 1,
      title: "途中追加lesson",
      summary: "Stable IDの間へ追加される検証lessonです。",
      goal: "既存indexを契約にせず途中追加できる。",
      introducedSurfaces: ["inserted-middle-surface"],
      requiredSurfaces: [...previous.introducedSurfaces],
      prerequisites: [previous.id],
      content: "lessons/inserted-middle-lesson/lesson.json",
    }
    for (const lesson of mutableLessons(value)) {
      if (lesson.order > previous.order) lesson.order += 1
    }
    next.prerequisites = next.prerequisites.map((id) =>
      id === previous.id ? inserted.id : id
    )
    chapter.lessons.splice(1, 0, inserted)
    value.requiredTopics.push("inserted-middle-surface")
    const audit = value.sampleAudit.find(
      ({ sampleId }) => sampleId === inserted.seedSamples[0]
    )!
    audit.tourLessons.push(inserted.id)
    const contentIndex = previous.order
    const insertedContent: CanonicalTourContent = {
      id: inserted.id,
      interactive: inserted.capabilities.includes("dom"),
      hasExpectedOutput: !inserted.capabilities.includes("dom"),
      hasExpectedFailure: false,
      source: 'pub effect fn main = println "inserted"',
      guide: "guide",
    }
    const parsed = parseTourCurriculum(value)

    expect(() =>
      validateTourCurriculum(
        parsed,
        [
          ...content.slice(0, contentIndex),
          insertedContent,
          ...content.slice(contentIndex),
        ],
        samples
      )
    ).not.toThrow()
    expect(tourCurriculumLessons(parsed).map(({ id }) => id)).toEqual([
      ...lessons.slice(0, contentIndex).map(({ id }) => id),
      inserted.id,
      ...lessons.slice(contentIndex).map(({ id }) => id),
    ])
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
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          mutableLessons(value)[1]!.prerequisites = []
        }),
        content,
        samples
      )
    ).toThrow("unreachable from canonical root")
  })

  test("requires introduced surfaces exactly once and before their use", () => {
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
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          const values = mutableLessons(value)
          values[1]!.requiredSurfaces = [values[2]!.introducedSurfaces[0]!]
        }),
        content,
        samples
      )
    ).toThrow("before string-literal introduces it")
    expect(() =>
      validateTourCurriculum(
        mutate((value) => {
          mutableLessons(value)[1]!.requiredSurfaces = ["missing-surface"]
        }),
        content,
        samples
      )
    ).toThrow("but no lesson introduces it")
  })

  test("reports compiler surface coverage and audits central concept count", () => {
    expect(tourCoverageReport(curriculum)).toMatchObject({
      inventoryTopicCount: curriculum.requiredTopics.length,
      coveredTopicCount: curriculum.requiredTopics.length,
      missingTopics: [],
      unexpectedTopics: [],
    })
    expect(
      tourCoverageReport(
        mutate((value) => {
          const lesson = mutableLessons(value).find(({ introducedSurfaces }) =>
            introducedSurfaces.includes("main")
          )!
          lesson.introducedSurfaces = lesson.introducedSurfaces.filter(
            (topic) => topic !== "main"
          )
        })
      ).missingTopics
    ).toEqual(["main"])

    const emptyFocus = mutableCurriculum()
    mutableLessons(emptyFocus)[0]!.focus = []
    expect(() => parseTourCurriculum(emptyFocus)).toThrow(
      "focus must contain one or two central concepts"
    )
    const excessiveFocus = mutableCurriculum()
    mutableLessons(excessiveFocus)[0]!.focus = ["one", "two", "three"]
    expect(() => parseTourCurriculum(excessiveFocus)).toThrow(
      "focus must contain one or two central concepts"
    )
  })

  test("requires canonical content and an expected result for every lesson", () => {
    expect(() =>
      validateTourCurriculum(curriculum, content.slice(1), samples)
    ).toThrow("content ids must exactly match")
    expect(() =>
      validateTourCurriculum(
        curriculum,
        content.map((lesson, index) =>
          index === 0
            ? {
                ...lesson,
                hasExpectedOutput: false,
                hasExpectedFailure: false,
              }
            : lesson
        ),
        samples
      )
    ).toThrow("requires an expected result")
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
  focus: string[]
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
