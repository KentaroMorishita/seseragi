export type ProviderConformanceCaseId =
  | "success"
  | "typed-failure"
  | "defect"
  | "cancellation"
  | "cleanup"
  | "concurrency"
  | "invalid-value"
  | "mismatch"
  | "ambiguity"
  | "leak"

type ProviderTerminal = "success" | "typed-failure" | "defect" | "cancellation"

type TerminalObservation = Readonly<{
  id: "success" | "typed-failure"
  terminal: ProviderTerminal
}>

type DefectObservation = Readonly<{
  id: "defect"
  terminal: ProviderTerminal
  stage: "input" | "call" | "result"
}>

type CancellationObservation = Readonly<{
  id: "cancellation"
  terminal: ProviderTerminal
  notifications: number
  lateCompletion: "discarded"
}>

type CleanupObservation = Readonly<{
  id: "cleanup"
  acquired: number
  released: number
  active: number
}>

type ConcurrencyObservation = Readonly<{
  id: "concurrency"
  started: number
  settled: number
  maximumActive: number
}>

type InvalidValueObservation = Readonly<{
  id: "invalid-value"
  boundary: "input" | "result"
  terminal: ProviderTerminal
  leakedToApplication: boolean
}>

type ResolutionObservation = Readonly<{
  id: "mismatch" | "ambiguity"
  phase: "resolution"
  entryEvaluations: number
}>

type LeakObservation = Readonly<{
  id: "leak"
  activeAfterCleanup: number
}>

export type ProviderConformanceObservation =
  | TerminalObservation
  | DefectObservation
  | CancellationObservation
  | CleanupObservation
  | ConcurrencyObservation
  | InvalidValueObservation
  | ResolutionObservation
  | LeakObservation

export class ProviderConformanceFailure extends Error {
  readonly caseId: ProviderConformanceCaseId

  constructor(caseId: ProviderConformanceCaseId, message: string) {
    super(`provider conformance ${caseId}: ${message}`)
    this.name = "ProviderConformanceFailure"
    this.caseId = caseId
  }
}

const requiredCases: ReadonlyArray<ProviderConformanceCaseId> = [
  "success",
  "typed-failure",
  "defect",
  "cancellation",
  "cleanup",
  "concurrency",
  "invalid-value",
  "mismatch",
  "ambiguity",
  "leak",
]

export function assertProviderConformanceProfile(
  observations: ReadonlyArray<ProviderConformanceObservation>
): void {
  const indexed = new Map<
    ProviderConformanceCaseId,
    ProviderConformanceObservation
  >()
  for (const observation of observations) {
    if (indexed.has(observation.id)) {
      fail(observation.id, "case is duplicated")
    }
    assertProviderConformanceCase(observation)
    indexed.set(observation.id, observation)
  }
  for (const caseId of requiredCases) {
    if (!indexed.has(caseId)) fail(caseId, "case is missing")
  }
}

export function assertProviderConformanceCase(
  observation: ProviderConformanceObservation
): void {
  switch (observation.id) {
    case "success":
      if (observation.terminal !== "success") {
        fail(observation.id, "must terminate with success")
      }
      return
    case "typed-failure":
      if (observation.terminal !== "typed-failure") {
        fail(observation.id, "must stay in the declared failure channel")
      }
      return
    case "defect":
      if (observation.terminal !== "defect") {
        fail(observation.id, "must terminate with a boundary defect")
      }
      return
    case "cancellation":
      natural(observation.id, "notifications", observation.notifications)
      if (
        observation.terminal !== "cancellation" ||
        observation.notifications > 1 ||
        observation.lateCompletion !== "discarded"
      ) {
        fail(
          observation.id,
          "must notify at most once and discard late completion"
        )
      }
      return
    case "cleanup":
      natural(observation.id, "acquired", observation.acquired)
      natural(observation.id, "released", observation.released)
      natural(observation.id, "active", observation.active)
      if (
        observation.acquired === 0 ||
        observation.released !== observation.acquired ||
        observation.active !== 0
      ) {
        fail(observation.id, "every acquired resource must be released")
      }
      return
    case "concurrency":
      natural(observation.id, "started", observation.started)
      natural(observation.id, "settled", observation.settled)
      natural(observation.id, "maximumActive", observation.maximumActive)
      if (
        observation.started < 2 ||
        observation.settled !== observation.started ||
        observation.maximumActive < 2
      ) {
        fail(observation.id, "must overlap and settle independent operations")
      }
      return
    case "invalid-value":
      if (
        observation.terminal !== "defect" ||
        observation.leakedToApplication
      ) {
        fail(
          observation.id,
          "must become an input or result defect before reaching application values"
        )
      }
      return
    case "mismatch":
    case "ambiguity":
      natural(observation.id, "entryEvaluations", observation.entryEvaluations)
      if (
        observation.phase !== "resolution" ||
        observation.entryEvaluations !== 0
      ) {
        fail(
          observation.id,
          "must be rejected before provider entry evaluation"
        )
      }
      return
    case "leak":
      natural(
        observation.id,
        "activeAfterCleanup",
        observation.activeAfterCleanup
      )
      if (observation.activeAfterCleanup !== 0) {
        fail(observation.id, "active handles remain after cleanup")
      }
      return
    default:
      throw new TypeError("unknown provider conformance case")
  }
}

function natural(
  caseId: ProviderConformanceCaseId,
  field: string,
  value: number
): void {
  if (!Number.isSafeInteger(value) || value < 0) {
    fail(caseId, `${field} must be a non-negative safe integer`)
  }
}

function fail(caseId: ProviderConformanceCaseId, message: string): never {
  throw new ProviderConformanceFailure(caseId, message)
}
