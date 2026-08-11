import type {
  DeepDiveArticle,
  DeepDiveCategoryData,
  DeepDiveChapterData,
} from "./catalog"

export type DeepDiveProgressSummary = Readonly<{
  completed: number
  total: number
}>

export type DeepDiveNavigationArticle = DeepDiveArticle &
  Readonly<{ state: "current" | "completed" | "unstarted" }>

export type DeepDiveNavigationChapter = DeepDiveChapterData &
  Readonly<{
    progress: DeepDiveProgressSummary
    articles: readonly DeepDiveNavigationArticle[]
  }>

export type DeepDiveNavigationCategory = DeepDiveCategoryData &
  Readonly<{
    progress: DeepDiveProgressSummary
    chapters: readonly DeepDiveNavigationChapter[]
  }>

export function buildDeepDiveNavigation(
  categories: readonly DeepDiveCategoryData[],
  articles: readonly DeepDiveArticle[],
  currentArticleId: string,
  completedArticleIds: readonly string[]
): Readonly<{
  progress: DeepDiveProgressSummary
  categories: readonly DeepDiveNavigationCategory[]
}> {
  const completed = new Set(completedArticleIds)
  const navigationCategories = categories.map((category) => {
    const categoryArticles = articles.filter(
      ({ categoryId }) => categoryId === category.id
    )
    return {
      ...category,
      progress: progressSummary(categoryArticles, completed),
      chapters: category.chapters.map((chapter) => {
        const chapterArticles = categoryArticles
          .filter(({ chapterId }) => chapterId === chapter.id)
          .map((article) => ({
            ...article,
            state:
              article.id === currentArticleId
                ? ("current" as const)
                : completed.has(article.id)
                  ? ("completed" as const)
                  : ("unstarted" as const),
          }))
        return {
          ...chapter,
          progress: progressSummary(chapterArticles, completed),
          articles: chapterArticles,
        }
      }),
    }
  })
  return {
    progress: progressSummary(articles, completed),
    categories: navigationCategories,
  }
}

function progressSummary(
  articles: readonly Readonly<{ id: string }>[],
  completed: ReadonlySet<string>
): DeepDiveProgressSummary {
  return {
    completed: articles.filter(({ id }) => completed.has(id)).length,
    total: articles.length,
  }
}
