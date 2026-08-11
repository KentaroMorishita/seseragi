import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const consoleIds = [
  "applications-console-data",
  "applications-console-transform",
  "applications-console-fallibility",
  "applications-console-output",
] as const

const webIds = [
  "applications-web-static",
  "applications-web-component",
  "applications-web-signal",
  "applications-web-form-event",
  "applications-web-action",
  "applications-web-validation",
  "applications-web-feature-ownership",
] as const

describe("Tour application capstones", () => {
  test("builds one console report through four runnable differences", () => {
    const lessons = consoleIds.map(lessonById)

    expect(lessons.map(({ deliveryIssue }) => deliveryIssue)).toEqual([
      181, 181, 181, 181,
    ])
    for (const [index, lesson] of lessons.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "web-feature-state" : consoleIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        consoleIds[index + 1] ?? "applications-web-static"
      )
      expect(lesson.source.trim()).not.toBe("")
      expect(lesson.expectedOutput.trim()).not.toBe("")
      expect(lesson.exerciseSource.trim()).not.toBe("")
      expect(lesson.diagnosticOutput).toContain("error[")
    }
  })

  test("adds data, transform, fallibility, then one Effect boundary", () => {
    expect(lessonById("applications-console-data").source).toContain(
      "let sales: Array<Sale>"
    )
    expect(lessonById("applications-console-transform").source).toContain(
      "|> arrays.filter completed"
    )
    const fallibility = lessonById("applications-console-fallibility")
    expect(fallibility.source).toContain("Maybe<String>")
    expect(fallibility.source).toContain("Either<String, Sale>")
    const output = lessonById("applications-console-output")
    expect(output.source).toContain("fn report values: Array<Sale> -> String")
    expect(output.source).toContain("report sales |> println")
  })

  test("documents the Tour, Recipe, and Showcase boundary", () => {
    const recap = lessonById("applications-console-data").format?.recap.join(
      "\n"
    )
    expect(recap).toContain("Tour")
    expect(recap).toContain("Recipe")
    expect(recap).toContain("Showcase")
  })

  test("builds one Web UI through seven runnable differences", () => {
    const lessons = webIds.map(lessonById)

    expect(lessons.every(({ deliveryIssue }) => deliveryIssue === 181)).toBe(
      true
    )
    for (const [index, lesson] of lessons.entries()) {
      expect(lesson.prerequisites).toEqual([
        index === 0 ? "applications-console-output" : webIds[index - 1]!,
      ])
      expect(lesson.format?.next.lessonId).toBe(
        webIds[index + 1] ?? "14-integrated-app"
      )
      expect(lesson.source.trim()).not.toBe("")
      expect(lesson.expectedOutput.trim()).not.toBe("")
      expect(lesson.exerciseSource.trim()).not.toBe("")
      expect(lesson.diagnosticOutput).toContain("error[")
    }
  })

  test("adds static, component, Signal, event, Action, validation, and ownership", () => {
    expect(lessonById(webIds[0]).source).toContain(
      "let page: html.Html<Action>"
    )
    expect(lessonById(webIds[1]).source).toContain("fn planForm")
    expect(lessonById(webIds[2]).source).toContain("signals.map page state")
    expect(lessonById(webIds[3]).source).toContain("onInput: draftAction")
    expect(lessonById(webIds[4]).source).toContain("fn handle state")
    expect(lessonById(webIds[5]).source).toContain("Either<String, String>")
    const ownership = lessonById(webIds[6])
    expect(ownership.source).toContain("effect fn createFeature")
    expect(ownership.source).toContain("page <$> personal <*> release")
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
