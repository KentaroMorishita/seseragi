import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const coreWebIds = [
  "web-html-value",
  "web-component-props",
  "web-typed-action",
  "web-form-submit",
  "web-dom-run",
] as const

const sideTripWebIds = [
  "web-link-image",
  "web-accessibility-label",
  "web-feature-state",
] as const

describe("Tour Web UI curriculum", () => {
  test("stages static HTML through dom.run in core and keeps details in side trips", () => {
    const core = tourLessons.slice(27, 27 + coreWebIds.length)
    expect(core.map(({ id }) => id)).toEqual([...coreWebIds])
    expect(core.every(({ deliveryIssue }) => deliveryIssue === 1348)).toBe(true)
    for (const [index, lesson] of core.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "signals-handler-boundary" : coreWebIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        coreWebIds[index + 1] ?? "applications-console-data"
      )
    }

    const sideTrips = tourLessons.slice(47, 47 + sideTripWebIds.length)
    expect(sideTrips.map(({ id }) => id)).toEqual([...sideTripWebIds])
    expect(sideTrips.every(({ deliveryIssue }) => deliveryIssue === 1348)).toBe(
      true
    )
    for (const [index, lesson] of sideTrips.entries()) {
      expect(lesson.format?.next.lessonId).toBe(
        sideTripWebIds[index + 1] ?? "list-values"
      )
    }
  })

  test("keeps every Web UI lesson editable and diagnosable", () => {
    for (const id of [...coreWebIds, ...sideTripWebIds]) {
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
    const staticHtml = lessonById("web-html-value").source
    expect(staticHtml).toContain("html.style")
    expect(staticHtml).toContain("html.main")

    expect(lessonById("web-component-props").source).toContain("fn card")

    const typed = lessonById("web-typed-action")
    expect(typed.source).toContain("onClick: Cleared")
    expect(typed.source).toContain("match action")
    expect(typed.source).toContain("state: MutableSignal<String>")

    const form = lessonById("web-form-submit").source
    expect(form).toContain("event.value")
    expect(form).toContain("event.checked")
    expect(form).toContain("onSubmit: Submitted")

    const runtime = lessonById("web-dom-run")
    expect(runtime.source).toContain("signals.map view state")
    expect(runtime.source).toContain('dom.query "#app"')
    expect(runtime.source).toContain(
      "dom.run options target (handle state) content"
    )
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
    expect(feature).toContain('class: "mx-auto max-w-xl p-4"')
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
