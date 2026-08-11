import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const webIds = [
  "web-html-value",
  "web-text-attributes",
  "web-style-class",
  "web-tags-children",
  "web-component-definition",
  "web-component-children",
  "web-component-props",
  "web-link-image",
  "web-click-action",
  "web-input-action",
  "web-change-action",
  "web-form-submit",
  "web-signal-preview",
  "web-dom-run",
  "web-typed-action",
  "web-accessibility-label",
  "web-feature-state",
] as const

describe("Tour Web UI curriculum", () => {
  test("stages static HTML through feature-owned state", () => {
    const lessons = tourLessons.slice(113, 113 + webIds.length)

    expect(lessons.map(({ id }) => id)).toEqual([...webIds])
    expect(lessons.every(({ deliveryIssue }) => deliveryIssue === 180)).toBe(
      true
    )
    for (const [index, lesson] of lessons.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "signals-handler-boundary" : webIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        webIds[index + 1] ?? "14-integrated-app"
      )
    }
  })

  test("keeps every Web UI lesson editable and diagnosable", () => {
    for (const id of webIds) {
      const lesson = lessonById(id)
      expect(lesson.source.trim()).not.toBe("")
      expect(lesson.exerciseSource.trim()).not.toBe("")
      expect(lesson.exerciseExpectedOutput.trim()).not.toBe("")
      expect(lesson.diagnosticSource.trim()).not.toBe("")
      expect(lesson.diagnosticOutput).toContain("error[")
      expect(lesson.format?.walkthrough.length).toBeGreaterThan(0)
      expect(lesson.format?.introduced.length).toBeGreaterThan(0)
      expect(lesson.format?.recap.length).toBeGreaterThan(0)
      if (!lesson.interactive) expect(lesson.expectedOutput.trim()).not.toBe("")
    }
  })

  test("makes the event and state path traceable", () => {
    expect(lessonById("web-click-action").source).toContain("onClick")
    expect(lessonById("web-input-action").source).toContain("event.value")
    expect(lessonById("web-change-action").source).toContain("event.checked")
    expect(lessonById("web-form-submit").source).toContain("onSubmit")

    const signal = lessonById("web-signal-preview")
    expect(signal.source).toContain("signals.map view state")
    expect(signal.source).toContain("signals.set 42 state")

    const runtime = lessonById("web-dom-run")
    expect(runtime.source).toContain('dom.query "#app"')
    expect(runtime.source).toContain("dom.run options target (handle state) content")

    const typed = lessonById("web-typed-action")
    expect(typed.source).toContain("match action")
    expect(typed.source).toContain("state: MutableSignal<String>")
  })

  test("covers media, forms, accessibility, and mobile-safe ownership", () => {
    const media = lessonById("web-link-image").source
    expect(media).toContain("html.img")
    expect(media).toContain("html.a")
    expect(media).toContain('target: "_blank"')
    expect(media).toContain('rel: "noopener"')

    const accessible = lessonById("web-accessibility-label").source
    expect(accessible).toContain('htmlFor: "draft"')
    expect(accessible).toContain('role: "status"')

    const feature = lessonById("web-feature-state").source
    expect(feature).toContain("onInput: draftAction")
    expect(feature).toContain("onChange: pinnedAction")
    expect(feature).toContain("onSubmit: Submitted")
    expect(feature).toContain("signals.map page state")
    expect(feature).toContain('className: "mx-auto max-w-xl p-4"')
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
