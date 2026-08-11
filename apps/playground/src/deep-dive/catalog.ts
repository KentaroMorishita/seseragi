import { generatedDeepDive } from "../generated/deep-dive-manifest"

export type DeepDiveArticleSection = Readonly<{
  id: string
  title: string
  body: string
}>

export type DeepDiveArticleData = Readonly<{
  id: string
  order: number
  title: string
  summary: string
  prerequisites: readonly string[]
  tourPrerequisites: readonly string[]
  relatedTourLessons: readonly string[]
  sections: readonly DeepDiveArticleSection[]
  recap: readonly string[]
  source: string
  expectedOutput: string
  diagnosticSource: string
  diagnosticOutput: string
}>

export type DeepDiveChapterData = Readonly<{
  id: string
  order: number
  title: string
  summary: string
  articles: readonly DeepDiveArticleData[]
}>

export type DeepDiveCategoryData = Readonly<{
  id: string
  order: number
  title: string
  summary: string
  chapters: readonly DeepDiveChapterData[]
}>

export type DeepDiveCatalogData = Readonly<{
  title: string
  categories: readonly DeepDiveCategoryData[]
}>

export type DeepDiveArticle = DeepDiveArticleData &
  Readonly<{
    categoryId: string
    chapterId: string
    position: number
  }>

export const deepDiveCategories = generatedDeepDive.categories
export const deepDiveChapters = deepDiveCategories.flatMap((category) =>
  category.chapters.map((chapter) => ({
    ...chapter,
    categoryId: category.id,
  }))
)
export const deepDiveArticles: readonly DeepDiveArticle[] = deepDiveCategories
  .flatMap((category) =>
    category.chapters.flatMap((chapter) =>
      chapter.articles.map((article) => ({
        ...article,
        categoryId: category.id,
        chapterId: chapter.id,
        position: 0,
      }))
    )
  )
  .map((article, index) => ({ ...article, position: index + 1 }))

export const deepDiveTitle = generatedDeepDive.title

export function findDeepDiveArticle(id: string | null): DeepDiveArticle {
  return (
    deepDiveArticles.find((article) => article.id === id) ??
    deepDiveArticles[0]!
  )
}

export function deepDiveArticlesForTourLesson(
  lessonId: string
): readonly DeepDiveArticle[] {
  return deepDiveArticles.filter(({ relatedTourLessons }) =>
    relatedTourLessons.includes(lessonId)
  )
}
