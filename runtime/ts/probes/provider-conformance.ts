import {
  assertProviderConformanceCase,
  assertProviderConformanceProfile,
  ProviderConformanceFailure,
} from "@seseragi/runtime/provider-conformance"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

assertProviderConformanceProfile([
  { id: "success", terminal: "success" },
  { id: "typed-failure", terminal: "typed-failure" },
  { id: "defect", terminal: "defect", stage: "call" },
  {
    id: "cancellation",
    terminal: "cancellation",
    notifications: 1,
    lateCompletion: "discarded",
  },
  { id: "cleanup", acquired: 2, released: 2, active: 0 },
  { id: "concurrency", started: 2, settled: 2, maximumActive: 2 },
  {
    id: "invalid-value",
    boundary: "result",
    terminal: "defect",
    leakedToApplication: false,
  },
  { id: "mismatch", phase: "resolution", entryEvaluations: 0 },
  { id: "ambiguity", phase: "resolution", entryEvaluations: 0 },
  { id: "leak", activeAfterCleanup: 0 },
])

for (const invalid of [
  {
    observation: { id: "leak", activeAfterCleanup: 1 } as const,
    caseId: "leak",
  },
  {
    observation: {
      id: "cancellation",
      terminal: "cancellation",
      notifications: 2,
      lateCompletion: "discarded",
    } as const,
    caseId: "cancellation",
  },
  {
    observation: {
      id: "invalid-value",
      boundary: "result",
      terminal: "defect",
      leakedToApplication: true,
    } as const,
    caseId: "invalid-value",
  },
]) {
  let failure: unknown
  try {
    assertProviderConformanceCase(invalid.observation)
  } catch (error) {
    failure = error
  }
  assert(
    failure instanceof ProviderConformanceFailure &&
      failure.caseId === invalid.caseId,
    `${invalid.caseId} violation must be detected`
  )
}

process.stdout.write("provider conformance profile probe passed\n")
