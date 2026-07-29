import { describe, expect, test } from "bun:test"
import { tourChapters, tourLessons } from "../src/tour/curriculum"
import {
  completeTourLesson,
  loadTourProgress,
  saveTourProgress,
  tourProgressStorageKey,
  visitTourLesson,
} from "../src/tour/progress"

class MemoryStorage {
  readonly values = new Map<string, string>()

  getItem(key: string): string | null {
    return this.values.get(key) ?? null
  }

  setItem(key: string, value: string): void {
    this.values.set(key, value)
  }
}

describe("Tour curriculum UI", () => {
  test("maps the curriculum to canonical lesson content and executable seeds", () => {
    expect(tourChapters).toHaveLength(3)
    expect(tourLessons).toHaveLength(14)
    expect(tourLessons.map(({ order }) => order)).toEqual(
      Array.from({ length: 14 }, (_, index) => index + 1)
    )
    for (const lesson of tourLessons) {
      expect(lesson.focus.length).toBeGreaterThanOrEqual(1)
      expect(lesson.focus.length).toBeLessThanOrEqual(2)
      expect(lesson.source.trim()).not.toBe("")
      expect(lesson.guide.trim()).not.toBe("")
    }
    for (const lesson of tourLessons) {
      expect(lesson.contentKind).toBe("canonical")
      expect(lesson.sourcePath).toBe(
        `examples/tour/lessons/${lesson.id}/main.ssrg`
      )
      if (lesson.interactive) {
        expect(lesson.id).toBe("14-integrated-app")
        expect(lesson.expectedOutput).toBe("")
      } else {
        expect(lesson.expectedOutput.trim()).not.toBe("")
      }
    }
    for (const lesson of tourLessons.slice(5)) {
      const previous = tourLessons[lesson.order - 2]
      if (previous === undefined) throw new Error("missing previous lesson")
      expect(lesson.prerequisites).toEqual([previous.id])
    }
    expect(tourLessons[12]).toMatchObject({ outputMode: "html" })
    expect(tourLessons[13]).toMatchObject({
      capabilities: ["dom"],
      outputMode: "html",
      interactive: true,
    })
  })

  test("persists the current lesson and unique completion state", () => {
    const storage = new MemoryStorage()
    const lessonIds = tourLessons.map(({ id }) => id)
    let progress = loadTourProgress(storage, lessonIds)
    expect(progress.currentLessonId).toBe("01-hello-world")

    progress = visitTourLesson(progress, "03-function-definitions")
    progress = completeTourLesson(progress, "03-function-definitions")
    progress = completeTourLesson(progress, "03-function-definitions")
    saveTourProgress(storage, progress)

    expect(storage.values.has(tourProgressStorageKey)).toBe(true)
    expect(loadTourProgress(storage, lessonIds)).toEqual({
      currentLessonId: "03-function-definitions",
      completedLessonIds: ["03-function-definitions"],
    })
  })

  test("lets a valid route override stored progress and drops stale ids", () => {
    const storage = new MemoryStorage()
    const lessonIds = tourLessons.map(({ id }) => id)
    storage.setItem(
      tourProgressStorageKey,
      JSON.stringify({
        currentLessonId: "missing",
        completedLessonIds: ["missing", "01-hello-world"],
      })
    )

    expect(loadTourProgress(storage, lessonIds, "14-integrated-app")).toEqual({
      currentLessonId: "14-integrated-app",
      completedLessonIds: ["01-hello-world"],
    })
  })

  test("provides a separate accessible page backed by shared tooling", async () => {
    const root = new URL("..", import.meta.url)
    const playgroundHtml = await Bun.file(new URL("index.html", root)).text()
    const tourHtml = await Bun.file(new URL("tour/index.html", root)).text()
    const tourMain = await Bun.file(new URL("src/tour/main.ts", root)).text()
    const vite = await Bun.file(new URL("vite.config.ts", root)).text()

    expect(playgroundHtml).toContain('href="./tour/"')
    expect(playgroundHtml).toContain("Tourを始める")
    expect(tourHtml).toContain('href="../"')
    expect(tourHtml).toContain('id="tour-chapters"')
    expect(tourHtml).toContain('id="tour-editor"')
    expect(tourHtml).toContain('id="tour-run-button"')
    expect(tourHtml).toContain('id="tour-reset-button"')
    expect(tourHtml).toContain('id="tour-format-button"')
    expect(tourHtml).toContain('id="tour-guide"')
    expect(tourHtml).toContain('id="tour-input-section"')
    expect(tourHtml).toContain('id="tour-output"')
    expect(tourHtml).toContain('id="tour-html-preview"')
    expect(tourMain).toContain("compileSingleFile(source)")
    expect(tourMain).toContain("formatSingleFile(requestedSource)")
    expect(tourMain).toContain("startGeneratedModule(")
    expect(tourMain).toContain("currentLesson.guide")
    expect(tourMain).not.toContain("compile_single_file(")
    expect(vite).toContain("tour:")
    expect(vite).toContain("tour/index.html")
  })

  test("uses the shared full-screen control on both pages", async () => {
    const root = new URL("..", import.meta.url)
    const playgroundHtml = await Bun.file(new URL("index.html", root)).text()
    const tourHtml = await Bun.file(new URL("tour/index.html", root)).text()
    const playgroundMain = await Bun.file(new URL("src/main.ts", root)).text()
    const tourMain = await Bun.file(new URL("src/tour/main.ts", root)).text()

    expect(playgroundHtml).toContain('id="fullscreen-preview-button"')
    expect(tourHtml).toContain('id="tour-fullscreen-button"')
    expect(playgroundMain).toContain("connectPreviewFullscreen(")
    expect(tourMain).toContain("connectPreviewFullscreen(")
  })

  test("keeps the Tour usable on narrow and compact landscape screens", async () => {
    const styles = await Bun.file(
      new URL("../src/tour/styles.css", import.meta.url)
    ).text()

    expect(styles).toContain(
      "@media (max-width: 760px), (max-width: 960px) and (max-height: 520px)"
    )
    expect(styles).toContain(
      "@media (orientation: landscape) and (max-width: 960px) and (max-height: 520px)"
    )
    expect(styles).toMatch(
      /\.tour-navigation\[data-mobile-open="true"\] \{\s*transform: translateX\(0\);/
    )
    expect(styles).toMatch(
      /\.tour-input-section textarea,[\s\S]*?font-size: 16px;/
    )
  })
})
