import type {
  Diagnostic,
  ProjectFileDiagnostics,
  ProjectProblem,
  ProjectRequest,
} from "../compiler/types"

export type WorkspaceDiagnostic = Readonly<{
  path: string
  source: string
  diagnostic: Diagnostic
}>

export function collectWorkspaceDiagnostics(
  request: ProjectRequest,
  files: readonly ProjectFileDiagnostics[],
  problems: readonly ProjectProblem[] = []
): readonly WorkspaceDiagnostic[] {
  const sources = new Map(
    request.files.map(({ path, source }) => [path, source])
  )
  const diagnostics = files.flatMap(({ path, diagnostics }) =>
    diagnostics.diagnostics.map((diagnostic) => ({
      path,
      source: sources.get(path) ?? "",
      diagnostic,
    }))
  )
  const problemDiagnostics: WorkspaceDiagnostic[] = problems.map((problem) => {
    const path = problem.path ?? request.files[0]?.path ?? "main.ssrg"
    const primary = problem.primary ?? { start: 0, end: 0 }
    return {
      path,
      source: sources.get(path) ?? "",
      diagnostic: {
        code: problem.code,
        messageKey: problem.label ?? "project.problem",
        message: problem.message,
        severity: "Error" as const,
        primary,
        related: [],
        labels: [],
        notes: providerProblemNotes(problem),
        helps: [],
        fixes: [],
        expectedType: null,
        actualType: null,
      },
    }
  })

  const seen = new Set<string>()
  return [...diagnostics, ...problemDiagnostics].filter(
    ({ path, diagnostic }) => {
      const key = [
        path,
        diagnostic.code,
        diagnostic.primary.start,
        diagnostic.primary.end,
        diagnostic.message,
      ].join("\0")
      if (seen.has(key)) return false
      seen.add(key)
      return true
    }
  )
}

function providerProblemNotes(problem: ProjectProblem): readonly string[] {
  const details = problem.details
  if (!details) return []
  const notes: string[] = []
  for (const [label, value] of [
    ["service", details.service],
    ["target", details.target],
    ["backend", details.backendFamily],
    ["provider", details.provider],
  ] as const) {
    if (value) notes.push(`${label}: ${value}`)
  }
  if (details.backendAbiMajor !== undefined) {
    notes.push(`backend ABI major: ${details.backendAbiMajor}`)
  }
  for (const [label, values] of [
    ["candidates", details.candidates],
    ["compatible targets", details.compatibleTargets],
    ["reasons", details.reasons],
    ["required", details.required],
    ["actual", details.actual],
  ] as const) {
    if (values && values.length > 0)
      notes.push(`${label}: ${values.join(", ")}`)
  }
  return notes
}
