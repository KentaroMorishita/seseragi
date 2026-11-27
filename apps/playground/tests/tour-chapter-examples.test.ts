import { describe, expect, test } from "bun:test"
import curriculumJson from "../../../examples/tour/curriculum.json"
import { samples } from "../src/samples"
import {
  chapterExampleIds,
  chapterExamplesForLesson,
  isChapterEnd,
  tourChapterExampleMap,
  tourOpenInPlaygroundUrl,
} from "../src/tour/chapter-examples"
import { tourLessons } from "../src/tour/curriculum"

describe("Tour chapter examples and Playground handoff (#1350)", () => {
  test("maps every curriculum chapter to 1-2 existing Playground samples", () => {
    const sampleIds = new Set(samples.map(({ id }) => id))
    const chapters = curriculumJson.categories.flatMap(
      (category) => category.chapters
    )
    for (const chapter of chapters) {
      const ids = chapterExampleIds(chapter.id)
      expect(ids.length, chapter.id).toBeGreaterThanOrEqual(1)
      expect(ids.length, chapter.id).toBeLessThanOrEqual(2)
      for (const id of ids) {
        expect(sampleIds.has(id), `${chapter.id} -> ${id}`).toBe(true)
      }
    }
    const chapterIds = new Set(chapters.map(({ id }) => id))
    for (const mappedChapter of Object.keys(tourChapterExampleMap)) {
      expect(chapterIds.has(mappedChapter), mappedChapter).toBe(true)
    }
  })

  test("shows chapter examples only on the final lesson of a chapter", () => {
    expect(isChapterEnd(tourLessons, "01-hello-world")).toBe(false)
    expect(
      chapterExamplesForLesson(tourLessons, "01-hello-world", samples)
    ).toHaveLength(0)

    expect(isChapterEnd(tourLessons, "if-expression")).toBe(true)
    const endExamples = chapterExamplesForLesson(
      tourLessons,
      "if-expression",
      samples
    )
    expect(endExamples.map(({ id }) => id)).toEqual([
      "weather-advice",
      "fizzbuzz",
    ])
  })

  test("builds locale-aware Playground ?source= links and falls back when oversized", () => {
    const jaUrl = tourOpenInPlaygroundUrl(
      "https://seseragi.vercel.app",
      "ja",
      'pub effect fn main = println "hi"'
    )
    expect(jaUrl).toContain("https://seseragi.vercel.app/?source=")

    const enUrl = tourOpenInPlaygroundUrl(
      "https://seseragi.vercel.app",
      "en",
      'pub effect fn main = println "hi"'
    )
    expect(enUrl).toContain("https://seseragi.vercel.app/en/?source=")

    const oversized = tourOpenInPlaygroundUrl(
      "https://seseragi.vercel.app",
      "ja",
      "x".repeat(70 * 1024)
    )
    expect(oversized).toBe("https://seseragi.vercel.app/")
  })
})
