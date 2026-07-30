export type TourNavigationCategoryInput = Readonly<{
  id: string
  title: string
  summary: string
}>

export type TourNavigationChapterInput = Readonly<{
  id: string
  categoryId: string
  title: string
  summary: string
}>

export type TourNavigationLessonInput = Readonly<{
  id: string
  categoryId: string
  chapterId: string
  position: number
  title: string
  goal: string
}>

export type TourLessonState = "current" | "completed" | "unstarted"

export type TourProgressSummary = Readonly<{
  completed: number
  total: number
}>

export type TourNavigationLesson = TourNavigationLessonInput &
  Readonly<{
    state: TourLessonState
  }>

export type TourNavigationChapter = TourNavigationChapterInput &
  Readonly<{
    progress: TourProgressSummary
    lessons: readonly TourNavigationLesson[]
  }>

export type TourNavigationCategory = TourNavigationCategoryInput &
  Readonly<{
    progress: TourProgressSummary
    goal: string
    resumeLessonId: string
    resumeLessonTitle: string
    chapters: readonly TourNavigationChapter[]
  }>

export type TourNavigationModel = Readonly<{
  progress: TourProgressSummary
  categories: readonly TourNavigationCategory[]
}>

export type TourLessonNeighbor = Readonly<{
  id: string
  title: string
  categoryId: string
  categoryTitle: string
  crossesCategory: boolean
}>

export type TourLessonNeighbors = Readonly<{
  previous?: TourLessonNeighbor
  next?: TourLessonNeighbor
}>

export function buildTourNavigationModel(
  categories: readonly TourNavigationCategoryInput[],
  chapters: readonly TourNavigationChapterInput[],
  lessons: readonly TourNavigationLessonInput[],
  currentLessonId: string,
  completedLessonIds: readonly string[]
): TourNavigationModel {
  const completed = new Set(completedLessonIds)
  const navigationCategories = categories.map((category) => {
    const categoryLessons = lessons.filter(
      ({ categoryId }) => categoryId === category.id
    )
    const categoryChapters = chapters
      .filter(({ categoryId }) => categoryId === category.id)
      .map((chapter) => {
        const chapterLessons = categoryLessons
          .filter(({ chapterId }) => chapterId === chapter.id)
          .map((lesson) => ({
            ...lesson,
            state: lessonState(lesson.id, currentLessonId, completed),
          }))
        return {
          ...chapter,
          progress: progressSummary(chapterLessons, completed),
          lessons: chapterLessons,
        }
      })
    const resumeLesson =
      categoryLessons.find(({ id }) => !completed.has(id)) ??
      categoryLessons.at(-1)
    return {
      ...category,
      progress: progressSummary(categoryLessons, completed),
      goal: categoryLessons.at(-1)?.goal ?? category.summary,
      resumeLessonId: resumeLesson?.id ?? "",
      resumeLessonTitle: resumeLesson?.title ?? "",
      chapters: categoryChapters,
    }
  })

  return {
    progress: progressSummary(lessons, completed),
    categories: navigationCategories,
  }
}

export function tourLessonNeighbors(
  categories: readonly TourNavigationCategoryInput[],
  lessons: readonly TourNavigationLessonInput[],
  currentLessonId: string
): TourLessonNeighbors {
  const index = lessons.findIndex(({ id }) => id === currentLessonId)
  if (index < 0) return {}
  const current = lessons[index]!
  return {
    previous: neighbor(categories, lessons[index - 1], current.categoryId),
    next: neighbor(categories, lessons[index + 1], current.categoryId),
  }
}

function lessonState(
  lessonId: string,
  currentLessonId: string,
  completed: ReadonlySet<string>
): TourLessonState {
  if (lessonId === currentLessonId) return "current"
  if (completed.has(lessonId)) return "completed"
  return "unstarted"
}

function progressSummary(
  lessons: readonly Pick<TourNavigationLessonInput, "id">[],
  completed: ReadonlySet<string>
): TourProgressSummary {
  return {
    completed: lessons.filter(({ id }) => completed.has(id)).length,
    total: lessons.length,
  }
}

function neighbor(
  categories: readonly TourNavigationCategoryInput[],
  lesson: TourNavigationLessonInput | undefined,
  currentCategoryId: string
): TourLessonNeighbor | undefined {
  if (lesson === undefined) return undefined
  return {
    id: lesson.id,
    title: lesson.title,
    categoryId: lesson.categoryId,
    categoryTitle:
      categories.find(({ id }) => id === lesson.categoryId)?.title ??
      lesson.categoryId,
    crossesCategory: lesson.categoryId !== currentCategoryId,
  }
}
