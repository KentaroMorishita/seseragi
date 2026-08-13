import { createHash } from "node:crypto"
import { mkdir, readdir, readFile, writeFile } from "node:fs/promises"
import { relative, resolve, sep } from "node:path"
import {
  validatePreviewSourceReadability,
  validatePreviewUtilityUsage,
} from "../apps/playground/src/preview-utility-contract"
import {
  type PlaygroundSampleDefinition,
  parseDiscoverGroups,
  parseSampleMetadata,
  type SamplePreviewContract,
  validateSampleCatalog,
} from "../apps/playground/src/sample-catalog"

const repositoryRoot = resolve(import.meta.dir, "..")
const samplesRoot = resolve(repositoryRoot, "examples/samples")
const outputPath = resolve(
  repositoryRoot,
  "apps/playground/src/generated/sample-manifest.ts"
)
const checkOnly = process.argv.includes("--check")

type LoadedSample = {
  readonly definition: PlaygroundSampleDefinition
  readonly sources: readonly LoadedSource[]
  readonly entrySource: LoadedSource
  readonly manifest?: LoadedManifest
  readonly guideImport: string
  readonly stdinImport?: string
  readonly outputImport?: string
  readonly expectedOutput?: LoadedOutput
  readonly preview?: SamplePreviewContract
}

type LoadedManifest = {
  readonly path: string
  readonly source: string
  readonly sourceHash: string
  readonly importName: string
  readonly sourceDirectory: string
  readonly entryFile: string
  readonly files: readonly string[]
}

type LoadedSource = {
  readonly path: string
  readonly sourcePath: string
  readonly sourceHash: string
  readonly source: string
  readonly importName: string
}

type LoadedOutput = {
  readonly path: string
  readonly content: string
}

const directoryEntries = await readdir(samplesRoot, { withFileTypes: true })
const sampleDirectories = directoryEntries
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort()

const loadedSamples = await Promise.all(
  sampleDirectories.map(async (directoryId, index): Promise<LoadedSample> => {
    const sampleDirectory = resolve(samplesRoot, directoryId)
    const metadata = parseSampleMetadata(
      JSON.parse(
        await readFile(resolve(sampleDirectory, "sample.json"), "utf8")
      ),
      directoryId
    )
    const guidePath = resolve(sampleDirectory, metadata.files.guide)
    await readFile(guidePath, "utf8")
    const manifestFile = metadata.files.manifest
    const manifest =
      manifestFile === undefined
        ? undefined
        : await loadManifest(sampleDirectory, manifestFile, index)
    const sourceFiles =
      manifest?.files ??
      metadata.workspace?.files ??
      (metadata.files.source === undefined ? [] : [metadata.files.source])
    const sources = await Promise.all(
      sourceFiles.map(async (path, sourceIndex): Promise<LoadedSource> => {
        const sourcePath = resolve(
          sampleDirectory,
          manifest?.sourceDirectory ?? "",
          path
        )
        const source = await readFile(sourcePath, "utf8")
        return {
          path,
          sourcePath: repositoryPath(sourcePath),
          sourceHash: sourceHash(source),
          source,
          importName: importName(
            index,
            sourceIndex === 0 ? "source" : `source${sourceIndex + 1}`
          ),
        }
      })
    )
    const entrySource = sources.find(
      ({ path }) => path === (manifest?.entryFile ?? metadata.files.source)
    )
    if (entrySource === undefined) {
      throw new Error(`sample ${metadata.id} entry source was not loaded`)
    }

    const stdinPath = metadata.files.stdin
      ? resolve(sampleDirectory, metadata.files.stdin)
      : undefined
    const expectedOutputFile = metadata.files.expectedOutput
    const expectedOutputPath = expectedOutputFile
      ? resolve(sampleDirectory, expectedOutputFile)
      : undefined
    if (stdinPath) await readFile(stdinPath, "utf8")
    const expectedOutput = expectedOutputPath
      ? {
          path: expectedOutputFile,
          content: await readFile(expectedOutputPath, "utf8"),
        }
      : undefined

    const project = resolveProject(metadata.workspace, manifest, sources)
    return {
      definition: {
        id: metadata.id,
        title: metadata.title,
        summary: metadata.summary,
        kind: metadata.kind,
        difficulty: metadata.difficulty,
        topics: metadata.topics,
        capabilities: metadata.capabilities,
        outputMode: metadata.outputMode,
        ...(metadata.experience === undefined
          ? {}
          : {
              experience: metadata.experience,
              architecture: metadata.architecture,
              focus: metadata.focus,
              ...(metadata.comparisonSample === undefined
                ? {}
                : { comparisonSample: metadata.comparisonSample }),
            }),
        prerequisites: metadata.prerequisites,
        featured: metadata.featured,
        isNew: metadata.isNew,
        interactive: metadata.interactive,
        sourcePath: entrySource.sourcePath,
        ...(manifest === undefined
          ? {}
          : {
              manifestPath: manifest.path,
              manifestHash: manifest.sourceHash,
            }),
        guidePath: repositoryPath(guidePath),
        ...(stdinPath ? { stdinPath: repositoryPath(stdinPath) } : {}),
        ...(expectedOutputPath
          ? { expectedOutputPath: repositoryPath(expectedOutputPath) }
          : {}),
        sourceHash: entrySource.sourceHash,
        workspaceHash: sourceHash(
          `${manifest === undefined ? "" : `${manifest.source}\0`}${sources
            .map(({ path, source }) => `${path}\0${source}\0`)
            .join("")}`
        ),
        ...(project === undefined ? {} : { project }),
      },
      sources,
      entrySource,
      ...(manifest === undefined ? {} : { manifest }),
      guideImport: importName(index, "guide"),
      ...(stdinPath ? { stdinImport: importName(index, "stdin") } : {}),
      ...(expectedOutputPath
        ? { outputImport: importName(index, "output") }
        : {}),
      ...(expectedOutput === undefined ? {} : { expectedOutput }),
      ...(metadata.preview === undefined ? {} : { preview: metadata.preview }),
    }
  })
)

const discoverGroups = parseDiscoverGroups(
  JSON.parse(
    await readFile(resolve(samplesRoot, "discover-groups.json"), "utf8")
  )
)
validateSampleCatalog(
  loadedSamples.map(({ definition }) => definition),
  discoverGroups
)
validatePreviewUtilityUsage(
  loadedSamples
    .filter(({ definition }) => definition.outputMode === "html")
    .map((sample) => ({
      id: sample.definition.id,
      sources: [
        ...sample.sources.map(({ path, source }) => ({
          path,
          content: source,
          format: "seseragi" as const,
        })),
        ...(sample.expectedOutput === undefined
          ? []
          : [
              {
                path: sample.expectedOutput.path,
                content: sample.expectedOutput.content,
                format: "html" as const,
              },
            ]),
      ],
      customClasses: sample.preview?.customClasses,
      dynamicUtilities: sample.preview?.dynamicUtilities,
    }))
)
validatePreviewSourceReadability(
  loadedSamples
    .filter(({ definition }) => definition.outputMode === "html")
    .map((sample) => ({
      id: sample.definition.id,
      sources: sample.sources.map(({ path, source }) => ({
        path,
        content: source,
        format: "seseragi" as const,
      })),
    }))
)

const generated = renderGeneratedModule(loadedSamples, discoverGroups)
if (checkOnly) {
  const current = await readFile(outputPath, "utf8").catch(() => "")
  if (current !== generated) {
    throw new Error(
      "Playground sample manifest is stale. Run `bun run samples:generate` in apps/playground."
    )
  }
  console.log(`Validated ${loadedSamples.length} Playground samples.`)
} else {
  await mkdir(resolve(outputPath, ".."), { recursive: true })
  await writeFile(outputPath, generated)
  console.log(
    `Generated ${repositoryPath(outputPath)} (${loadedSamples.length} samples).`
  )
}

function renderGeneratedModule(
  samples: readonly LoadedSample[],
  groups: ReturnType<typeof parseDiscoverGroups>
): string {
  const imports: string[] = [
    'import type { DiscoverGroupDefinition, GeneratedSample } from "../sample-catalog"',
    "",
  ]
  for (const [index, sample] of samples.entries()) {
    if (sample.manifest !== undefined) {
      imports.push(
        renderImport(sample.manifest.importName, sample.manifest.path)
      )
    }
    for (const source of sample.sources) {
      imports.push(renderImport(source.importName, source.sourcePath))
    }
    imports.push(renderImport(sample.guideImport, sample.definition.guidePath))
    if (sample.stdinImport && sample.definition.stdinPath) {
      imports.push(
        renderImport(sample.stdinImport, sample.definition.stdinPath)
      )
    }
    if (sample.outputImport && sample.definition.expectedOutputPath) {
      imports.push(
        renderImport(sample.outputImport, sample.definition.expectedOutputPath)
      )
    }
    if (index < samples.length - 1) imports.push("")
  }

  const records = samples.map((sample) => {
    const definition = indent(JSON.stringify(sample.definition, null, 2), 4)
    return [
      "  {",
      `    definition: ${definition.trimStart()},`,
      `    source: ${sample.entrySource.importName},`,
      `    manifest: ${sample.manifest?.importName ?? '""'},`,
      "    projectFiles: [",
      ...sample.sources.map(
        (source) =>
          `      { path: ${JSON.stringify(source.path)}, source: ${source.importName} },`
      ),
      "    ],",
      `    guide: ${sample.guideImport},`,
      `    stdin: ${sample.stdinImport ?? '""'},`,
      `    expectedOutput: (${sample.outputImport ?? '""'}).replace(/\\r?\\n$/u, ""),`,
      "  }",
    ].join("\n")
  })

  return [
    "// Generated by scripts/generate-playground-samples.ts. Do not edit.",
    ...imports,
    "",
    "export const generatedSamples: readonly GeneratedSample[] = [",
    records.join(",\n"),
    "]",
    "",
    "export const generatedDiscoverGroups: readonly DiscoverGroupDefinition[] =",
    `${indent(JSON.stringify(groups, null, 2), 2)}`,
    "",
  ].join("\n")
}

async function loadManifest(
  sampleDirectory: string,
  manifestFile: string,
  sampleIndex: number
): Promise<LoadedManifest> {
  const path = resolve(sampleDirectory, manifestFile)
  const source = await readFile(path, "utf8")
  const parsed = expectRecord(Bun.TOML.parse(source), manifestFile)
  const layout = optionalRecord(parsed.layout, `${manifestFile}.layout`)
  const run = expectRecord(parsed.run, `${manifestFile}.run`)
  const sourceDirectory = packagePath(
    layout?.source ?? "src",
    `${manifestFile}.layout.source`
  )
  const entry = packagePath(run.entry, `${manifestFile}.run.entry`)
  const sourceRoot = resolve(sampleDirectory, sourceDirectory)
  const entryFile = `${entry}.ssrg`
  const discoveredFiles = await discoverSources(sourceRoot)
  const files = [
    entryFile,
    ...discoveredFiles.filter((path) => path !== entryFile),
  ]
  if (!files.includes(entryFile)) {
    throw new Error(`${manifestFile} entry ${entryFile} does not exist`)
  }
  return {
    path: repositoryPath(path),
    source,
    sourceHash: sourceHash(source),
    importName: importName(sampleIndex, "manifest"),
    sourceDirectory,
    entryFile,
    files,
  }
}

async function discoverSources(
  root: string,
  directory = root
): Promise<string[]> {
  const files: string[] = []
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name)
    if (entry.isDirectory()) {
      files.push(...(await discoverSources(root, path)))
    } else if (entry.isFile() && entry.name.endsWith(".ssrg")) {
      files.push(relative(root, path).split(sep).join("/"))
    }
  }
  return files.sort()
}

function resolveProject(
  workspace: ReturnType<typeof parseSampleMetadata>["workspace"],
  manifest: LoadedManifest | undefined,
  sources: readonly LoadedSource[]
): PlaygroundSampleDefinition["project"] {
  if (workspace === undefined && manifest === undefined) return undefined
  const files = sources.map(({ path }) => path)
  const entryFile = manifest?.entryFile ?? workspace?.entry
  if (entryFile === undefined || !files.includes(entryFile)) {
    throw new Error("sample project entry must appear in its source tree")
  }
  const activeFile = workspace?.active ?? entryFile
  const openFiles = workspace?.open.length ? workspace.open : [activeFile]
  for (const path of [activeFile, ...openFiles]) {
    if (!files.includes(path)) {
      throw new Error(`sample project view references missing source ${path}`)
    }
  }
  if (!openFiles.includes(activeFile)) {
    throw new Error("sample project active file must appear in open files")
  }
  const folders = new Set(
    files.flatMap((path) => {
      const segments = path.split("/")
      return segments
        .slice(0, -1)
        .map((_, index) => segments.slice(0, index + 1).join("/"))
    })
  )
  const expandedFolders = workspace?.expanded ?? []
  for (const folder of expandedFolders) {
    if (!folders.has(folder)) {
      throw new Error(`sample project expands missing folder ${folder}`)
    }
  }
  return {
    entryFile,
    activeFile,
    openFiles,
    expandedFolders,
    files: sources.map(({ path, sourcePath, sourceHash }) => ({
      path,
      sourcePath,
      sourceHash,
    })),
  }
}

function expectRecord(
  value: unknown,
  context: string
): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${context} must be a TOML table`)
  }
  return value as Record<string, unknown>
}

function optionalRecord(
  value: unknown,
  context: string
): Record<string, unknown> | undefined {
  return value === undefined ? undefined : expectRecord(value, context)
}

function packagePath(value: unknown, context: string): string {
  if (
    typeof value !== "string" ||
    value === "" ||
    value.startsWith("/") ||
    value.endsWith("/") ||
    value.includes("\\") ||
    value.split("/").some((segment) => !/^[a-z0-9][a-z0-9._-]*$/u.test(segment))
  ) {
    throw new Error(`${context} must be a canonical relative path`)
  }
  return value
}

function renderImport(name: string, repositoryFile: string): string {
  const relativeFile = relative(
    resolve(outputPath, ".."),
    resolve(repositoryRoot, repositoryFile)
  )
    .split(sep)
    .join("/")
  const specifier = relativeFile.startsWith(".")
    ? relativeFile
    : `./${relativeFile}`
  return `import ${name} from ${JSON.stringify(`${specifier}?raw`)}`
}

function importName(index: number, role: string): string {
  return `sample${index}${role[0]?.toUpperCase()}${role.slice(1)}`
}

function repositoryPath(file: string): string {
  return relative(repositoryRoot, file).split(sep).join("/")
}

function sourceHash(source: string): string {
  return `sha256:${createHash("sha256").update(source).digest("hex")}`
}

function indent(value: string, spaces: number): string {
  const prefix = " ".repeat(spaces)
  return value
    .split("\n")
    .map((line, index) => (index === 0 ? line : `${prefix}${line}`))
    .join("\n")
}
