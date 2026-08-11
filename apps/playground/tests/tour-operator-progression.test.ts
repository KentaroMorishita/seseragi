import { describe, expect, test } from "bun:test"
import curriculum from "../../../examples/tour/curriculum.json"
import { tourLessons } from "../src/tour/curriculum"

const operatorCoverage = [
  "collection-functor-operator",
  "maybe-functor-operator",
  "maybe-applicative-operator",
  "maybe-monad-operator",
  "either-functor-operator",
  "effect-monad-correspondence",
] as const

describe("Tour concrete operator progression", () => {
  test("introduces operators in concrete chapters before abstraction", () => {
    expect(firstLessonContaining("<$>")?.id).toBe("collection-map")
    expect(firstLessonContaining("<*>")?.id).toBe("maybe-combine")
    expect(firstLessonContaining(">>=")?.id).toBe("maybe-short-circuit")

    const abstractionOrder = lessonById("abstraction-functor-map").order
    for (const id of [
      "collection-map",
      "maybe-map",
      "maybe-combine",
      "maybe-short-circuit",
      "either-map-error",
      "10-effects-and-do",
    ]) {
      expect(lessonById(id).order).toBeLessThan(abstractionOrder)
    }
  })

  test("requires type-specific coverage instead of one abstract occurrence", () => {
    for (const topic of operatorCoverage) {
      expect(curriculum.requiredTopics).toContain(topic)
      expect(
        tourLessons.filter(({ introducedSurfaces }) =>
          introducedSurfaces.includes(topic)
        )
      ).toHaveLength(1)
    }
  })

  test("records every standard instance family and intentional Tour exits", async () => {
    const audit = await Bun.file(
      new URL(
        "../../../examples/tour/standard-instance-coverage.md",
        import.meta.url
      )
    ).text()

    for (const type of [
      "`Maybe`",
      "`Either<E, _>`",
      "`Array`",
      "`List`",
      "`NonEmptyList`",
      "`Effect` / `Task`",
      "`Signal`",
      "`Stream<R, E, _>`",
      "`Validation<E, _>`",
    ]) {
      expect(audit).toContain(`| ${type} |`)
    }
    expect(audit).toContain("Arrayだけのcoverageを禁止")
    expect(audit).toContain("現行必修Tour外")
  })
})

function firstLessonContaining(operator: string) {
  return tourLessons.find(({ source }) => source.includes(operator))
}

function lessonById(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
