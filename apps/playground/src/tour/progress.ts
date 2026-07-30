export const tourProgressStorageKey = "seseragi.tour.progress.v2"
export const legacyTourProgressStorageKey = "seseragi.tour.progress.v1"

export type TourProgress = Readonly<{
  currentLessonId: string
  completedLessonIds: readonly string[]
}>

type ProgressStorage = Pick<Storage, "getItem" | "setItem">

type StoredTourProgress = TourProgress &
  Readonly<{
    schema: 2
  }>

export function loadTourProgress(
  storage: ProgressStorage,
  lessonIds: readonly string[],
  requestedLessonId?: string | null
): TourProgress {
  const fallback = lessonIds[0] ?? ""
  let stored: Partial<TourProgress> = {}
  const current = storage.getItem(tourProgressStorageKey)
  const legacy =
    current === null ? storage.getItem(legacyTourProgressStorageKey) : null
  try {
    const parsed = JSON.parse(current ?? legacy ?? "{}") as
      | Partial<StoredTourProgress>
      | undefined
    stored =
      parsed !== undefined && (legacy !== null || parsed.schema === 2)
        ? parsed
        : {}
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
  const progress = {
    currentLessonId,
    completedLessonIds: [...new Set(completedLessonIds)],
  }
  if (legacy !== null) saveTourProgress(storage, progress)
  return progress
}

export function saveTourProgress(
  storage: ProgressStorage,
  progress: TourProgress
): void {
  const stored: StoredTourProgress = { schema: 2, ...progress }
  storage.setItem(tourProgressStorageKey, JSON.stringify(stored))
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
