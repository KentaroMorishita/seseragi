export type LiveAnalysisController = {
  readonly schedule: (source: string, identity?: string) => number
  readonly cancel: () => void
}

type LiveAnalysisOptions<Result> = {
  readonly analyze: (source: string, identity?: string) => Promise<Result>
  readonly apply: (analysis: Result, source: string, identity?: string) => void
  readonly onPending?: (source: string, identity?: string) => void
  readonly onError?: (error: unknown, source: string, identity?: string) => void
  readonly delayMs?: number
}

export function createLiveAnalysis<Result>(
  options: LiveAnalysisOptions<Result>
): LiveAnalysisController {
  let revision = 0
  let timer: ReturnType<typeof setTimeout> | undefined
  let inFlight = false
  let pending:
    | { source: string; identity?: string; revision: number }
    | undefined

  const flush = (): void => {
    if (inFlight || pending === undefined) return
    const request = pending
    pending = undefined
    inFlight = true
    void Promise.resolve()
      .then(() => options.analyze(request.source, request.identity))
      .then(
        (analysis) => {
          if (request.revision !== revision) return
          try {
            options.apply(analysis, request.source, request.identity)
          } catch (error) {
            options.onError?.(error, request.source, request.identity)
          }
        },
        (error: unknown) => {
          if (request.revision !== revision) return
          options.onError?.(error, request.source, request.identity)
        }
      )
      .finally(() => {
        inFlight = false
        // Edits during a slow analysis replace the pending request instead
        // of building an unbounded queue in the compiler worker.
        if (timer === undefined) flush()
      })
  }

  const schedule = (source: string, identity?: string): number => {
    revision += 1
    const scheduledRevision = revision
    pending = { source, identity, revision }
    if (timer !== undefined) clearTimeout(timer)
    options.onPending?.(source, identity)
    timer = setTimeout(() => {
      timer = undefined
      flush()
    }, options.delayMs ?? 240)
    return scheduledRevision
  }

  return {
    schedule,
    cancel: () => {
      revision += 1
      if (timer !== undefined) clearTimeout(timer)
      timer = undefined
      pending = undefined
    },
  }
}
