import { describe, expect, test } from "bun:test"
import { tourLessons } from "../src/tour/curriculum"

const applicationIds = [
  "applications-console-data",
  "14-integrated-app",
  "design-learning-map",
] as const

describe("Tour application capstones", () => {
  test("consolidates console capstone, integrated web app, and next steps sequentially", () => {
    const lessons = tourLessons.slice(32, 32 + applicationIds.length)

    expect(lessons.map(({ id }) => id)).toEqual([...applicationIds])
    expect(lessons.every(({ deliveryIssue }) => deliveryIssue === 1348)).toBe(
      true
    )
    expect(lessons[0]!.prerequisites).toEqual(["web-dom-run"])
    expect(lessons[1]!.prerequisites).toEqual(["applications-console-data"])
    expect(lessons[2]!.prerequisites).toEqual(["14-integrated-app"])
    expect(lessons[0]!.format?.next.lessonId).toBe("14-integrated-app")
    expect(lessons[2]!.format?.next.lessonId).toBe("abstraction-functor-map")
  })

  test("combines data, validation, and Effect boundary in the console capstone", () => {
    const consoleLesson = lessonById("applications-console-data")
    expect(consoleLesson.source).toContain("let sales: Array<Sale>")
    expect(consoleLesson.source).toContain("Either<String, Sale>")
    expect(consoleLesson.source).toContain("|> println")
    expect(consoleLesson.expectedOutput).toContain("Book: 2400")
    expect(consoleLesson.expectedOutput).toContain("error: Cancelled")
    expect(consoleLesson.exerciseSource.trim()).not.toBe("")
    expect(consoleLesson.diagnosticOutput).toContain("error[")

    const recap = consoleLesson.format?.recap.join("\n")
    expect(recap).toContain("Tour")
    expect(recap).toContain("Recipe")
    expect(recap).toContain("Showcase")
  })

  test("ships the interactive dom.app capstone and next-step guide", () => {
    const integrated = lessonById("14-integrated-app")
    expect(integrated.interactive).toBe(true)
    expect(integrated.source).toContain("dom.app")
    expect(integrated.source).toContain("update")
    expect(integrated.source).toContain("view")

    const nextSteps = lessonById("design-learning-map")
    expect(nextSteps.source).toContain("Discover")
    expect(nextSteps.source).toContain("Docs")
    expect(nextSteps.source).toContain("SideTrips")
    expect(nextSteps.expectedOutput).toContain("Discover:")
    expect(nextSteps.exerciseSource.trim()).not.toBe("")
    expect(nextSteps.diagnosticOutput).toContain("error[")
  })
})

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
