// Decides which samples `check:sample` runs and how to run each one.
//
// Only the fields needed to run a sample are read here. The full shape of
// `sample.json` and the catalog belong to `check:sample-catalog`.

/** Changing one of these files reruns every sample. */
export const sampleCheckerFiles = [
  "scripts/check-samples-cli.ts",
  "scripts/sample-cli-process.ts",
  "scripts/sample-selection.ts",
] as const

const samplesPrefix = "examples/samples/"

export type SampleRunPlan = {
  readonly id: string
  readonly interactive: boolean
  readonly source?: string
  readonly manifest?: string
  readonly stdin?: string
  readonly expectedOutput?: string
}

/** Reads what running one sample needs from its `sample.json`. */
export function readSampleRunPlan(value: unknown, id: string): SampleRunPlan {
  const metadata = expectObject(value, `sample ${id}/sample.json`)
  const files = expectObject(metadata.files, `sample ${id}.files`)
  const interactive = metadata.interactive ?? false
  if (typeof interactive !== "boolean") {
    throw new Error(`sample ${id}.interactive must be a boolean`)
  }
  const source = optionalFile(files.source, `sample ${id}.files.source`)
  const manifest = optionalFile(files.manifest, `sample ${id}.files.manifest`)
  if (source === undefined && manifest === undefined) {
    throw new Error(`sample ${id} has neither files.source nor files.manifest`)
  }
  const stdin = optionalFile(files.stdin, `sample ${id}.files.stdin`)
  const expectedOutput = optionalFile(
    files.expectedOutput,
    `sample ${id}.files.expectedOutput`
  )
  return {
    id,
    interactive,
    ...(source === undefined ? {} : { source }),
    ...(manifest === undefined ? {} : { manifest }),
    ...(stdin === undefined ? {} : { stdin }),
    ...(expectedOutput === undefined ? {} : { expectedOutput }),
  }
}

/**
 * Picks the samples touched by `changedPaths`, which are repository-relative
 * paths. Returns every sample when a checker file changed.
 */
export function selectChangedSamples(
  changedPaths: readonly string[],
  sampleIds: readonly string[]
): readonly string[] {
  const normalized = changedPaths.map((path) => path.replaceAll("\\", "/"))
  if (normalized.some((path) => isCheckerFile(path))) return [...sampleIds]
  const changed = new Set<string>()
  for (const path of normalized) {
    if (!path.startsWith(samplesPrefix)) continue
    const id = path.slice(samplesPrefix.length).split("/")[0]
    if (id !== undefined && path.length > samplesPrefix.length + id.length) {
      changed.add(id)
    }
  }
  return sampleIds.filter((id) => changed.has(id))
}

function isCheckerFile(path: string): boolean {
  return (sampleCheckerFiles as readonly string[]).includes(path)
}

type JsonObject = Record<string, unknown>

function expectObject(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${context} must be an object`)
  }
  return value as JsonObject
}

function optionalFile(value: unknown, context: string): string | undefined {
  if (value === undefined) return undefined
  if (typeof value !== "string" || value === "") {
    throw new Error(`${context} must be a file name`)
  }
  if (value.startsWith("/") || value.split(/[\\/]/u).includes("..")) {
    throw new Error(`${context} must stay inside the sample directory`)
  }
  return value
}
