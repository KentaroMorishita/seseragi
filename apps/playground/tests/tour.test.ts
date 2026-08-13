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
        next: { lessonId: "program-entry" },
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

    expect(playgroundHtml).toContain('id="surface-switcher-menu"')
    expect(playgroundHtml).toContain('href="./tour/" role="menuitem"')
    expect(tourHtml).toContain('href="../"')
    expect(tourHtml).toContain('id="tour-surface-switcher-button"')
    expect(tourHtml).toContain('id="tour-chapters"')
    expect(tourHtml).toContain('id="tour-editor"')
    expect(tourHtml).toContain('id="tour-run-button"')
    expect(tourHtml).toContain('id="tour-reset-button"')
    expect(tourHtml).toContain('id="tour-format-button"')
    expect(tourHtml).toContain('id="tour-settings-button"')
    expect(tourHtml).toContain('id="tour-guide"')
    expect(tourHtml).toContain('id="tour-walkthrough"')
    expect(tourHtml).toContain('id="tour-exercise-button"')
    expect(tourHtml).toContain('id="tour-diagnostic-output"')
    expect(tourHtml).toContain('id="tour-recap-list"')
    expect(tourHtml).toContain('id="tour-input-section"')
    expect(tourHtml).toContain('id="tour-output"')
    expect(tourHtml).toContain('id="tour-html-preview"')
    expect(tourHtml).toContain(
      'sandbox="allow-forms allow-popups allow-popups-to-escape-sandbox allow-same-origin allow-scripts"'
    )
    expect(tourMain).toContain("compileSingleFile(source)")
    expect(tourMain).toContain(
      "formatSingleFile(requestedSource, { lineWidth })"
    )
    expect(tourMain).toContain("startGeneratedModule(")
    expect(tourMain).toContain("currentLesson.guide")
    expect(tourMain).toContain("format.walkthrough")
    expect(tourMain).toContain("sourceExcerpt(currentLesson.source")
    expect(tourMain).toContain("highlightSeseragi(")
    expect(tourMain).toContain('code.className = "seseragi-highlight"')
    expect(tourMain).toContain("currentLesson.exerciseSource")
    expect(tourMain).toContain("currentLesson.diagnosticSource")
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
    const styles = await Bun.file(new URL("src/styles.css", root)).text()

    expect(playgroundHtml).toContain('id="fullscreen-preview-button"')
    expect(tourHtml).toContain('id="tour-fullscreen-button"')
    expect(playgroundHtml).toContain(
      'class="output-section preview-fullscreen-surface"'
    )
    expect(tourHtml).toContain(
      'class="tour-output-section preview-fullscreen-surface"'
    )
    expect(playgroundMain).toContain("connectPreviewFullscreen(")
    expect(tourMain).toContain("connectPreviewFullscreen(")
    expect(controller).toContain('useFallback("unsupported")')
    expect(controller).toContain('useFallback("rejected")')
    expect(controller).toContain('event.key !== "Escape"')
    expect(controller).not.toContain(".catch(() => undefined)")
    expect(styles).toContain("body.preview-fullscreen-fallback-open")
    expect(styles).toContain(
      '.preview-fullscreen-surface[data-preview-fullscreen="fallback"]'
    )
    expect(styles).toContain("env(safe-area-inset-top)")
  })

  test("keeps the Tour usable on narrow and compact landscape screens", async () => {
    const root = new URL("..", import.meta.url)
    const tourHtml = await Bun.file(new URL("tour/index.html", root)).text()
    const tourMain = await Bun.file(new URL("src/tour/main.ts", root)).text()
    const styles = await Bun.file(
      new URL("../src/tour/styles.css", import.meta.url)
    ).text()

    expect(tourHtml).toContain('id="tour-menu-close-button"')
    expect(tourHtml).toContain(
      'aria-label="Category / chapter / lesson一覧を閉じる"'
    )
    expect(tourHtml).toContain('id="tour-lesson-title" tabindex="-1"')
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
      /\.tour-navigation \{[\s\S]*?inset: 0;[\s\S]*?width: 100vw;[\s\S]*?height: 100dvh;/
    )
    expect(styles).toMatch(
      /\.tour-lesson-link-title \{[\s\S]*?white-space: normal;/
    )
    expect(styles).toContain("calc(var(--safe-area-top) + 14px)")
    expect(styles).toContain(".tour-body.tour-navigation-sheet-open")
    expect(styles).not.toContain("top: 109px")
    expect(styles).not.toContain(
      "scroll-margin-top: calc(var(--safe-area-top) + 154px)"
    )
    expect(styles).toContain(
      ".tour-topbar {\n    position: relative;\n    top: auto;"
    )
    expect(styles).toContain(
      ".tour-local-navigation {\n    position: sticky;\n    z-index: 30;\n    top: calc(var(--safe-area-top) + 54px);"
    )
    expect(styles).toMatch(
      /@media \(max-width: 1180px\)[\s\S]*?\.tour-local-navigation \.tour-menu-button \{[\s\S]*?display: block;/
    )
    expect(
      styles.match(/scroll-margin-top: calc\(var\(--safe-area-top\) \+ 98px\)/g)
    ).toHaveLength(2)
    expect(styles).toMatch(
      /\.tour-input-section textarea,[\s\S]*?font-size: 16px;/
    )
    expect(styles).toMatch(
      /\.tour-walkthrough-card pre,[\s\S]*?overflow-wrap: anywhere;/
    )
    expect(styles).toMatch(/\.tour-section-actions button,[\s\S]*?width: 100%;/)
    expect(tourMain).toContain('event.key === "Escape"')
    expect(tourMain).toContain('event.key !== "Tab"')
    expect(tourMain).toContain('navigation.setAttribute("aria-modal", "true")')
    expect(tourMain).toContain("navigation.inert = !open")
    expect(tourMain).toContain("setNavigationBackgroundInert(open)")
    expect(tourMain).toContain(
      "document.body.scrollTop = navigationBackgroundScrollTop"
    )
    expect(tourMain).toContain("menuButton.focus({ preventScroll: true })")
    expect(tourMain).toContain('lesson.scrollIntoView({ block: "start" })')
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
      new URL("../src/tour/styles.css", import.meta.url)
    ).text()

    expect(tourMain).toContain("buildTourNavigationModel(")
    expect(tourMain).toContain("category.progress")
    expect(tourMain).toContain("chapter.progress")
    expect(tourMain).toContain('"tour-category-overview"')
    expect(tourMain).toContain('"tour-category-resume"')
    expect(tourMain).toContain("neighbor.crossesCategory")
    expect(tourMain).toContain('window.addEventListener("popstate"')
    expect(tourMain).toContain('history[mode === "push" ? "pushState"')
    expect(styles).toContain("clamp(240px, 20vw, 310px)")
    expect(styles).toMatch(
      /\.tour-lesson-link-title \{[\s\S]*?white-space: normal;/
    )
  })
})

function findLesson(id: string) {
  const lesson = tourLessons.find((candidate) => candidate.id === id)
  if (lesson === undefined) throw new Error(`missing Tour lesson ${id}`)
  return lesson
}
