import type { AnalysisDocument } from "../compiler/types"

export type LiveAnalysisController = {
  readonly schedule: (source: string, identity?: string) => number
  readonly cancel: () => void
}

type LiveAnalysisOptions = {
  readonly analyze: (source: string) => Promise<AnalysisDocument>
  readonly apply: (
    analysis: AnalysisDocument,
    source: string,
    identity?: string
  ) => void
  readonly onPending?: (source: string, identity?: string) => void
  readonly onError?: (error: unknown, source: string, identity?: string) => void
  readonly delayMs?: number
}

export function createLiveAnalysis(
  options: LiveAnalysisOptions
): LiveAnalysisController {
  let revision = 0
  let timer: ReturnType<typeof setTimeout> | undefined

  const schedule = (source: string, identity?: string): number => {
    revision += 1
    const scheduledRevision = revision
    if (timer !== undefined) clearTimeout(timer)
    options.onPending?.(source, identity)
    timer = setTimeout(() => {
      timer = undefined
      void options.analyze(source).then(
        (analysis) => {
          if (scheduledRevision !== revision) return
          try {
            options.apply(analysis, source, identity)
          } catch (error) {
            options.onError?.(error, source, identity)
          }
        },
        (error: unknown) => {
          if (scheduledRevision !== revision) return
          options.onError?.(error, source, identity)
        }
      )
    }, options.delayMs ?? 240)
    return scheduledRevision
  }

  return {
    schedule,
    cancel: () => {
      revision += 1
      if (timer !== undefined) clearTimeout(timer)
      timer = undefined
    },
  }
}
