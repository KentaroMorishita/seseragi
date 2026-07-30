import { readFile } from "node:fs/promises"
import { relative, resolve, sep } from "node:path"
import type {
  TourIntroducedSurface,
  TourLessonFormat,
  TourSourceRange,
  TourWalkthroughStep,
} from "../apps/playground/src/tour/content"

export type TourLessonContentReference = Readonly<{
  id: string
  content: string
}>

export type TourLessonMetadata = Readonly<{
  id: string
  challenge?: string
  interactive: boolean
  formatVersion?: 2
  format?: TourLessonFormat
  files: Readonly<{
    source: string
    guide?: string
    stdin?: string
    expectedOutput?: string
    exercise?: string
    exerciseExpectedOutput?: string
    diagnosticExample?: string
    diagnosticOutput?: string
  }>
}>

export type LoadedTourLesson = Readonly<{
  directory: string
  metadata: TourLessonMetadata
  source: string
  guide: string
  exerciseSource: string
  exerciseExpectedOutput: string
  diagnosticSource: string
  diagnosticOutput: string
  sourcePath: string
  guidePath?: string
  stdinPath?: string
  expectedOutputPath?: string
  exercisePath?: string
  exerciseExpectedOutputPath?: string
  diagnosticExamplePath?: string
  diagnosticOutputPath?: string
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
      const guidePath = resolveOptionalFile(directory, metadata.files.guide)
      const stdinPath = resolveOptionalFile(directory, metadata.files.stdin)
      const expectedOutputPath = resolveOptionalFile(
        directory,
        metadata.files.expectedOutput
      )
      const exercisePath = resolveOptionalFile(
        directory,
        metadata.files.exercise
      )
      const exerciseExpectedOutputPath = resolveOptionalFile(
        directory,
        metadata.files.exerciseExpectedOutput
      )
      const diagnosticExamplePath = resolveOptionalFile(
        directory,
        metadata.files.diagnosticExample
      )
      const diagnosticOutputPath = resolveOptionalFile(
        directory,
        metadata.files.diagnosticOutput
      )
      const [
        source,
        guideFile,
        exerciseSource,
        exerciseExpectedOutput,
        diagnosticSource,
        diagnosticOutput,
      ] = await Promise.all([
        readFile(sourcePath, "utf8"),
        readOptionalFile(guidePath),
        readOptionalFile(exercisePath),
        readOptionalFile(exerciseExpectedOutputPath),
        readOptionalFile(diagnosticExamplePath),
        readOptionalFile(diagnosticOutputPath),
      ])
      if (metadata.format === undefined) {
        if (!source.includes("//")) {
          throw new Error(`Legacy Tour lesson ${id} needs a source comment`)
        }
        if (guideFile.trim() === "") {
          throw new Error(`Legacy Tour lesson ${id} guide is empty`)
        }
      } else {
        validateTourLessonFormat(id, metadata.format, source)
      }
      if (stdinPath) await readFile(stdinPath, "utf8")
      if (!metadata.interactive && expectedOutputPath === undefined) {
        throw new Error(
          `Non-interactive Tour lesson ${id} needs expected output`
        )
      }
      return {
        directory,
        metadata,
        source,
        guide:
          metadata.format === undefined
            ? guideFile
            : lessonFormatText(metadata.format),
        exerciseSource,
        exerciseExpectedOutput,
        diagnosticSource,
        diagnosticOutput,
        sourcePath,
        ...(guidePath ? { guidePath } : {}),
        ...(stdinPath ? { stdinPath } : {}),
        ...(expectedOutputPath ? { expectedOutputPath } : {}),
        ...(exercisePath ? { exercisePath } : {}),
        ...(exerciseExpectedOutputPath ? { exerciseExpectedOutputPath } : {}),
        ...(diagnosticExamplePath ? { diagnosticExamplePath } : {}),
        ...(diagnosticOutputPath ? { diagnosticOutputPath } : {}),
      }
    })
  )
}

export function repositoryPath(repositoryRoot: string, file: string): string {
  return relative(repositoryRoot, file).split(sep).join("/")
}

export function parseTourLessonMetadata(
  value: unknown,
  directoryId: string
): TourLessonMetadata {
  const record = expectRecord(value, `Tour lesson ${directoryId}`)
  expectKeys(record, [
    "$schema",
    "id",
    "challenge",
    "interactive",
    "formatVersion",
    "sections",
    "files",
  ])
  const id = expectString(record.id, `Tour lesson ${directoryId}.id`)
  if (id !== directoryId) {
    throw new Error(`Tour lesson ${directoryId} declares mismatched id ${id}`)
  }
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/u.test(id)) {
    throw new Error(`Tour lesson ${directoryId} has an invalid id`)
  }
  const interactive =
    record.interactive === undefined
      ? false
      : expectBoolean(record.interactive, `Tour lesson ${id}.interactive`)
  const files = parseFiles(record.files, id)
  if (record.formatVersion === undefined) {
    if (record.sections !== undefined) {
      throw new Error(`Legacy Tour lesson ${id} cannot declare sections`)
    }
    if (files.guide === undefined) {
      throw new Error(`Legacy Tour lesson ${id} needs files.guide`)
    }
    return {
      id,
      challenge: expectString(record.challenge, `Tour lesson ${id}.challenge`),
      interactive,
      files,
    }
  }
  if (record.formatVersion !== 2) {
    throw new Error(`Tour lesson ${id}.formatVersion must be 2`)
  }
  if (record.challenge !== undefined || files.guide !== undefined) {
    throw new Error(
      `Structured Tour lesson ${id} keeps explanation in sections, not challenge or guide`
    )
  }
  for (const [name, file] of [
    ["exercise", files.exercise],
    ["exerciseExpectedOutput", files.exerciseExpectedOutput],
    ["diagnosticExample", files.diagnosticExample],
    ["diagnosticOutput", files.diagnosticOutput],
  ] as const) {
    if (file === undefined) {
      throw new Error(`Structured Tour lesson ${id} needs files.${name}`)
    }
  }
  if (!interactive && files.expectedOutput === undefined) {
    throw new Error(
      `Non-interactive structured Tour lesson ${id} needs files.expectedOutput`
    )
  }
  return {
    id,
    interactive,
    formatVersion: 2,
    format: parseLessonFormat(record.sections, id),
    files,
  }
}

export function validateTourLessonFormat(
  lessonId: string,
  format: TourLessonFormat,
  source: string
): void {
  const lineCount = source.split(/\r?\n/u).length
  for (const [index, step] of format.walkthrough.entries()) {
    const { startLine, endLine } = step.sourceRange
    if (endLine < startLine) {
      throw new Error(
        `Tour lesson ${lessonId}.walkthrough.${index} source range ends before it starts`
      )
    }
    if (endLine > lineCount) {
      throw new Error(
        `Tour lesson ${lessonId}.walkthrough.${index} references line ${endLine}, but source has ${lineCount}`
      )
    }
  }
  const introduced = new Set<string>()
  for (const surface of format.introduced) {
    if (introduced.has(surface.name)) {
      throw new Error(
        `Tour lesson ${lessonId} introduces ${surface.name} more than once`
      )
    }
    introduced.add(surface.name)
  }
  if (format.next.lessonId === lessonId) {
    throw new Error(`Tour lesson ${lessonId} cannot connect to itself`)
  }
}

function parseFiles(value: unknown, id: string): TourLessonMetadata["files"] {
  const files = expectRecord(value, `Tour lesson ${id}.files`)
  expectKeys(files, [
    "source",
    "guide",
    "stdin",
    "expectedOutput",
    "exercise",
    "exerciseExpectedOutput",
    "diagnosticExample",
    "diagnosticOutput",
  ])
  const source = expectFileName(files.source, `Tour lesson ${id}.files.source`)
  const guide = optionalFileName(files.guide, `Tour lesson ${id}.files.guide`)
  const stdin = optionalFileName(files.stdin, `Tour lesson ${id}.files.stdin`)
  const expectedOutput = optionalFileName(
    files.expectedOutput,
    `Tour lesson ${id}.files.expectedOutput`
  )
  const exercise = optionalFileName(
    files.exercise,
    `Tour lesson ${id}.files.exercise`
  )
  const exerciseExpectedOutput = optionalFileName(
    files.exerciseExpectedOutput,
    `Tour lesson ${id}.files.exerciseExpectedOutput`
  )
  const diagnosticExample = optionalFileName(
    files.diagnosticExample,
    `Tour lesson ${id}.files.diagnosticExample`
  )
  const diagnosticOutput = optionalFileName(
    files.diagnosticOutput,
    `Tour lesson ${id}.files.diagnosticOutput`
  )
  return {
    source,
    ...(guide ? { guide } : {}),
    ...(stdin ? { stdin } : {}),
    ...(expectedOutput ? { expectedOutput } : {}),
    ...(exercise ? { exercise } : {}),
    ...(exerciseExpectedOutput ? { exerciseExpectedOutput } : {}),
    ...(diagnosticExample ? { diagnosticExample } : {}),
    ...(diagnosticOutput ? { diagnosticOutput } : {}),
  }
}

function parseLessonFormat(value: unknown, id: string): TourLessonFormat {
  const label = `Tour lesson ${id}.sections`
  const sections = expectRecord(value, label)
  expectKeys(sections, [
    "prerequisite",
    "walkthrough",
    "introduced",
    "exercise",
    "diagnostic",
    "recap",
    "next",
    "notes",
  ])
  const exercise = expectRecord(sections.exercise, `${label}.exercise`)
  expectKeys(exercise, ["instruction", "reset"])
  if (exercise.reset !== "restore-lesson-source") {
    throw new Error(`${label}.exercise.reset must be restore-lesson-source`)
  }
  const diagnostic = expectRecord(sections.diagnostic, `${label}.diagnostic`)
  expectKeys(diagnostic, ["heading", "body"])
  const next = expectRecord(sections.next, `${label}.next`)
  expectKeys(next, ["lessonId", "body"])
  const lessonId =
    next.lessonId === null
      ? null
      : expectLessonId(next.lessonId, `${label}.next.lessonId`)
  const notes =
    sections.notes === undefined
      ? undefined
      : expectStrings(sections.notes, `${label}.notes`, false)
  return {
    prerequisite: expectString(sections.prerequisite, `${label}.prerequisite`),
    walkthrough: expectArray(
      sections.walkthrough,
      `${label}.walkthrough`,
      false
    ).map((step, index) =>
      parseWalkthroughStep(step, `${label}.walkthrough.${index}`)
    ),
    introduced: expectArray(
      sections.introduced,
      `${label}.introduced`,
      false
    ).map((surface, index) =>
      parseIntroducedSurface(surface, `${label}.introduced.${index}`)
    ),
    exercise: {
      instruction: expectString(
        exercise.instruction,
        `${label}.exercise.instruction`
      ),
      reset: "restore-lesson-source",
    },
    diagnostic: {
      heading: expectString(diagnostic.heading, `${label}.diagnostic.heading`),
      body: expectString(diagnostic.body, `${label}.diagnostic.body`),
    },
    recap: expectStrings(sections.recap, `${label}.recap`, false),
    next: {
      lessonId,
      body: expectString(next.body, `${label}.next.body`),
    },
    ...(notes ? { notes } : {}),
  }
}

function parseWalkthroughStep(
  value: unknown,
  label: string
): TourWalkthroughStep {
  const step = expectRecord(value, label)
  expectKeys(step, ["heading", "body", "sourceRange"])
  return {
    heading: expectString(step.heading, `${label}.heading`),
    body: expectString(step.body, `${label}.body`),
    sourceRange: parseSourceRange(step.sourceRange, `${label}.sourceRange`),
  }
}

function parseSourceRange(value: unknown, label: string): TourSourceRange {
  const range = expectRecord(value, label)
  expectKeys(range, ["startLine", "endLine"])
  return {
    startLine: expectPositiveInteger(range.startLine, `${label}.startLine`),
    endLine: expectPositiveInteger(range.endLine, `${label}.endLine`),
  }
}

function parseIntroducedSurface(
  value: unknown,
  label: string
): TourIntroducedSurface {
  const surface = expectRecord(value, label)
  expectKeys(surface, ["kind", "name", "body"])
  const kind = expectString(surface.kind, `${label}.kind`)
  if (!["syntax", "type", "api"].includes(kind)) {
    throw new Error(`${label}.kind must be syntax, type or api`)
  }
  return {
    kind: kind as TourIntroducedSurface["kind"],
    name: expectString(surface.name, `${label}.name`),
    body: expectString(surface.body, `${label}.body`),
  }
}

function lessonFormatText(format: TourLessonFormat): string {
  return [
    format.prerequisite,
    ...format.walkthrough.flatMap(({ heading, body }) => [heading, body]),
    ...format.introduced.flatMap(({ name, body }) => [name, body]),
    format.exercise.instruction,
    format.diagnostic.heading,
    format.diagnostic.body,
    ...format.recap,
    format.next.body,
    ...(format.notes ?? []),
  ].join("\n")
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

function expectArray(
  value: unknown,
  label: string,
  allowEmpty = true
): readonly unknown[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`)
  if (!allowEmpty && value.length === 0) {
    throw new Error(`${label} must not be empty`)
  }
  return value
}

function expectStrings(
  value: unknown,
  label: string,
  allowEmpty = true
): readonly string[] {
  return expectArray(value, label, allowEmpty).map((item, index) =>
    expectString(item, `${label}.${index}`)
  )
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

function expectPositiveInteger(value: unknown, label: string): number {
  if (!Number.isInteger(value) || (value as number) < 1) {
    throw new Error(`${label} must be a positive integer`)
  }
  return value as number
}

function expectLessonId(value: unknown, label: string): string {
  const id = expectString(value, label)
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/u.test(id)) {
    throw new Error(`${label} must be a stable lesson id`)
  }
  return id
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

function resolveOptionalFile(
  directory: string,
  file: string | undefined
): string | undefined {
  return file === undefined ? undefined : resolve(directory, file)
}

async function readOptionalFile(file: string | undefined): Promise<string> {
  return file === undefined ? "" : readFile(file, "utf8")
}
