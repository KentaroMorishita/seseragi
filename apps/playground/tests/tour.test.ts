import { describe, expect, test } from "bun:test"
import {
  tourCategories,
  tourChapters,
  tourLessons,
} from "../src/tour/curriculum"
import {
  completeTourLesson,
  legacyTourProgressStorageKey,
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
    expect(tourCategories.length).toBeGreaterThan(0)
    expect(tourChapters.length).toBeGreaterThanOrEqual(tourCategories.length)
    expect(tourLessons.map(({ position }) => position)).toEqual(
      tourLessons.map((_, index) => index + 1)
    )
    const knownCategories = new Set(tourCategories.map(({ id }) => id))
    const knownChapters = new Set(tourChapters.map(({ id }) => id))
    const knownLessons = new Set(tourLessons.map(({ id }) => id))
    for (const lesson of tourLessons) {
      expect(lesson.focus.length).toBeGreaterThanOrEqual(1)
      expect(lesson.focus.length).toBeLessThanOrEqual(2)
      expect(lesson.goal.trim()).not.toBe("")
      expect(lesson.summary.trim()).not.toBe("")
      expect(lesson.introducedSurfaces.length).toBeGreaterThan(0)
      expect(knownCategories.has(lesson.categoryId)).toBe(true)
      expect(knownChapters.has(lesson.chapterId)).toBe(true)
      for (const prerequisite of lesson.prerequisites) {
        expect(knownLessons.has(prerequisite)).toBe(true)
        expect(
          tourLessons.findIndex(({ id }) => id === prerequisite)
        ).toBeLessThan(lesson.position - 1)
      }
      expect(lesson.source.trim()).not.toBe("")
      expect(lesson.sourcePath).toBe(
        `examples/tour/lessons/${lesson.id}/main.ssrg`
      )
      if (lesson.format === undefined) {
        expect(lesson.guide.trim()).not.toBe("")
      } else {
        expect(lesson.guide).toBe("")
        expect(lesson.format.walkthrough.length).toBeGreaterThan(0)
        expect(lesson.format.introduced.length).toBeGreaterThan(0)
        expect(lesson.format.recap.length).toBeGreaterThan(0)
        expect(lesson.exerciseSource.trim()).not.toBe("")
        expect(lesson.exerciseExpectedOutput.trim()).not.toBe("")
        expect(lesson.diagnosticSource.trim()).not.toBe("")
        expect(lesson.diagnosticOutput).toContain("error[")
      }
      if (lesson.interactive) {
        expect(lesson.capabilities).toContain("dom")
        expect(lesson.expectedOutput).toBe("")
        expect(lesson.expectedFailure).toBe("")
      } else {
        expect(
          lesson.expectedOutput.trim() !== "" ||
            lesson.expectedFailure.trim() !== ""
        ).toBe(true)
      }
    }
    expect(findLesson("web-html-value")).toMatchObject({
      outputMode: "html",
    })
    expect(findLesson("01-hello-world")).toMatchObject({
      format: {
        exercise: { reset: "restore-lesson-source" },
        next: { lessonId: "02-values-and-bindings" },
      },
    })
    expect(findLesson("14-integrated-app")).toMatchObject({
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
    expect(JSON.parse(storage.values.get(tourProgressStorageKey)!)).toEqual({
      schema: 2,
      currentLessonId: "03-function-definitions",
      completedLessonIds: ["03-function-definitions"],
    })
    expect(loadTourProgress(storage, lessonIds)).toEqual({
      currentLessonId: "03-function-definitions",
      completedLessonIds: ["03-function-definitions"],
    })
  })

  test("migrates v1 progress by stable id and survives manifest reordering", () => {
    const storage = new MemoryStorage()
    storage.setItem(
      legacyTourProgressStorageKey,
      JSON.stringify({
        currentLessonId: "03-function-definitions",
        completedLessonIds: [
          "01-hello-world",
          "03-function-definitions",
          "missing",
        ],
      })
    )
    const reorderedIds = tourLessons.map(({ id }) => id).reverse()

    expect(loadTourProgress(storage, reorderedIds)).toEqual({
      currentLessonId: "03-function-definitions",
      completedLessonIds: ["01-hello-world", "03-function-definitions"],
    })
    expect(storage.values.has(tourProgressStorageKey)).toBe(true)
  })

  test("lets a valid route override stored progress and drops stale ids", () => {
    const storage = new MemoryStorage()
    const lessonIds = tourLessons.map(({ id }) => id)
    storage.setItem(
      tourProgressStorageKey,
      JSON.stringify({
        schema: 2,
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

    expect(playgroundHtml).toContain('class="ui-view-switch pg-views"')
    expect(playgroundHtml).toContain('<a href="./tour/">')
    expect(tourHtml).toContain('class="ui-view-switch pg-views"')
    expect(tourHtml).toContain('<a href="./" aria-current="page">')
    for (const id of [
      "tour-chapters",
      "tour-editor",
      "tour-run-button",
      "tour-reset-button",
      "tour-format-button",
      "tour-search-button",
      "search-dialog",
      "tour-try-steps",
      "tour-guide",
      "tour-walkthrough",
      "tour-exercise-button",
      "tour-diagnostic-output",
      "tour-recap-list",
      "tour-input-section",
      "tour-output",
      "tour-html-preview",
      "tour-next-button",
    ]) {
      expect(tourHtml).toContain(`id="${id}"`)
    }
    expect(tourHtml).toContain('sandbox="allow-scripts allow-forms"')
    expect(tourHtml).toContain("<summary>Going further</summary>")
    expect(tourMain).toContain("compileSingleFile(requestedSource)")
    expect(tourMain).toContain(
      "formatSingleFile(requestedSource, { lineWidth })"
    )
    expect(tourMain).toContain("startIsolatedProject(")
    expect(tourMain).not.toContain("startGeneratedModule(")
    expect(tourMain).toContain("currentLesson.guide")
    expect(tourMain).toContain("format.walkthrough")
    expect(tourMain).toContain("sourceExcerpt(currentLesson.source")
    expect(tourMain).toContain("highlightSeseragi(")
    expect(tourMain).toContain('code.className = "seseragi-highlight"')
    expect(tourMain).toContain("currentLesson.exerciseSource")
    expect(tourMain).toContain("currentLesson.diagnosticSource")
    expect(tourMain).toContain("setEditorLinePins(")
    expect(tourMain).toContain("markTryProgress(")
    expect(tourMain).toContain("connectSearchPalette({")
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
    const controller = await Bun.file(
      new URL("src/ui/preview-fullscreen.ts", root)
    ).text()
    expect(playgroundHtml).toContain('id="fullscreen-preview-button"')
    expect(tourHtml).toContain('id="tour-fullscreen-button"')
    expect(playgroundHtml).toContain(
      'class="output-section pg-output preview-fullscreen-surface"'
    )
    expect(tourHtml).toContain(
      'class="ui-card tour-output-card preview-fullscreen-surface"'
    )
    expect(playgroundMain).toContain("connectPreviewFullscreen(")
    expect(tourMain).toContain("connectPreviewFullscreen(")
    expect(controller).toContain('useFallback("unsupported")')
    expect(controller).toContain('useFallback("rejected")')
    expect(controller).toContain('event.key !== "Escape"')
    expect(controller).not.toContain(".catch(() => undefined)")
    const playgroundStyles = await Bun.file(
      new URL("src/ui/playground.css", root)
    ).text()
    for (const css of [playgroundStyles]) {
      expect(css).toContain("body.preview-fullscreen-fallback-open")
      expect(css).toContain(
        '.preview-fullscreen-surface[data-preview-fullscreen="fallback"]'
      )
      expect(css).toContain("env(safe-area-inset-top)")
    }
  })

  test("keeps the Tour usable on narrow and compact landscape screens", async () => {
    const root = new URL("..", import.meta.url)
    const tourHtml = await Bun.file(new URL("tour/index.html", root)).text()
    const tourMain = await Bun.file(new URL("src/tour/main.ts", root)).text()
    const styles = await Bun.file(new URL("src/tour/tour.css", root)).text()

    expect(tourHtml).toContain('id="tour-menu-close-button"')
    expect(tourHtml).toContain('aria-label="Close contents"')
    expect(tourHtml).toContain('id="tour-lesson-title" tabindex="-1"')
    // 狭い画面では、下の Lesson / Code でレッスンとコードを切り替える
    expect(tourHtml).toContain('id="tour-lesson-focus-button"')
    expect(tourHtml).toContain('id="tour-code-focus-button"')
    expect(styles).toContain(
      "@media (max-width: 760px), (max-width: 960px) and (max-height: 520px)"
    )
    expect(styles).toContain(
      '.tour-workspace[data-mobile-panel="lesson"] .tour-lab'
    )
    expect(styles).toContain(
      '.tour-workspace[data-mobile-panel="code"] .tour-lesson'
    )
    expect(styles).toMatch(/\.tour-run \{[\s\S]*?position: fixed;/)
    expect(tourMain).toContain("showMobilePanel(")
    expect(tourMain).toMatch(
      /function selectCanonicalSourceRange\([\s\S]*?showMobilePanel\("code"\)[\s\S]*?editor\.focus\(\)/
    )
    expect(tourMain).toMatch(
      /function loadSourceVariant\([\s\S]*?showMobilePanel\("code"\)[\s\S]*?editor\.focus\(\)/
    )
    expect(tourMain).toContain('event.key === "Escape"')
    expect(tourMain).toContain('event.key !== "Tab"')
    expect(tourMain).toContain('navigation.setAttribute("aria-modal", "true")')
    expect(tourMain).toContain("navigation.inert = !open")
    expect(tourMain).toContain("setNavigationBackgroundInert(open)")
    expect(tourMain).toContain("menuButton.focus({ preventScroll: true })")
    expect(tourMain).toContain('button.setAttribute("aria-expanded"')
    expect(tourMain).toContain('event.key !== "ArrowLeft"')
    expect(tourMain).toContain('button.setAttribute("aria-current", "step")')
    expect(tourMain).toContain('"未着手"')
    expect(tourMain).toContain('"完了"')
  })
  test("keeps hierarchy, progress and routes understandable as the Tour grows", async () => {
    const root = new URL("..", import.meta.url)
    const tourMain = await Bun.file(new URL("src/tour/main.ts", root)).text()
    const styles = await Bun.file(
      new URL("../src/tour/tour.css", import.meta.url)
    ).text()

    expect(tourMain).toContain("buildTourNavigationModel(")
    expect(tourMain).toContain("category.progress")
    expect(tourMain).toContain("chapter.progress")
    expect(tourMain).toContain('"tour-category-overview"')
    expect(tourMain).toContain('"tour-category-resume"')
    expect(tourMain).toContain("neighbor.crossesCategory")
    expect(tourMain).toContain('window.addEventListener("popstate"')
    expect(tourMain).toContain('history[mode === "push" ? "pushState"')
    // コースマップは章のカードを並べ、今いる部を縁取る
    expect(styles).toContain("repeat(auto-fill, minmax(300px, 1fr))")
    expect(styles).toContain('.tour-category:has([aria-current="step"])')
  })
})

function findLesson(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
