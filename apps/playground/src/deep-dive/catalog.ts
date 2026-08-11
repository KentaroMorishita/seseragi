import { generatedDeepDive } from "../generated/deep-dive-manifest"

export type DeepDiveArticleSection = Readonly<{
  id: string
  title: string
  body: string
}>

export type DeepDiveRelatedLink = Readonly<{
  label: string
  href: string
}>

export type DeepDiveArticleData = Readonly<{
  id: string
  order: number
  title: string
  summary: string
  relatedTourLessons: readonly string[]
  relatedLinks: readonly DeepDiveRelatedLink[]
  sections: readonly DeepDiveArticleSection[]
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
  }>

export const deepDiveCategories = generatedDeepDive.categories
export const deepDiveChapters = deepDiveCategories.flatMap((category) =>
  category.chapters.map((chapter) => ({
    ...chapter,
    categoryId: category.id,
  }))
)
export const deepDiveArticles: readonly DeepDiveArticle[] =
  deepDiveCategories.flatMap((category) =>
    category.chapters.flatMap((chapter) =>
      chapter.articles.map((article) => ({
        ...article,
        categoryId: category.id,
        chapterId: chapter.id,
      }))
    )
  )

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
