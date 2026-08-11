import { describe, expect, test } from "bun:test"
import {
  type DeepDiveArticleContent,
  type DeepDiveCatalog,
  type LoadedDeepDiveArticle,
  loadValidatedDeepDive,
  validateDeepDive,
} from "../../../scripts/deep-dive"
import { deepDiveArticles } from "../src/deep-dive/catalog"
import {
  deepDiveArticleIdFromUrl,
  deepDiveArticleUrl,
} from "../src/deep-dive/route"

describe("Seseragi Articles", () => {
  test("loads topic categories and links every article back to Tour", async () => {
    const repositoryRoot = new URL("../../..", import.meta.url).pathname
    const loaded = await loadValidatedDeepDive(repositoryRoot)

    expect(loaded.catalog.title).toBe("Seseragi Articles")
    expect(loaded.catalog.categories).toHaveLength(3)
    expect(loaded.articles.map(({ content }) => content.id)).toEqual(
      deepDiveArticles.map(({ id }) => id)
    )
    expect(deepDiveArticles).toHaveLength(5)
    expect(deepDiveArticles[0]).toMatchObject({
      categoryId: "architecture",
      chapterId: "reading-the-system",
      relatedTourLessons: ["design-learning-map"],
    })
    for (const article of loaded.articles) {
      expect(article.content.relatedTourLessons.length).toBeGreaterThan(0)
      expect(article.content.relatedLinks.length).toBeGreaterThan(0)
      expect(article.content.sections.length).toBeGreaterThan(0)
    }
  })

  test("keeps direct URL, reload, back and forward on stable article ids", () => {
    const articleIds = ["article-1", "article-2", "article-3"]
    const direct = "https://example.test/deep-dive/?article=article-2"
    const history = [
      direct,
      deepDiveArticleUrl(direct, "article-3"),
      deepDiveArticleUrl(direct, "article-1"),
    ]

    expect(deepDiveArticleIdFromUrl(direct, articleIds)).toBe("article-2")
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

  test("rejects duplicate ids, broken Tour links, orphan files and empty sections", () => {
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
            relatedTourLessons: ["missing-lesson"],
          }),
          valid.articles[1]!,
        ],
        valid.tourLessonIds,
        valid.files
      )
    ).toThrow("references missing Tour lesson")

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

  test("ships a responsive article surface without course progress", async () => {
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
    expect(html).toContain('id="deep-dive-related"')
    expect(html).not.toContain("deep-dive-progress")
    expect(html).not.toContain("complete-button")
    expect(html).not.toContain("前後の記事")
    expect(main).toContain('window.addEventListener("popstate"')
    expect(main).not.toContain("localStorage")
    expect(main).not.toContain("completedArticleIds")
    expect(styles).toContain("@media (max-width: 760px)")
    expect(tourHtml).toContain('id="tour-deep-dive-section"')
    expect(tourMain).toContain("deepDiveArticlesForTourLesson")
    expect(playgroundHtml).not.toContain('href="./deep-dive/"')
    expect(vite).toContain("deepDive:")
    expect(vite).toContain("deep-dive/index.html")
  })

  test("keeps executable teaching files only in canonical Tour lessons", async () => {
    const root = new URL("../../..", import.meta.url)
    const articleFiles = Array.from(
      new Bun.Glob("examples/deep-dive/articles/**/*").scanSync({
        cwd: root.pathname,
        onlyFiles: true,
      })
    )
    expect(articleFiles.every((file) => file.endsWith("article.json"))).toBe(
      true
    )
    for (const lessonId of [
      "design-learning-map",
      "design-type-constructor-kinds",
      "design-trait-evidence",
      "design-abstraction-laws",
      "design-trait-boundary",
    ]) {
      expect(
        await Bun.file(
          new URL(`examples/tour/lessons/${lessonId}/main.ssrg`, root)
        ).exists()
      ).toBe(true)
    }
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
            id: "topic",
            order: 1,
            title: "Topic",
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
    chapterId: "topic",
    order: number,
    contentPath: `articles/article-${number}/article.json`,
    content: articleContent(`article-${number}`),
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
    relatedTourLessons: ["tour-lesson"],
    relatedLinks: [{ label: "Reference", href: "https://example.test/spec" }],
    sections: [{ id: "section", title: "Section", body: "Body" }],
  }
}

function withContent(
  article: LoadedDeepDiveArticle,
  content: Partial<DeepDiveArticleContent>
): LoadedDeepDiveArticle {
  return { ...article, content: { ...article.content, ...content } }
}
