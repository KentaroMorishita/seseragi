export const tourProgressStorageKey = "seseragi.tour.progress.v1"

export type TourProgress = Readonly<{
  currentLessonId: string
  completedLessonIds: readonly string[]
}>

type ProgressStorage = Pick<Storage, "getItem" | "setItem">

export function loadTourProgress(
  storage: ProgressStorage,
  lessonIds: readonly string[],
  requestedLessonId?: string | null
): TourProgress {
  const fallback = lessonIds[0] ?? ""
  let stored: Partial<TourProgress> = {}
  try {
    const parsed = JSON.parse(
      storage.getItem(tourProgressStorageKey) ?? "{}"
    ) as Partial<TourProgress> | undefined
    stored = parsed ?? {}
  } catch {
    stored = {}
  }
  const known = new Set(lessonIds)
  const completedLessonIds = Array.isArray(stored.completedLessonIds)
    ? stored.completedLessonIds.filter(
        (id): id is string => typeof id === "string" && known.has(id)
      )
    : []
  const requested = requestedLessonId ?? undefined
  const currentLessonId =
    requested !== undefined && known.has(requested)
      ? requested
      : typeof stored.currentLessonId === "string" &&
          known.has(stored.currentLessonId)
        ? stored.currentLessonId
        : fallback
  return {
    currentLessonId,
    completedLessonIds: [...new Set(completedLessonIds)],
  }
}

export function saveTourProgress(
  storage: ProgressStorage,
  progress: TourProgress
): void {
  storage.setItem(tourProgressStorageKey, JSON.stringify(progress))
}

export function visitTourLesson(
  progress: TourProgress,
  lessonId: string
): TourProgress {
  return { ...progress, currentLessonId: lessonId }
}

export function completeTourLesson(
  progress: TourProgress,
  lessonId: string
): TourProgress {
  return {
    currentLessonId: lessonId,
    completedLessonIds: [
      ...new Set([...progress.completedLessonIds, lessonId]),
    ],
  }
}
