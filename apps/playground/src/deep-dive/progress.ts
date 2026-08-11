export const deepDiveProgressStorageKey = "seseragi.deepDive.progress.v1"

export type DeepDiveProgress = Readonly<{
  currentArticleId: string
  completedArticleIds: readonly string[]
}>

type ProgressStorage = Pick<Storage, "getItem" | "setItem">

type StoredDeepDiveProgress = DeepDiveProgress & Readonly<{ schema: 1 }>

export function loadDeepDiveProgress(
  storage: ProgressStorage,
  articleIds: readonly string[],
  requestedArticleId?: string | null
): DeepDiveProgress {
  const known = new Set(articleIds)
  let stored: Partial<StoredDeepDiveProgress> = {}
  try {
    const parsed = JSON.parse(
      storage.getItem(deepDiveProgressStorageKey) ?? "{}"
    ) as Partial<StoredDeepDiveProgress>
    stored = parsed.schema === 1 ? parsed : {}
  } catch {
    stored = {}
  }
  const completedArticleIds = Array.isArray(stored.completedArticleIds)
    ? [...new Set(stored.completedArticleIds)].filter(
        (id): id is string => typeof id === "string" && known.has(id)
      )
    : []
  const currentArticleId =
    requestedArticleId !== undefined &&
    requestedArticleId !== null &&
    known.has(requestedArticleId)
      ? requestedArticleId
      : typeof stored.currentArticleId === "string" &&
          known.has(stored.currentArticleId)
        ? stored.currentArticleId
        : (articleIds[0] ?? "")
  return { currentArticleId, completedArticleIds }
}

export function saveDeepDiveProgress(
  storage: ProgressStorage,
  progress: DeepDiveProgress
): void {
  const stored: StoredDeepDiveProgress = { schema: 1, ...progress }
  storage.setItem(deepDiveProgressStorageKey, JSON.stringify(stored))
}

export function visitDeepDiveArticle(
  progress: DeepDiveProgress,
  articleId: string
): DeepDiveProgress {
  return { ...progress, currentArticleId: articleId }
}

export function completeDeepDiveArticle(
  progress: DeepDiveProgress,
  articleId: string
): DeepDiveProgress {
  return {
    currentArticleId: articleId,
    completedArticleIds: [
      ...new Set([...progress.completedArticleIds, articleId]),
    ],
  }
}
