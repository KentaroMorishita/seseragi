export const sampleKinds = ["lesson", "recipe", "showcase"] as const
export const sampleDifficulties = [
  "beginner",
  "intermediate",
  "advanced",
] as const
export const sampleCapabilities = ["console", "stdin", "dom"] as const
export const sampleOutputModes = ["text", "html"] as const
export const sampleExperiences = ["minimal", "guided", "showcase"] as const
export const sampleArchitectures = [
  "static",
  "dom-app",
  "signal-run",
  "signal-mount",
  "multi-module",
] as const
export const sampleFocuses = [
  "component",
  "state",
  "form",
  "event",
  "composition",
  "project",
] as const
export const maximumFeaturedSamples = 8

export type SampleKind = (typeof sampleKinds)[number]
export type SampleDifficulty = (typeof sampleDifficulties)[number]
export type SampleCapability = (typeof sampleCapabilities)[number]
export type SampleOutputMode = (typeof sampleOutputModes)[number]
export type SampleExperience = (typeof sampleExperiences)[number]
export type SampleArchitecture = (typeof sampleArchitectures)[number]
export type SampleFocus = (typeof sampleFocuses)[number]

export type SampleFiles = {
  readonly source: string
  readonly guide: string
  readonly stdin?: string
  readonly expectedOutput?: string
}

export type SampleWorkspace = {
  readonly entry: string
  readonly files: readonly string[]
  readonly active: string
  readonly open: readonly string[]
  readonly expanded: readonly string[]
}

export type SamplePreviewContract = {
  readonly customClasses: readonly string[]
  readonly dynamicUtilities: readonly string[]
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
  readonly experience?: SampleExperience
  readonly architecture?: SampleArchitecture
  readonly focus?: SampleFocus
  readonly comparisonSample?: string
  readonly prerequisites: readonly string[]
  readonly featured: boolean
  readonly isNew: boolean
  readonly interactive: boolean
  readonly files: SampleFiles
  readonly workspace?: SampleWorkspace
  readonly preview?: SamplePreviewContract
}

export type PlaygroundSampleProjectFile = {
  readonly path: string
  readonly sourcePath: string
  readonly sourceHash: string
}

export type PlaygroundSampleProject = {
  readonly entryFile: string
  readonly activeFile: string
  readonly openFiles: readonly string[]
  readonly expandedFolders: readonly string[]
  readonly files: readonly PlaygroundSampleProjectFile[]
}

export type PlaygroundSampleDefinition = Omit<
  SampleMetadata,
  "files" | "preview" | "workspace"
> & {
  readonly sourcePath: string
  readonly guidePath: string
  readonly stdinPath?: string
  readonly expectedOutputPath?: string
  readonly sourceHash: string
  readonly workspaceHash: string
  readonly project?: PlaygroundSampleProject
}

export type GeneratedSampleProjectFile = {
  readonly path: string
  readonly source: string
}

export type GeneratedSample = {
  readonly definition: PlaygroundSampleDefinition
  readonly source: string
  readonly projectFiles: readonly GeneratedSampleProjectFile[]
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
type CatalogSample = Pick<SampleMetadata, "id" | "kind" | "prerequisites"> &
  Partial<
    Pick<
      SampleMetadata,
      "experience" | "architecture" | "focus" | "comparisonSample" | "featured"
    >
  >

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
      "experience",
      "architecture",
      "focus",
      "comparisonSample",
      "prerequisites",
      "featured",
      "isNew",
      "interactive",
      "files",
      "workspace",
      "preview",
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
  const workspace =
    metadata.workspace === undefined
      ? undefined
      : parseSampleWorkspace(metadata.workspace, id, files.source)
  const preview =
    metadata.preview === undefined
      ? undefined
      : parseSamplePreviewContract(metadata.preview, id)
  const webCatalog = parseWebCatalogClassification(
    metadata,
    id,
    outputMode,
    interactive,
    workspace !== undefined
  )
  const comparisonSample =
    metadata.comparisonSample === undefined
      ? undefined
      : expectSlug(metadata.comparisonSample, `sample ${id}.comparisonSample`)

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
    ...webCatalog,
    ...(comparisonSample === undefined ? {} : { comparisonSample }),
    prerequisites,
    featured,
    isNew,
    interactive,
    files,
    ...(workspace === undefined ? {} : { workspace }),
    ...(preview === undefined ? {} : { preview }),
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
  samples: readonly CatalogSample[],
  discoverGroups: readonly DiscoverGroupDefinition[]
): void {
  const byId = new Map<string, CatalogSample>()
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

  const featured = samples.filter((sample) => sample.featured)
  const featuredLesson = featured.find((sample) => sample.kind === "lesson")
  if (featuredLesson !== undefined) {
    throw new Error(
      `Tour lesson ${featuredLesson.id} cannot be featured in Discover`
    )
  }
  if (featured.length > maximumFeaturedSamples) {
    throw new Error(
      `Discover supports at most ${maximumFeaturedSamples} featured samples`
    )
  }

  validateWebCatalogCoverage(samples)
  validateComparisonSamples(samples, byId)
}

function parseWebCatalogClassification(
  metadata: JsonObject,
  id: string,
  outputMode: SampleOutputMode,
  interactive: boolean,
  hasWorkspace: boolean
) {
  const values = [metadata.experience, metadata.architecture, metadata.focus]
  if (outputMode !== "html") {
    if (values.some((value) => value !== undefined)) {
      throw new Error(
        `non-HTML sample ${id} must not declare Web catalog classification`
      )
    }
    return {}
  }
  if (values.some((value) => value === undefined)) {
    throw new Error(
      `HTML sample ${id} requires experience, architecture and focus`
    )
  }

  const experience = expectEnum(
    metadata.experience,
    sampleExperiences,
    `sample ${id}.experience`
  )
  const architecture = expectEnum(
    metadata.architecture,
    sampleArchitectures,
    `sample ${id}.architecture`
  )
  const focus = expectEnum(metadata.focus, sampleFocuses, `sample ${id}.focus`)

  if (architecture === "static" && interactive) {
    throw new Error(`static HTML sample ${id} must not be interactive`)
  }
  if (architecture !== "static" && !interactive) {
    throw new Error(`${architecture} HTML sample ${id} must be interactive`)
  }
  if (architecture === "multi-module" && !hasWorkspace) {
    throw new Error(`multi-module sample ${id} requires a workspace`)
  }

  return { experience, architecture, focus }
}

function validateWebCatalogCoverage(samples: readonly CatalogSample[]): void {
  const webSamples = samples.filter(
    (sample) =>
      sample.experience !== undefined &&
      sample.architecture !== undefined &&
      sample.focus !== undefined
  )
  if (webSamples.length === 0) return

  const requiredRoles = [
    {
      label: "minimal static HTML/component",
      matches: (sample: (typeof webSamples)[number]) =>
        sample.experience === "minimal" && sample.architecture === "static",
    },
    {
      label: "minimal dom.app",
      matches: (sample: (typeof webSamples)[number]) =>
        sample.experience === "minimal" && sample.architecture === "dom-app",
    },
    {
      label: "explicit Signal + dom.run",
      matches: (sample: (typeof webSamples)[number]) =>
        sample.architecture === "signal-run",
    },
    {
      label: "advanced single-file interactive Showcase",
      matches: (sample: (typeof webSamples)[number]) =>
        sample.experience === "showcase" &&
        sample.architecture !== "multi-module",
    },
    {
      label: "multi-module application Showcase",
      matches: (sample: (typeof webSamples)[number]) =>
        sample.architecture === "multi-module" && sample.focus === "project",
    },
  ]
  for (const role of requiredRoles) {
    if (!webSamples.some(role.matches)) {
      throw new Error(`Web sample catalog is missing ${role.label}`)
    }
  }
}

function validateComparisonSamples(
  samples: readonly CatalogSample[],
  byId: ReadonlyMap<string, CatalogSample>
): void {
  for (const sample of samples) {
    if (sample.comparisonSample === undefined) continue
    if (sample.comparisonSample === sample.id) {
      throw new Error(`sample ${sample.id} cannot compare with itself`)
    }
    const comparison = byId.get(sample.comparisonSample)
    if (comparison === undefined) {
      throw new Error(
        `sample ${sample.id} comparison references unknown sample ${sample.comparisonSample}`
      )
    }
    if (comparison.comparisonSample !== sample.id) {
      throw new Error(
        `sample ${sample.id} comparison with ${comparison.id} must be reciprocal`
      )
    }
    if (
      sample.experience !== comparison.experience ||
      sample.focus !== comparison.focus ||
      sample.architecture === comparison.architecture
    ) {
      throw new Error(
        `sample ${sample.id} comparison must share experience and focus but use another architecture`
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

function parseSampleWorkspace(
  value: unknown,
  id: string,
  source: string
): SampleWorkspace {
  const workspace = expectObject(value, `sample ${id}.workspace`)
  assertAllowedKeys(
    workspace,
    ["entry", "files", "active", "open", "expanded"],
    `sample ${id}.workspace`
  )
  const entry = expectWorkspaceFile(
    workspace.entry,
    `sample ${id}.workspace.entry`
  )
  const files = expectUniqueWorkspaceFiles(
    workspace.files,
    `sample ${id}.workspace.files`
  )
  if (files.length < 2) {
    throw new Error(`sample ${id}.workspace.files must contain multiple files`)
  }
  if (entry !== source) {
    throw new Error(
      `sample ${id}.workspace.entry must match files.source ${source}`
    )
  }
  if (!files.includes(entry)) {
    throw new Error(`sample ${id}.workspace.entry must appear in files`)
  }

  const active =
    workspace.active === undefined
      ? entry
      : expectWorkspaceFile(workspace.active, `sample ${id}.workspace.active`)
  const open =
    workspace.open === undefined
      ? [entry]
      : expectUniqueWorkspaceFiles(
          workspace.open,
          `sample ${id}.workspace.open`
        )
  if (!files.includes(active)) {
    throw new Error(`sample ${id}.workspace.active must appear in files`)
  }
  if (!open.includes(active)) {
    throw new Error(`sample ${id}.workspace.active must appear in open`)
  }
  for (const path of open) {
    if (!files.includes(path)) {
      throw new Error(`sample ${id}.workspace.open must only reference files`)
    }
  }

  const folders = new Set(files.flatMap((path) => workspaceAncestors(path)))
  const expanded =
    workspace.expanded === undefined
      ? []
      : expectUniqueWorkspacePaths(
          workspace.expanded,
          `sample ${id}.workspace.expanded`
        )
  for (const path of expanded) {
    if (!folders.has(path)) {
      throw new Error(
        `sample ${id}.workspace.expanded must only reference folders`
      )
    }
  }
  return { entry, files, active, open, expanded }
}

function parseSamplePreviewContract(
  value: unknown,
  id: string
): SamplePreviewContract {
  const preview = expectObject(value, `sample ${id}.preview`)
  assertAllowedKeys(
    preview,
    ["customClasses", "dynamicUtilities"],
    `sample ${id}.preview`
  )
  const customClasses = expectUniqueClassTokens(
    preview.customClasses ?? [],
    `sample ${id}.preview.customClasses`
  )
  const dynamicUtilities = expectUniqueClassTokens(
    preview.dynamicUtilities ?? [],
    `sample ${id}.preview.dynamicUtilities`
  )
  const overlap = customClasses.find((token) =>
    dynamicUtilities.includes(token)
  )
  if (overlap !== undefined) {
    throw new Error(
      `sample ${id}.preview class ${overlap} cannot be both custom and utility`
    )
  }
  return { customClasses, dynamicUtilities }
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

function expectWorkspacePath(value: unknown, context: string): string {
  const path = expectNonEmptyString(value, context)
  if (
    path.startsWith("/") ||
    path.endsWith("/") ||
    path.includes("\\") ||
    path.includes("//") ||
    path.includes("\0") ||
    path.split("/").some((segment) => !fileNamePattern.test(segment))
  ) {
    throw new Error(`${context} must be a relative sample path`)
  }
  return path
}

function expectWorkspaceFile(value: unknown, context: string): string {
  const path = expectWorkspacePath(value, context)
  if (!path.endsWith(".ssrg")) {
    throw new Error(`${context} must be a Seseragi source file`)
  }
  return path
}

function expectUniqueWorkspaceFiles(value: unknown, context: string): string[] {
  if (!Array.isArray(value)) throw new Error(`${context} must be an array`)
  const paths = value.map((item, index) =>
    expectWorkspaceFile(item, `${context}[${index}]`)
  )
  assertUniqueStrings(paths, context)
  return paths
}

function expectUniqueWorkspacePaths(value: unknown, context: string): string[] {
  if (!Array.isArray(value)) throw new Error(`${context} must be an array`)
  const paths = value.map((item, index) =>
    expectWorkspacePath(item, `${context}[${index}]`)
  )
  assertUniqueStrings(paths, context)
  return paths
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

function expectUniqueClassTokens(value: unknown, context: string): string[] {
  const tokens = expectUniqueStrings(value, context)
  for (const token of tokens) {
    if (/\s/u.test(token)) {
      throw new Error(`${context} must contain individual class tokens`)
    }
  }
  return tokens
}

function assertUniqueStrings(
  strings: readonly string[],
  context: string
): void {
  if (new Set(strings).size !== strings.length) {
    throw new Error(`${context} must not contain duplicates`)
  }
}

function workspaceAncestors(path: string): string[] {
  const segments = path.split("/")
  return segments
    .slice(1)
    .map((_, index) => segments.slice(0, index + 1).join("/"))
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
