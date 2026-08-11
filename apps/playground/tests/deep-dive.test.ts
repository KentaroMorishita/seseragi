import { describe, expect, test } from "bun:test"
import {
  type DeepDiveArticleContent,
  type DeepDiveCatalog,
  type LoadedDeepDiveArticle,
  loadValidatedDeepDive,
  validateDeepDive,
} from "../../../scripts/deep-dive"
import { deepDiveArticles, deepDiveCategories } from "../src/deep-dive/catalog"
import { buildDeepDiveNavigation } from "../src/deep-dive/navigation"
import {
  completeDeepDiveArticle,
  deepDiveProgressStorageKey,
  loadDeepDiveProgress,
  saveDeepDiveProgress,
} from "../src/deep-dive/progress"
import {
  deepDiveArticleIdFromUrl,
  deepDiveArticleUrl,
} from "../src/deep-dive/route"

class MemoryStorage {
  readonly values = new Map<string, string>()

  getItem(key: string): string | null {
    return this.values.get(key) ?? null
  }

  setItem(key: string, value: string): void {
    this.values.set(key, value)
  }
}

describe("Deep Dive foundation", () => {
  test("loads the canonical category, chapter and article hierarchy", async () => {
    const repositoryRoot = new URL("../../..", import.meta.url).pathname
    const loaded = await loadValidatedDeepDive(repositoryRoot)

    expect(loaded.catalog.categories).toHaveLength(3)
    expect(loaded.articles.map(({ content }) => content.id)).toEqual(
      deepDiveArticles.map(({ id }) => id)
    )
    expect(deepDiveArticles).toHaveLength(5)
    expect(deepDiveArticles[0]).toMatchObject({
      categoryId: "orientation",
      chapterId: "orientation-path",
      position: 1,
      tourPrerequisites: ["abstraction-instance-selection"],
    })
    for (const article of loaded.articles) {
      expect(article.source.trim()).not.toBe("")
      expect(article.expectedOutput.trim()).not.toBe("")
      expect(article.diagnosticSource.trim()).not.toBe("")
      expect(article.diagnosticOutput).toContain("error[")
      expect(article.content.recap.length).toBeGreaterThan(0)
    }
  })

  test("derives independent article progress from data", () => {
    const model = buildDeepDiveNavigation(
      deepDiveCategories,
      deepDiveArticles,
      deepDiveArticles[0]!.id,
      [deepDiveArticles[0]!.id]
    )

    expect(model.progress).toEqual({ completed: 1, total: 5 })
    expect(model.categories[0]!.chapters[0]!.articles[0]!.state).toBe("current")

    const storage = new MemoryStorage()
    let progress = loadDeepDiveProgress(storage, [deepDiveArticles[0]!.id])
    progress = completeDeepDiveArticle(progress, deepDiveArticles[0]!.id)
    saveDeepDiveProgress(storage, progress)
    expect(storage.values.has(deepDiveProgressStorageKey)).toBe(true)
    expect(loadDeepDiveProgress(storage, [deepDiveArticles[0]!.id])).toEqual(
      progress
    )
    expect(deepDiveProgressStorageKey).not.toContain("tour")
  })

  test("keeps direct URL, reload, back and forward on stable article ids", () => {
    const articleIds = ["article-1", "article-2", "article-3"]
    const direct = "https://example.test/deep-dive/?article=article-2"
    expect(deepDiveArticleIdFromUrl(direct, articleIds)).toBe("article-2")
    expect(deepDiveArticleIdFromUrl(direct, articleIds)).toBe("article-2")

    const history = [
      direct,
      deepDiveArticleUrl(direct, "article-3"),
      deepDiveArticleUrl(direct, "article-1"),
    ]
    expect(deepDiveArticleIdFromUrl(history[1]!, articleIds)).toBe("article-3")
    expect(deepDiveArticleIdFromUrl(history[0]!, articleIds)).toBe("article-2")
    expect(deepDiveArticleIdFromUrl(history[1]!, articleIds)).toBe("article-3")
    expect(
      deepDiveArticleIdFromUrl(
        "https://example.test/deep-dive/?article=missing",
        articleIds
      )
    ).toBe("article-1")
  })

  test("scales navigation to dozens of articles without code changes", () => {
    const categories = Array.from({ length: 5 }, (_, categoryIndex) => ({
      id: `category-${categoryIndex + 1}`,
      order: categoryIndex + 1,
      title: `Category ${categoryIndex + 1}`,
      summary: "Summary",
      chapters: Array.from({ length: 3 }, (_, chapterIndex) => ({
        id: `category-${categoryIndex + 1}-chapter-${chapterIndex + 1}`,
        order: chapterIndex + 1,
        title: `Chapter ${chapterIndex + 1}`,
        summary: "Summary",
        articles: [],
      })),
    }))
    const articles = categories.flatMap((category) =>
      category.chapters.flatMap((chapter, chapterIndex) =>
        Array.from({ length: 4 }, (_, articleIndex) => {
          const position =
            (category.order - 1) * 12 + chapterIndex * 4 + articleIndex + 1
          return {
            id: `article-${position}`,
            order: articleIndex + 1,
            title: `Article ${position}`,
            summary: "Summary",
            prerequisites: [],
            tourPrerequisites: [],
            relatedTourLessons: [],
            sections: [{ id: "section", title: "Section", body: "Body" }],
            recap: ["Recap"],
            source: 'pub effect fn main = println "ok"\n',
            expectedOutput: "ok",
            diagnosticSource: 'let invalid: Int = "bad"\n',
            diagnosticOutput: "error",
            categoryId: category.id,
            chapterId: chapter.id,
            position,
          }
        })
      )
    )
    const categoriesWithArticles = categories.map((category) => ({
      ...category,
      chapters: category.chapters.map((chapter) => ({
        ...chapter,
        articles: articles.filter(({ chapterId }) => chapterId === chapter.id),
      })),
    }))
    const model = buildDeepDiveNavigation(
      categoriesWithArticles,
      articles,
      "article-47",
      articles.slice(0, 46).map(({ id }) => id)
    )

    expect(model.progress).toEqual({ completed: 46, total: 60 })
    expect(model.categories).toHaveLength(5)
    expect(model.categories[3]!.chapters[2]!.articles[2]!.state).toBe("current")
  })

  test("rejects duplicate ids, broken links, cycles, orphan files and empty sections", () => {
    const valid = validationFixture()

    expect(() =>
      validateDeepDive(
        {
          ...valid.catalog,
          categories: [
            ...valid.catalog.categories,
            { ...valid.catalog.categories[0]!, order: 2 },
          ],
        },
        valid.articles,
        valid.tourLessonIds,
        valid.files
      )
    ).toThrow("Duplicate Deep Dive category id")

    expect(() =>
      validateDeepDive(
        valid.catalog,
        [
          withContent(valid.articles[0]!, {
            prerequisites: ["missing-article"],
          }),
          valid.articles[1]!,
        ],
        valid.tourLessonIds,
        valid.files
      )
    ).toThrow("references missing prerequisite")

    expect(() =>
      validateDeepDive(
        valid.catalog,
        [
          withContent(valid.articles[0]!, { prerequisites: ["article-2"] }),
          withContent(valid.articles[1]!, { prerequisites: ["article-1"] }),
        ],
        valid.tourLessonIds,
        valid.files
      )
    ).toThrow("Deep Dive prerequisite cycle")

    expect(() =>
      validateDeepDive(valid.catalog, valid.articles, valid.tourLessonIds, [
        ...valid.files,
        "articles/orphan/article.json",
      ])
    ).toThrow("orphan: articles/orphan/article.json")

    expect(() =>
      validateDeepDive(
        valid.catalog,
        [withContent(valid.articles[0]!, { sections: [] }), valid.articles[1]!],
        valid.tourLessonIds,
        valid.files
      )
    ).toThrow("must contain a section")
  })

  test("ships an independent responsive page without a Learn or Discover entry", async () => {
    const root = new URL("..", import.meta.url)
    const [html, main, styles, tourHtml, tourMain, playgroundHtml, vite] =
      await Promise.all([
        Bun.file(new URL("deep-dive/index.html", root)).text(),
        Bun.file(new URL("src/deep-dive/main.ts", root)).text(),
        Bun.file(new URL("src/deep-dive/styles.css", root)).text(),
        Bun.file(new URL("tour/index.html", root)).text(),
        Bun.file(new URL("src/tour/main.ts", root)).text(),
        Bun.file(new URL("index.html", root)).text(),
        Bun.file(new URL("vite.config.ts", root)).text(),
      ])

    expect(html).toContain('id="deep-dive-navigation"')
    expect(html).toContain('id="deep-dive-menu-button"')
    expect(main).toContain('window.addEventListener("popstate"')
    expect(main).toContain('history[historyMode === "push" ? "pushState"')
    expect(styles).toContain("@media (max-width: 760px)")
    expect(tourHtml).toContain('id="tour-deep-dive-section"')
    expect(tourMain).toContain("deepDiveArticlesForTourLesson")
    expect(playgroundHtml).not.toContain('href="./deep-dive/"')
    expect(playgroundHtml).not.toContain(">Learn<")
    expect(vite).toContain("deepDive:")
    expect(vite).toContain("deep-dive/index.html")
  })
})

function validationFixture(): Readonly<{
  catalog: DeepDiveCatalog
  articles: readonly LoadedDeepDiveArticle[]
  tourLessonIds: readonly string[]
  files: readonly string[]
}> {
  const catalog: DeepDiveCatalog = {
    title: "Fixture",
    categories: [
      {
        id: "category",
        order: 1,
        title: "Category",
        summary: "Summary",
        chapters: [
          {
            id: "chapter",
            order: 1,
            title: "Chapter",
            summary: "Summary",
            articles: [
              {
                id: "article-1",
                order: 1,
                content: "articles/article-1/article.json",
              },
              {
                id: "article-2",
                order: 2,
                content: "articles/article-2/article.json",
              },
            ],
          },
        ],
      },
    ],
  }
  const articles = [1, 2].map((number) => ({
    id: `article-${number}`,
    categoryId: "category",
    chapterId: "chapter",
    order: number,
    contentPath: `articles/article-${number}/article.json`,
    content: articleContent(`article-${number}`),
    sourcePath: `/fixture/article-${number}/main.ssrg`,
    expectedOutputPath: `/fixture/article-${number}/stdout.txt`,
    diagnosticExamplePath: `/fixture/article-${number}/diagnostic.ssrg`,
    diagnosticOutputPath: `/fixture/article-${number}/diagnostic.txt`,
    source: 'pub effect fn main = println "ok"\n',
    expectedOutput: "ok\n",
    diagnosticSource: 'let invalid: Int = "bad"\n',
    diagnosticOutput: "error\n",
  }))
  return {
    catalog,
    articles,
    tourLessonIds: ["tour-lesson"],
    files: articles.map(({ contentPath }) => contentPath),
  }
}

function articleContent(id: string): DeepDiveArticleContent {
  return {
    id,
    title: `Article ${id}`,
    summary: "Summary",
    prerequisites: [],
    tourPrerequisites: ["tour-lesson"],
    relatedTourLessons: ["tour-lesson"],
    sections: [{ id: "section", title: "Section", body: "Body" }],
    recap: ["Recap"],
    files: {
      source: "main.ssrg",
      expectedOutput: "stdout.txt",
      diagnosticExample: "diagnostic.ssrg",
      diagnosticOutput: "diagnostic.txt",
    },
  }
}

function withContent(
  article: LoadedDeepDiveArticle,
  content: Partial<DeepDiveArticleContent>
): LoadedDeepDiveArticle {
  return { ...article, content: { ...article.content, ...content } }
}
