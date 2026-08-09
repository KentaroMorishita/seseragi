import { describe, expect, test } from "bun:test"
import type { TourInlineRichText, TourLessonFormat } from "../src/tour/content"
import {
  tourCategories,
  tourChapters,
  tourLessons,
} from "../src/tour/curriculum"
import {
  guideInlineSourceProblem,
  parseGuideInline,
} from "../src/ui/guide-markdown"

describe("structured Tour inline rich text", () => {
  test("audits every declared inline field in the generated catalog", () => {
    const fields = tourLessons.flatMap((lesson) =>
      lesson.format === undefined ? [] : inlineFields(lesson.format)
    )
    expect(fields.length).toBeGreaterThan(250)
    expect(new Set(fields.map(({ role }) => role))).toEqual(
      new Set([
        "prerequisite",
        "walkthrough.body",
        "introduced.body",
        "exercise.instruction",
        "diagnostic.body",
        "recap",
        "next.body",
        "notes",
      ])
    )
    for (const field of fields) {
      expect(
        guideInlineSourceProblem(field.value),
        field.identity
      ).toBeUndefined()
    }
  })

  test("renders the primitive comparison markers as inline code tokens", () => {
    const lesson = tourLessons.find(({ id }) => id === "primitive-comparison")
    if (lesson?.format === undefined) {
      throw new Error("missing primitive-comparison structured lesson")
    }

    expect(codeValues(lesson.format.prerequisite)).toContain("+")
    expect(
      codeValues(lesson.format.walkthrough[0]?.body ?? inline("missing"))
    ).toEqual(expect.arrayContaining(["42 > 40", "True"]))
  })

  test("keeps identifiers and accessibility headings outside rich text", async () => {
    const main = await Bun.file(
      new URL("../src/tour/main.ts", import.meta.url)
    ).text()
    expect(main).toContain("lessonTitle.textContent = currentLesson.title")
    expect(main).toContain("title.textContent = step.heading")
    expect(main).toContain("diagnosticHeading.textContent")
    expect(main).toContain("renderTourInline(prerequisiteCopy")
    expect(main).toContain("renderTourInline(diagnosticCopy")

    const plainText = [
      ...tourCategories.flatMap(({ title, summary }) => [title, summary]),
      ...tourChapters.flatMap(({ title, summary }) => [title, summary]),
      ...tourLessons.flatMap(({ title, summary, goal, focus }) => [
        title,
        summary,
        goal,
        ...focus,
      ]),
    ]
    expect(plainText.filter(hasRichTextMarker)).toEqual([])
    expect(
      new Set(
        tourLessons
          .filter(({ format }) => format !== undefined)
          .map(({ challenge }) => challenge)
      )
    ).toEqual(new Set([""]))
  })
})

function inlineFields(format: TourLessonFormat): readonly Readonly<{
  identity: string
  role: string
  value: TourInlineRichText
}>[] {
  return [
    field("prerequisite", format.prerequisite),
    ...format.walkthrough.map((step, index) =>
      field("walkthrough.body", step.body, index)
    ),
    ...format.introduced.map((surface, index) =>
      field("introduced.body", surface.body, index)
    ),
    field("exercise.instruction", format.exercise.instruction),
    field("diagnostic.body", format.diagnostic.body),
    ...format.recap.map((item, index) => field("recap", item, index)),
    field("next.body", format.next.body),
    ...(format.notes ?? []).map((note, index) => field("notes", note, index)),
  ]
}

function field(
  role: string,
  value: TourInlineRichText,
  index?: number
): Readonly<{ identity: string; role: string; value: TourInlineRichText }> {
  return {
    identity: index === undefined ? role : `${role}.${index}`,
    role,
    value,
  }
}

function codeValues(value: TourInlineRichText): string[] {
  return parseGuideInline(value).flatMap((node) =>
    node.kind === "code" ? [node.value] : []
  )
}

function inline(value: string): TourInlineRichText {
  return value as TourInlineRichText
}

function hasRichTextMarker(value: string): boolean {
  return /(?:`|\*\*|\*[^*\n]+\*|\[[^\]\n]+\]\([^\n)]*\)|^ {0,3}(?:#{1,6}\s|[-+*]\s|\d+[.)]\s|>\s))/mu.test(
    value
  )
}
