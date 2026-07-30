import { readFile } from "node:fs/promises"
import { relative, resolve, sep } from "node:path"

export type TourLessonContentReference = Readonly<{
  id: string
  content: string
}>

export type TourLessonMetadata = Readonly<{
  id: string
  challenge: string
  interactive: boolean
  files: Readonly<{
    source: string
    guide: string
    stdin?: string
    expectedOutput?: string
    exercise?: string
    diagnosticExample?: string
  }>
}>

export type LoadedTourLesson = Readonly<{
  directory: string
  metadata: TourLessonMetadata
  source: string
  guide: string
  sourcePath: string
  guidePath: string
  stdinPath?: string
  expectedOutputPath?: string
  exercisePath?: string
  diagnosticExamplePath?: string
}>

export async function loadTourLessons(
  repositoryRoot: string,
  references: readonly TourLessonContentReference[]
): Promise<readonly LoadedTourLesson[]> {
  const lessonsRoot = resolve(repositoryRoot, "examples/tour/lessons")
  return Promise.all(
    references.map(async ({ id, content }) => {
      const descriptorPath = resolve(repositoryRoot, "examples/tour", content)
      const directory = resolve(descriptorPath, "..")
      if (resolve(directory, "..") !== lessonsRoot) {
        throw new Error(`Tour lesson ${id} content escapes the lessons root`)
      }
      const metadata = parseTourLessonMetadata(
        JSON.parse(await readFile(descriptorPath, "utf8")),
        id
      )
      const sourcePath = resolve(directory, metadata.files.source)
      const guidePath = resolve(directory, metadata.files.guide)
      const stdinPath = metadata.files.stdin
        ? resolve(directory, metadata.files.stdin)
        : undefined
      const expectedOutputPath = metadata.files.expectedOutput
        ? resolve(directory, metadata.files.expectedOutput)
        : undefined
      const exercisePath = metadata.files.exercise
        ? resolve(directory, metadata.files.exercise)
        : undefined
      const diagnosticExamplePath = metadata.files.diagnosticExample
        ? resolve(directory, metadata.files.diagnosticExample)
        : undefined
      const [source, guide] = await Promise.all([
        readFile(sourcePath, "utf8"),
        readFile(guidePath, "utf8"),
      ])
      if (!source.includes("//")) {
        throw new Error(`Tour lesson ${id} needs a source comment`)
      }
      if (guide.trim() === "") {
        throw new Error(`Tour lesson ${id} guide is empty`)
      }
      if (stdinPath) await readFile(stdinPath, "utf8")
      if (expectedOutputPath) await readFile(expectedOutputPath, "utf8")
      if (exercisePath) await readFile(exercisePath, "utf8")
      if (diagnosticExamplePath) {
        await readFile(diagnosticExamplePath, "utf8")
      }
      if (!metadata.interactive && expectedOutputPath === undefined) {
        throw new Error(
          `Non-interactive Tour lesson ${id} needs expected output`
        )
      }
      return {
        directory,
        metadata,
        source,
        guide,
        sourcePath,
        guidePath,
        ...(stdinPath ? { stdinPath } : {}),
        ...(expectedOutputPath ? { expectedOutputPath } : {}),
        ...(exercisePath ? { exercisePath } : {}),
        ...(diagnosticExamplePath ? { diagnosticExamplePath } : {}),
      }
    })
  )
}

export function repositoryPath(repositoryRoot: string, file: string): string {
  return relative(repositoryRoot, file).split(sep).join("/")
}

function parseTourLessonMetadata(
  value: unknown,
  directoryId: string
): TourLessonMetadata {
  const record = expectRecord(value, `Tour lesson ${directoryId}`)
  expectKeys(record, ["$schema", "id", "challenge", "interactive", "files"])
  const id = expectString(record.id, `Tour lesson ${directoryId}.id`)
  if (id !== directoryId) {
    throw new Error(`Tour lesson ${directoryId} declares mismatched id ${id}`)
  }
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/u.test(id)) {
    throw new Error(`Tour lesson ${directoryId} has an invalid id`)
  }
  const files = expectRecord(record.files, `Tour lesson ${id}.files`)
  expectKeys(files, [
    "source",
    "guide",
    "stdin",
    "expectedOutput",
    "exercise",
    "diagnosticExample",
  ])
  const source = expectFileName(files.source, `Tour lesson ${id}.files.source`)
  const guide = expectFileName(files.guide, `Tour lesson ${id}.files.guide`)
  const stdin = optionalFileName(files.stdin, `Tour lesson ${id}.files.stdin`)
  const expectedOutput = optionalFileName(
    files.expectedOutput,
    `Tour lesson ${id}.files.expectedOutput`
  )
  const exercise = optionalFileName(
    files.exercise,
    `Tour lesson ${id}.files.exercise`
  )
  const diagnosticExample = optionalFileName(
    files.diagnosticExample,
    `Tour lesson ${id}.files.diagnosticExample`
  )
  return {
    id,
    challenge: expectString(record.challenge, `Tour lesson ${id}.challenge`),
    interactive:
      record.interactive === undefined
        ? false
        : expectBoolean(record.interactive, `Tour lesson ${id}.interactive`),
    files: {
      source,
      guide,
      ...(stdin ? { stdin } : {}),
      ...(expectedOutput ? { expectedOutput } : {}),
      ...(exercise ? { exercise } : {}),
      ...(diagnosticExample ? { diagnosticExample } : {}),
    },
  }
}

function expectRecord(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${label} must be an object`)
  }
  return value as Record<string, unknown>
}

function expectKeys(
  record: Record<string, unknown>,
  allowed: readonly string[]
): void {
  const allowedKeys = new Set(allowed)
  const unknown = Object.keys(record).filter((key) => !allowedKeys.has(key))
  if (unknown.length > 0) {
    throw new Error(`Unknown Tour lesson field(s): ${unknown.join(", ")}`)
  }
}

function expectString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${label} must be a non-empty string`)
  }
  return value
}

function expectBoolean(value: unknown, label: string): boolean {
  if (typeof value !== "boolean") throw new Error(`${label} must be a boolean`)
  return value
}

function expectFileName(value: unknown, label: string): string {
  const file = expectString(value, label)
  if (!/^[a-z0-9][a-z0-9._-]*$/u.test(file)) {
    throw new Error(`${label} must stay inside its lesson directory`)
  }
  return file
}

function optionalFileName(value: unknown, label: string): string | undefined {
  return value === undefined ? undefined : expectFileName(value, label)
}
