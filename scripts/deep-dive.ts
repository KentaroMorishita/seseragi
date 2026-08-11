import { readdir, readFile } from "node:fs/promises"
import { relative, resolve, sep } from "node:path"

export type DeepDiveSection = Readonly<{
  id: string
  title: string
  body: string
}>

export type DeepDiveArticleContent = Readonly<{
  id: string
  title: string
  summary: string
  prerequisites: readonly string[]
  tourPrerequisites: readonly string[]
  relatedTourLessons: readonly string[]
  sections: readonly DeepDiveSection[]
}>

export type DeepDiveArticleReference = Readonly<{
  id: string
  order: number
  content: string
}>

export type DeepDiveChapter = Readonly<{
  id: string
  order: number
  title: string
  summary: string
  articles: readonly DeepDiveArticleReference[]
}>

export type DeepDiveCategory = Readonly<{
  id: string
  order: number
  title: string
  summary: string
  chapters: readonly DeepDiveChapter[]
}>

export type DeepDiveCatalog = Readonly<{
  title: string
  categories: readonly DeepDiveCategory[]
}>

export type LoadedDeepDiveArticle = Readonly<{
  id: string
  categoryId: string
  chapterId: string
  order: number
  contentPath: string
  content: DeepDiveArticleContent
}>

export async function loadValidatedDeepDive(repositoryRoot: string): Promise<
  Readonly<{
    catalog: DeepDiveCatalog
    articles: readonly LoadedDeepDiveArticle[]
  }>
> {
  const catalogPath = resolve(repositoryRoot, "examples/deep-dive/catalog.json")
  const rawCatalog = JSON.parse(await readFile(catalogPath, "utf8")) as unknown
  const catalog = parseDeepDiveCatalog(rawCatalog)
  const references = flattenReferences(catalog)
  const articles = await Promise.all(
    references.map(async (reference) => ({
      ...reference,
      content: parseDeepDiveArticle(
        JSON.parse(
          await readFile(
            resolve(
              repositoryRoot,
              "examples/deep-dive",
              reference.contentPath
            ),
            "utf8"
          )
        ) as unknown,
        reference.contentPath
      ),
    }))
  )
  const tourLessonIds = await loadTourLessonIds(repositoryRoot)
  const files = await articleDescriptorPaths(repositoryRoot)
  validateDeepDive(catalog, articles, tourLessonIds, files)
  return { catalog, articles }
}

export function parseDeepDiveCatalog(value: unknown): DeepDiveCatalog {
  const root = expectRecord(value, "Deep Dive catalog")
  expectKeys(
    root,
    ["$schema", "schema", "title", "categories"],
    "Deep Dive catalog"
  )
  if (root.schema !== 1) throw new Error("Deep Dive catalog.schema must be 1")
  return {
    title: expectPlainText(root.title, "Deep Dive catalog.title"),
    categories: expectArray(
      root.categories,
      "Deep Dive catalog.categories"
    ).map((category, categoryIndex) => parseCategory(category, categoryIndex)),
  }
}

export function parseDeepDiveArticle(
  value: unknown,
  label = "Deep Dive article"
): DeepDiveArticleContent {
  const article = expectRecord(value, label)
  expectKeys(
    article,
    [
      "$schema",
      "id",
      "title",
      "summary",
      "prerequisites",
      "tourPrerequisites",
      "relatedTourLessons",
      "sections",
    ],
    label
  )
  return {
    id: expectSlug(article.id, `${label}.id`),
    title: expectPlainText(article.title, `${label}.title`),
    summary: expectPlainText(article.summary, `${label}.summary`),
    prerequisites: expectSlugs(article.prerequisites, `${label}.prerequisites`),
    tourPrerequisites: expectSlugs(
      article.tourPrerequisites,
      `${label}.tourPrerequisites`
    ),
    relatedTourLessons: expectSlugs(
      article.relatedTourLessons,
      `${label}.relatedTourLessons`
    ),
    sections: expectArray(article.sections, `${label}.sections`).map(
      (section, index) => parseSection(section, `${label}.sections.${index}`)
    ),
  }
}

export function validateDeepDive(
  catalog: DeepDiveCatalog,
  articles: readonly LoadedDeepDiveArticle[],
  tourLessonIds: readonly string[],
  articleFiles: readonly string[]
): void {
  if (catalog.categories.length === 0) {
    throw new Error("Deep Dive catalog must contain a category")
  }
  assertUnique(
    "Deep Dive category id",
    catalog.categories.map(({ id }) => id)
  )
  assertOrdered("Deep Dive category", catalog.categories)

  const chapters = catalog.categories.flatMap(({ chapters }) => chapters)
  const references = flattenReferences(catalog)
  assertUnique(
    "Deep Dive chapter id",
    chapters.map(({ id }) => id)
  )
  assertUnique(
    "Deep Dive article id",
    references.map(({ id }) => id)
  )
  for (const category of catalog.categories) {
    if (category.chapters.length === 0) {
      throw new Error(
        `Deep Dive category ${category.id} must contain a chapter`
      )
    }
    assertOrdered(
      `Deep Dive category ${category.id} chapter`,
      category.chapters
    )
    for (const chapter of category.chapters) {
      if (chapter.articles.length === 0) {
        throw new Error(
          `Deep Dive chapter ${chapter.id} must contain an article`
        )
      }
      assertOrdered(`Deep Dive chapter ${chapter.id} article`, chapter.articles)
    }
  }

  const articleIds = new Set(references.map(({ id }) => id))
  const tourIds = new Set(tourLessonIds)
  const expectedFiles = references.map(({ contentPath }) => contentPath).sort()
  const actualFiles = [...articleFiles].sort()
  if (!sameStrings(expectedFiles, actualFiles)) {
    const missing = expectedFiles.filter((path) => !actualFiles.includes(path))
    const orphan = actualFiles.filter((path) => !expectedFiles.includes(path))
    throw new Error(
      `Deep Dive article file mismatch; missing: ${missing.join(", ") || "none"}; orphan: ${orphan.join(", ") || "none"}`
    )
  }
  if (articles.length !== references.length) {
    throw new Error("Every Deep Dive article reference must have content")
  }

  const articleById = new Map<string, DeepDiveArticleContent>()
  for (const article of articles) {
    if (
      article.content.id !==
      article.contentPath.match(/^articles\/([^/]+)\/article\.json$/u)?.[1]
    ) {
      throw new Error(
        `Deep Dive article ${article.contentPath} id must match its directory`
      )
    }
    if (
      article.content.id !==
      references.find(({ contentPath }) => contentPath === article.contentPath)
        ?.id
    ) {
      throw new Error(
        `Deep Dive article ${article.contentPath} id must match its catalog reference`
      )
    }
    if (article.content.sections.length === 0) {
      throw new Error(
        `Deep Dive article ${article.content.id} must contain a section`
      )
    }
    assertUnique(
      `Deep Dive article ${article.content.id} section id`,
      article.content.sections.map(({ id }) => id)
    )
    assertUnique(
      `Deep Dive article ${article.content.id} prerequisite`,
      article.content.prerequisites
    )
    assertUnique(
      `Deep Dive article ${article.content.id} Tour prerequisite`,
      article.content.tourPrerequisites
    )
    assertUnique(
      `Deep Dive article ${article.content.id} related Tour lesson`,
      article.content.relatedTourLessons
    )
    for (const linkedId of article.content.prerequisites) {
      if (!articleIds.has(linkedId)) {
        throw new Error(
          `Deep Dive article ${article.content.id} references missing prerequisite ${linkedId}`
        )
      }
    }
    for (const lessonId of [
      ...article.content.tourPrerequisites,
      ...article.content.relatedTourLessons,
    ]) {
      if (!tourIds.has(lessonId)) {
        throw new Error(
          `Deep Dive article ${article.content.id} references missing Tour lesson ${lessonId}`
        )
      }
    }
    articleById.set(article.content.id, article.content)
  }
  validatePrerequisiteGraph(articleById)
}

function parseCategory(value: unknown, index: number): DeepDiveCategory {
  const label = `Deep Dive catalog.categories.${index}`
  const category = expectRecord(value, label)
  expectKeys(category, ["id", "order", "title", "summary", "chapters"], label)
  return {
    id: expectSlug(category.id, `${label}.id`),
    order: expectInteger(category.order, `${label}.order`),
    title: expectPlainText(category.title, `${label}.title`),
    summary: expectPlainText(category.summary, `${label}.summary`),
    chapters: expectArray(category.chapters, `${label}.chapters`).map(
      (chapter, chapterIndex) =>
        parseChapter(chapter, `${label}.chapters.${chapterIndex}`)
    ),
  }
}

function parseChapter(value: unknown, label: string): DeepDiveChapter {
  const chapter = expectRecord(value, label)
  expectKeys(chapter, ["id", "order", "title", "summary", "articles"], label)
  return {
    id: expectSlug(chapter.id, `${label}.id`),
    order: expectInteger(chapter.order, `${label}.order`),
    title: expectPlainText(chapter.title, `${label}.title`),
    summary: expectPlainText(chapter.summary, `${label}.summary`),
    articles: expectArray(chapter.articles, `${label}.articles`).map(
      (article, index) =>
        parseArticleReference(article, `${label}.articles.${index}`)
    ),
  }
}

function parseArticleReference(
  value: unknown,
  label: string
): DeepDiveArticleReference {
  const reference = expectRecord(value, label)
  expectKeys(reference, ["id", "order", "content"], label)
  const id = expectSlug(reference.id, `${label}.id`)
  const content = expectString(reference.content, `${label}.content`)
  const expected = `articles/${id}/article.json`
  if (content !== expected)
    throw new Error(`${label}.content must be ${expected}`)
  return {
    id,
    order: expectInteger(reference.order, `${label}.order`),
    content,
  }
}

function parseSection(value: unknown, label: string): DeepDiveSection {
  const section = expectRecord(value, label)
  expectKeys(section, ["id", "title", "body"], label)
  return {
    id: expectSlug(section.id, `${label}.id`),
    title: expectPlainText(section.title, `${label}.title`),
    body: expectString(section.body, `${label}.body`),
  }
}

function flattenReferences(catalog: DeepDiveCatalog): LoadedDeepDiveArticle[] {
  return catalog.categories.flatMap((category) =>
    category.chapters.flatMap((chapter) =>
      chapter.articles.map((article) => ({
        id: article.id,
        categoryId: category.id,
        chapterId: chapter.id,
        order: article.order,
        contentPath: article.content,
        content: undefined as never,
      }))
    )
  )
}

async function loadTourLessonIds(repositoryRoot: string): Promise<string[]> {
  const value = JSON.parse(
    await readFile(
      resolve(repositoryRoot, "examples/tour/curriculum.json"),
      "utf8"
    )
  ) as { categories?: { chapters?: { lessons?: { id?: unknown }[] }[] }[] }
  return (value.categories ?? []).flatMap((category) =>
    (category.chapters ?? []).flatMap((chapter) =>
      (chapter.lessons ?? []).flatMap((lesson) =>
        typeof lesson.id === "string" ? [lesson.id] : []
      )
    )
  )
}

async function articleDescriptorPaths(
  repositoryRoot: string
): Promise<string[]> {
  const base = resolve(repositoryRoot, "examples/deep-dive")
  const articleRoot = resolve(base, "articles")
  const entries = await readdir(articleRoot, { recursive: true })
  return entries
    .filter((entry) => entry.endsWith("article.json"))
    .map((entry) =>
      relative(base, resolve(articleRoot, entry)).split(sep).join("/")
    )
}

function validatePrerequisiteGraph(
  articles: ReadonlyMap<string, DeepDiveArticleContent>
): void {
  const visiting = new Set<string>()
  const visited = new Set<string>()
  const visit = (id: string, path: readonly string[]): void => {
    if (visited.has(id)) return
    if (visiting.has(id)) {
      const cycleStart = path.indexOf(id)
      throw new Error(
        `Deep Dive prerequisite cycle: ${[...path.slice(cycleStart), id].join(" -> ")}`
      )
    }
    visiting.add(id)
    for (const prerequisite of articles.get(id)?.prerequisites ?? []) {
      visit(prerequisite, [...path, id])
    }
    visiting.delete(id)
    visited.add(id)
  }
  for (const id of articles.keys()) visit(id, [])
}

function expectRecord(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${label} must be an object`)
  }
  return value as Record<string, unknown>
}

function expectKeys(
  record: Record<string, unknown>,
  allowed: readonly string[],
  label: string
): void {
  const unknown = Object.keys(record).filter((key) => !allowed.includes(key))
  if (unknown.length > 0) {
    throw new Error(`${label} has unknown field(s): ${unknown.join(", ")}`)
  }
}

function expectArray(value: unknown, label: string): readonly unknown[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`)
  return value
}

function expectString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${label} must be a non-empty string`)
  }
  return value
}

function expectPlainText(value: unknown, label: string): string {
  const text = expectString(value, label)
  if (/[`*]|\[[^\]]+\]\([^)]+\)|^\s*(?:#|[-+]\s|\d+[.)]\s|>)/mu.test(text)) {
    throw new Error(`${label} must remain plain text without Markdown markers`)
  }
  return text
}

function expectSlug(value: unknown, label: string): string {
  const text = expectString(value, label)
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/u.test(text)) {
    throw new Error(`${label} must be a stable slug`)
  }
  return text
}

function expectSlugs(value: unknown, label: string): readonly string[] {
  return expectArray(value, label).map((item, index) =>
    expectSlug(item, `${label}.${index}`)
  )
}

function expectInteger(value: unknown, label: string): number {
  if (!Number.isInteger(value)) throw new Error(`${label} must be an integer`)
  return value as number
}

function assertUnique(label: string, values: readonly string[]): void {
  const seen = new Set<string>()
  for (const value of values) {
    if (seen.has(value)) throw new Error(`Duplicate ${label}: ${value}`)
    seen.add(value)
  }
}

function assertOrdered(
  label: string,
  entries: readonly Readonly<{ id: string; order: number }>[]
): void {
  assertUnique(
    `${label} order`,
    entries.map(({ order }) => String(order))
  )
  for (const [index, entry] of entries.entries()) {
    if (entry.order !== index + 1) {
      throw new Error(
        `${label} ${entry.id} has order ${entry.order}; expected ${index + 1}`
      )
    }
  }
}

function sameStrings(
  left: readonly string[],
  right: readonly string[]
): boolean {
  return (
    left.length === right.length &&
    left.every((value, index) => value === right[index])
  )
}
