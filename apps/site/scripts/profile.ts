import { appendFileSync } from "node:fs"

// Optional diagnostics never enter the published site or its manifest.
export function sitePhase<T>(
  phase: string,
  action: () => T,
  details: Record<string, number | string> = {}
): T {
  const path = process.env.SESERAGI_SITE_PROFILE
  if (!path) return action()
  const started = performance.now()
  let completed = false
  try {
    const result = action()
    completed = true
    return result
  } finally {
    appendFileSync(
      path,
      `${JSON.stringify({
        pid: process.pid,
        phase,
        ...details,
        elapsedMs: performance.now() - started,
        completed,
      })}\n`
    )
  }
}
