export const sampleKinds = ["lesson", "recipe", "showcase"] as const
export const sampleDifficulties = [
  "beginner",
  "intermediate",
  "advanced",
] as const
export const sampleCapabilities = ["console", "stdin", "dom"] as const
export const sampleOutputModes = ["text", "html"] as const

export type SampleKind = (typeof sampleKinds)[number]
export type SampleDifficulty = (typeof sampleDifficulties)[number]
export type SampleCapability = (typeof sampleCapabilities)[number]
export type SampleOutputMode = (typeof sampleOutputModes)[number]

export type SampleFiles = {
  readonly source: string
  readonly guide: string
  readonly stdin?: string
  readonly expectedOutput?: string
}

export type SampleMetadata = {
  readonly id: string
  readonly title: string
  readonly summary: string
  readonly kind: SampleKind
  readonly difficulty: SampleDifficulty
  readonly topics: readonly string[]
  readonly capabilities: readonly SampleCapability[]
  readonly outputMode: SampleOutputMode
  readonly prerequisites: readonly string[]
  readonly featured: boolean
  readonly isNew: boolean
  readonly interactive: boolean
  readonly files: SampleFiles
}

export type PlaygroundSampleDefinition = Omit<SampleMetadata, "files"> & {
  readonly sourcePath: string
  readonly guidePath: string
  readonly stdinPath?: string
  readonly expectedOutputPath?: string
  readonly sourceHash: string
}

export type GeneratedSample = {
  readonly definition: PlaygroundSampleDefinition
  readonly source: string
  readonly guide: string
  readonly stdin: string
  readonly expectedOutput: string
}

export type DiscoverGroupDefinition = {
  readonly id: string
  readonly title: string
  readonly summary: string
  readonly kind: Exclude<SampleKind, "lesson">
  readonly samples: readonly string[]
}

type JsonObject = Readonly<Record<string, unknown>>

const slugPattern = /^[a-z0-9]+(?:-[a-z0-9]+)*$/u
const fileNamePattern = /^[a-z0-9][a-z0-9._-]*$/u

export function parseSampleMetadata(
  value: unknown,
  directoryId: string
): SampleMetadata {
  const metadata = expectObject(value, `sample ${directoryId}`)
  assertAllowedKeys(
    metadata,
    [
      "$schema",
      "id",
      "title",
      "summary",
      "kind",
      "difficulty",
      "topics",
      "capabilities",
      "outputMode",
      "prerequisites",
      "featured",
      "isNew",
      "interactive",
      "files",
    ],
    `sample ${directoryId}`
  )

  const id = expectSlug(metadata.id, `sample ${directoryId}.id`)
  if (id !== directoryId) {
    throw new Error(`sample directory ${directoryId} does not match id ${id}`)
  }
  const kind = expectEnum(metadata.kind, sampleKinds, `sample ${id}.kind`)
  const difficulty = expectEnum(
    metadata.difficulty,
    sampleDifficulties,
    `sample ${id}.difficulty`
  )
  const topics = expectUniqueStrings(metadata.topics, `sample ${id}.topics`)
  if (topics.length === 0)
    throw new Error(`sample ${id}.topics must not be empty`)
  const capabilities = expectEnumArray(
    metadata.capabilities,
    sampleCapabilities,
    `sample ${id}.capabilities`
  )
  if (capabilities.length === 0) {
    throw new Error(`sample ${id}.capabilities must not be empty`)
  }
  const outputMode = expectEnum(
    metadata.outputMode,
    sampleOutputModes,
    `sample ${id}.outputMode`
  )
  const prerequisites = expectUniqueSlugs(
    metadata.prerequisites,
    `sample ${id}.prerequisites`
  )
  const featured = expectBoolean(metadata.featured, `sample ${id}.featured`)
  const isNew = expectOptionalBoolean(metadata.isNew, `sample ${id}.isNew`)
  const interactive = expectOptionalBoolean(
    metadata.interactive,
    `sample ${id}.interactive`
  )
  const files = parseSampleFiles(metadata.files, id)

  if (interactive && !capabilities.includes("dom")) {
    throw new Error(`interactive sample ${id} must declare the dom capability`)
  }
  if (capabilities.includes("stdin") !== (files.stdin !== undefined)) {
    throw new Error(
      `sample ${id} must declare both the stdin capability and stdin file`
    )
  }
  if (!interactive && files.expectedOutput === undefined) {
    throw new Error(`non-interactive sample ${id} requires expectedOutput`)
  }

  return {
    id,
    title: expectNonEmptyString(metadata.title, `sample ${id}.title`),
    summary: expectNonEmptyString(metadata.summary, `sample ${id}.summary`),
    kind,
    difficulty,
    topics,
    capabilities,
    outputMode,
    prerequisites,
    featured,
    isNew,
    interactive,
    files,
  }
}

export function parseDiscoverGroups(value: unknown): DiscoverGroupDefinition[] {
  const root = expectObject(value, "discover groups")
  assertAllowedKeys(root, ["$schema", "schema", "groups"], "discover groups")
  if (root.schema !== 1) throw new Error("discover groups.schema must be 1")
  if (!Array.isArray(root.groups))
    throw new Error("discover groups.groups must be an array")

  return root.groups.map((rawGroup, index) => {
    const group = expectObject(rawGroup, `discover group ${index}`)
    assertAllowedKeys(
      group,
      ["id", "title", "summary", "kind", "samples"],
      `discover group ${index}`
    )
    const id = expectSlug(group.id, `discover group ${index}.id`)
    return {
      id,
      title: expectNonEmptyString(group.title, `discover group ${id}.title`),
      summary: expectNonEmptyString(
        group.summary,
        `discover group ${id}.summary`
      ),
      kind: expectEnum(
        group.kind,
        ["recipe", "showcase"] as const,
        `discover group ${id}.kind`
      ),
      samples: expectUniqueSlugs(group.samples, `discover group ${id}.samples`),
    }
  })
}

export function validateSampleCatalog(
  samples: readonly Pick<SampleMetadata, "id" | "kind" | "prerequisites">[],
  discoverGroups: readonly DiscoverGroupDefinition[]
): void {
  const byId = new Map<
    string,
    Pick<SampleMetadata, "id" | "kind" | "prerequisites">
  >()
  for (const sample of samples) {
    if (byId.has(sample.id))
      throw new Error(`duplicate sample id: ${sample.id}`)
    byId.set(sample.id, sample)
  }

  for (const sample of samples) {
    for (const prerequisite of sample.prerequisites) {
      if (!byId.has(prerequisite)) {
        throw new Error(
          `sample ${sample.id} references missing prerequisite ${prerequisite}`
        )
      }
    }
  }
  assertAcyclicPrerequisites(byId)

  const groupIds = new Set<string>()
  const sampleGroups = new Map<string, string>()
  for (const group of discoverGroups) {
    if (groupIds.has(group.id))
      throw new Error(`duplicate discover group id: ${group.id}`)
    groupIds.add(group.id)
    if (group.samples.length === 0) {
      throw new Error(`discover group ${group.id} must not be empty`)
    }
    for (const sampleId of group.samples) {
      const sample = byId.get(sampleId)
      if (!sample) {
        throw new Error(
          `discover group ${group.id} references missing sample ${sampleId}`
        )
      }
      if (sample.kind !== group.kind) {
        throw new Error(
          `discover group ${group.id} requires ${group.kind} samples, but ${sampleId} is ${sample.kind}`
        )
      }
      const existingGroup = sampleGroups.get(sampleId)
      if (existingGroup) {
        throw new Error(
          `sample ${sampleId} appears in multiple discover groups: ${existingGroup}, ${group.id}`
        )
      }
      sampleGroups.set(sampleId, group.id)
    }
  }

  for (const sample of samples) {
    if (sample.kind === "lesson") continue
    if (!sampleGroups.has(sample.id)) {
      throw new Error(
        `${sample.kind} sample ${sample.id} is missing from discover groups`
      )
    }
  }
}

function parseSampleFiles(value: unknown, id: string): SampleFiles {
  const files = expectObject(value, `sample ${id}.files`)
  assertAllowedKeys(
    files,
    ["source", "guide", "stdin", "expectedOutput"],
    `sample ${id}.files`
  )
  return {
    source: expectFileName(files.source, `sample ${id}.files.source`),
    guide: expectFileName(files.guide, `sample ${id}.files.guide`),
    ...(files.stdin === undefined
      ? {}
      : { stdin: expectFileName(files.stdin, `sample ${id}.files.stdin`) }),
    ...(files.expectedOutput === undefined
      ? {}
      : {
          expectedOutput: expectFileName(
            files.expectedOutput,
            `sample ${id}.files.expectedOutput`
          ),
        }),
  }
}

function assertAcyclicPrerequisites(
  byId: ReadonlyMap<
    string,
    Pick<SampleMetadata, "id" | "kind" | "prerequisites">
  >
): void {
  const visiting = new Set<string>()
  const visited = new Set<string>()

  const visit = (id: string, trail: readonly string[]): void => {
    if (visited.has(id)) return
    if (visiting.has(id)) {
      throw new Error(
        `sample prerequisite cycle: ${[...trail, id].join(" -> ")}`
      )
    }
    visiting.add(id)
    const sample = byId.get(id)
    if (!sample) return
    for (const prerequisite of sample.prerequisites) {
      visit(prerequisite, [...trail, id])
    }
    visiting.delete(id)
    visited.add(id)
  }

  for (const id of byId.keys()) visit(id, [])
}

function expectObject(value: unknown, context: string): JsonObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${context} must be an object`)
  }
  return value as JsonObject
}

function expectNonEmptyString(value: unknown, context: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${context} must be a non-empty string`)
  }
  return value
}

function expectBoolean(value: unknown, context: string): boolean {
  if (typeof value !== "boolean")
    throw new Error(`${context} must be a boolean`)
  return value
}

function expectOptionalBoolean(value: unknown, context: string): boolean {
  return value === undefined ? false : expectBoolean(value, context)
}

function expectSlug(value: unknown, context: string): string {
  const slug = expectNonEmptyString(value, context)
  if (!slugPattern.test(slug))
    throw new Error(`${context} must be a stable slug`)
  return slug
}

function expectFileName(value: unknown, context: string): string {
  const fileName = expectNonEmptyString(value, context)
  if (!fileNamePattern.test(fileName)) {
    throw new Error(`${context} must be a file in the sample directory`)
  }
  return fileName
}

function expectUniqueStrings(value: unknown, context: string): string[] {
  if (!Array.isArray(value)) throw new Error(`${context} must be an array`)
  const strings = value.map((item, index) =>
    expectNonEmptyString(item, `${context}[${index}]`)
  )
  if (new Set(strings).size !== strings.length) {
    throw new Error(`${context} must not contain duplicates`)
  }
  return strings
}

function expectUniqueSlugs(value: unknown, context: string): string[] {
  const strings = expectUniqueStrings(value, context)
  for (const [index, string] of strings.entries()) {
    expectSlug(string, `${context}[${index}]`)
  }
  return strings
}

function expectEnum<const Value extends string>(
  value: unknown,
  allowed: readonly Value[],
  context: string
): Value {
  if (typeof value !== "string" || !allowed.includes(value as Value)) {
    throw new Error(`${context} must be one of: ${allowed.join(", ")}`)
  }
  return value as Value
}

function expectEnumArray<const Value extends string>(
  value: unknown,
  allowed: readonly Value[],
  context: string
): Value[] {
  const values = expectUniqueStrings(value, context)
  return values.map((item, index) =>
    expectEnum(item, allowed, `${context}[${index}]`)
  )
}

function assertAllowedKeys(
  value: JsonObject,
  allowed: readonly string[],
  context: string
): void {
  const allowedKeys = new Set(allowed)
  const unknown = Object.keys(value).filter((key) => !allowedKeys.has(key))
  if (unknown.length > 0) {
    throw new Error(`${context} has unknown field(s): ${unknown.join(", ")}`)
  }
}
