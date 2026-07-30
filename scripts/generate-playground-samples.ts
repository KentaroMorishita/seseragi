import { createHash } from "node:crypto"
import { mkdir, readdir, readFile, writeFile } from "node:fs/promises"
import { relative, resolve, sep } from "node:path"
import {
  parseDiscoverGroups,
  parseSampleMetadata,
  type PlaygroundSampleDefinition,
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
  readonly guideImport: string
  readonly stdinImport?: string
  readonly outputImport?: string
}

type LoadedSource = {
  readonly path: string
  readonly sourcePath: string
  readonly sourceHash: string
  readonly source: string
  readonly importName: string
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
    const sourceFiles = metadata.workspace?.files ?? [metadata.files.source]
    const sources = await Promise.all(
      sourceFiles.map(async (path, sourceIndex): Promise<LoadedSource> => {
        const sourcePath = resolve(sampleDirectory, path)
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
      ({ path }) => path === metadata.files.source
    )
    if (entrySource === undefined) {
      throw new Error(`sample ${metadata.id} entry source was not loaded`)
    }

    const stdinPath = metadata.files.stdin
      ? resolve(sampleDirectory, metadata.files.stdin)
      : undefined
    const expectedOutputPath = metadata.files.expectedOutput
      ? resolve(sampleDirectory, metadata.files.expectedOutput)
      : undefined
    if (stdinPath) await readFile(stdinPath, "utf8")
    if (expectedOutputPath) await readFile(expectedOutputPath, "utf8")

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
        prerequisites: metadata.prerequisites,
        featured: metadata.featured,
        isNew: metadata.isNew,
        interactive: metadata.interactive,
        sourcePath: entrySource.sourcePath,
        guidePath: repositoryPath(guidePath),
        ...(stdinPath ? { stdinPath: repositoryPath(stdinPath) } : {}),
        ...(expectedOutputPath
          ? { expectedOutputPath: repositoryPath(expectedOutputPath) }
          : {}),
        sourceHash: entrySource.sourceHash,
        workspaceHash: sourceHash(
          sources.map(({ path, source }) => `${path}\0${source}\0`).join("")
        ),
        ...(metadata.workspace === undefined
          ? {}
          : {
              project: {
                entryFile: metadata.workspace.entry,
                activeFile: metadata.workspace.active,
                openFiles: metadata.workspace.open,
                expandedFolders: metadata.workspace.expanded,
                files: sources.map(({ path, sourcePath, sourceHash }) => ({
                  path,
                  sourcePath,
                  sourceHash,
                })),
              },
            }),
      },
      sources,
      entrySource,
      guideImport: importName(index, "guide"),
      ...(stdinPath ? { stdinImport: importName(index, "stdin") } : {}),
      ...(expectedOutputPath
        ? { outputImport: importName(index, "output") }
        : {}),
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
