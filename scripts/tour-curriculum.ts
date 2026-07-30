import { readdir, readFile } from "node:fs/promises"
import { resolve } from "node:path"
import {
  parseSampleMetadata,
  type SampleKind,
} from "../apps/playground/src/sample-catalog"
import { type LoadedTourLesson, loadTourLessons } from "./tour-lessons"

export type CurriculumCategory = Readonly<{
  id: string
  order: number
  title: string
  summary: string
  chapters: readonly CurriculumChapter[]
}>

export type CurriculumChapter = Readonly<{
  id: string
  order: number
  title: string
  summary: string
  lessons: readonly CurriculumLesson[]
}>

export type CurriculumLesson = Readonly<{
  id: string
  order: number
  title: string
  summary: string
  goal: string
  focus: readonly string[]
  introducedSurfaces: readonly string[]
  requiredSurfaces: readonly string[]
  prerequisites: readonly string[]
  capabilities: readonly ("console" | "stdin" | "dom")[]
  outputMode: "text" | "html"
  content: string
  deliveryIssue: number
  seedSamples: readonly string[]
}>

type SampleAuditEntry = Readonly<{
  sampleId: string
  currentKind: SampleKind
  decision:
    | "tour-seed-only"
    | "tour-seed-and-recipe"
    | "tour-seed-and-showcase"
    | "discover-recipe"
    | "discover-showcase"
  tourLessons: readonly string[]
  reason: string
}>

type PathDuplicate = Readonly<{
  sampleId: string
  paths: readonly string[]
}>

type ExcludedDesignSurface = Readonly<{
  surface: string
  designLessons: readonly number[]
  forbiddenTopics: readonly string[]
  forbiddenImports: readonly string[]
  reason: string
}>

export type TourCurriculum = Readonly<{
  title: string
  requiredTopics: readonly string[]
  categories: readonly CurriculumCategory[]
  sampleAudit: readonly SampleAuditEntry[]
  currentPathDuplicates: readonly PathDuplicate[]
  excludedDesignSurfaces: readonly ExcludedDesignSurface[]
}>

export type TourSampleRole = Readonly<{
  id: string
  kind: SampleKind
}>

export type CanonicalTourContent = Readonly<{
  id: string
  interactive: boolean
  hasExpectedOutput: boolean
  source: string
  guide: string
  formatNextLessonId?: string | null
}>

export async function loadValidatedTourCurriculum(
  repositoryRoot: string
): Promise<
  Readonly<{
    curriculum: TourCurriculum
    lessons: readonly LoadedTourLesson[]
    samples: readonly TourSampleRole[]
  }>
> {
  const [curriculumValue, samples] = await Promise.all([
    readFile(
      resolve(repositoryRoot, "examples/tour/curriculum.json"),
      "utf8"
    ).then((source) => JSON.parse(source) as unknown),
    loadSampleRoles(repositoryRoot),
  ])
  const curriculum = parseTourCurriculum(curriculumValue)
  const curriculumLessons = tourCurriculumLessons(curriculum)
  const lessons = await loadTourLessons(
    repositoryRoot,
    curriculumLessons.map(({ id, content }) => ({ id, content }))
  )
  validateTourCurriculum(
    curriculum,
    lessons.map((lesson) => ({
      id: lesson.metadata.id,
      interactive: lesson.metadata.interactive,
      hasExpectedOutput: lesson.expectedOutputPath !== undefined,
      source: lesson.source,
      guide: lesson.guide,
      ...(lesson.metadata.format
        ? { formatNextLessonId: lesson.metadata.format.next.lessonId }
        : {}),
    })),
    samples
  )
  return { curriculum, lessons, samples }
}

export function parseTourCurriculum(value: unknown): TourCurriculum {
  const root = expectRecord(value, "Tour curriculum")
  expectKeys(
    root,
    [
      "$schema",
      "schema",
      "title",
      "requiredTopics",
      "categories",
      "sampleAudit",
      "currentPathDuplicates",
      "excludedDesignSurfaces",
    ],
    "Tour curriculum"
  )
  if (root.schema !== 2) throw new Error("Tour curriculum.schema must be 2")
  return {
    title: expectString(root.title, "Tour curriculum.title"),
    requiredTopics: expectStrings(
      root.requiredTopics,
      "Tour curriculum.requiredTopics"
    ),
    categories: expectArray(root.categories, "Tour curriculum.categories").map(
      parseCategory
    ),
    sampleAudit: expectArray(
      root.sampleAudit,
      "Tour curriculum.sampleAudit"
    ).map(parseSampleAudit),
    currentPathDuplicates: expectArray(
      root.currentPathDuplicates,
      "Tour curriculum.currentPathDuplicates"
    ).map(parsePathDuplicate),
    excludedDesignSurfaces: expectArray(
      root.excludedDesignSurfaces,
      "Tour curriculum.excludedDesignSurfaces"
    ).map(parseExcludedSurface),
  }
}

export function validateTourCurriculum(
  curriculum: TourCurriculum,
  content: readonly CanonicalTourContent[],
  samples: readonly TourSampleRole[]
): void {
  const chapters = tourCurriculumChapters(curriculum)
  const lessons = tourCurriculumLessons(curriculum)
  if (curriculum.categories.length === 0) {
    throw new Error("Tour curriculum must contain a category")
  }
  assertUnique(
    "Tour category id",
    curriculum.categories.map(({ id }) => id)
  )
  assertOrdered(
    "Tour category",
    curriculum.categories.map(({ id, order }) => ({ id, order }))
  )
  assertUnique(
    "Tour chapter id",
    chapters.map(({ id }) => id)
  )
  assertUnique(
    "Tour lesson id",
    lessons.map(({ id }) => id)
  )
  assertUnique(
    "Tour lesson order",
    lessons.map(({ order }) => String(order))
  )
  for (const category of curriculum.categories) {
    if (category.chapters.length === 0) {
      throw new Error(`Tour category ${category.id} must contain a chapter`)
    }
    assertOrdered(
      `Tour category ${category.id} chapter`,
      category.chapters.map(({ id, order }) => ({ id, order }))
    )
    for (const chapter of category.chapters) {
      if (chapter.lessons.length === 0) {
        throw new Error(`Tour chapter ${chapter.id} must contain a lesson`)
      }
    }
  }
  for (const [index, lesson] of lessons.entries()) {
    const expectedOrder = index + 1
    if (lesson.order !== expectedOrder) {
      throw new Error(
        `Tour lesson ${lesson.id} has order ${lesson.order}; expected ${expectedOrder}`
      )
    }
    const expectedContent = `lessons/${lesson.id}/lesson.json`
    if (lesson.content !== expectedContent) {
      throw new Error(
        `Tour lesson ${lesson.id} content must be ${expectedContent}`
      )
    }
  }
  validatePrerequisiteGraph(lessons)

  assertUnique("required Tour topic", curriculum.requiredTopics)
  const introducedBy = new Map<string, string>()
  for (const lesson of lessons) {
    for (const topic of lesson.introducedSurfaces) {
      const previous = introducedBy.get(topic)
      if (previous) {
        throw new Error(
          `Tour topic ${topic} is introduced by both ${previous} and ${lesson.id}`
        )
      }
      introducedBy.set(topic, lesson.id)
    }
  }
  const missingTopics = curriculum.requiredTopics.filter(
    (topic) => !introducedBy.has(topic)
  )
  if (missingTopics.length > 0) {
    throw new Error(
      `Tour required topic(s) missing: ${missingTopics.join(", ")}`
    )
  }
  const requiredTopics = new Set(curriculum.requiredTopics)
  const unexpectedTopics = [...introducedBy.keys()].filter(
    (topic) => !requiredTopics.has(topic)
  )
  if (unexpectedTopics.length > 0) {
    throw new Error(
      `Tour introduced topic(s) missing from requiredTopics: ${unexpectedTopics.join(", ")}`
    )
  }

  const lessonIds = lessons.map(({ id }) => id)
  assertUnique(
    "canonical Tour lesson id",
    content.map(({ id }) => id)
  )
  if (
    !sameStrings(
      content.map(({ id }) => id),
      lessonIds
    )
  ) {
    throw new Error(
      "Canonical Tour content ids must exactly match curriculum lesson order"
    )
  }
  for (const [index, lessonContent] of content.entries()) {
    const lesson = lessons[index]!
    const expectsInteractive = lesson.capabilities.includes("dom")
    if (lessonContent.interactive !== expectsInteractive) {
      throw new Error(
        `Tour lesson ${lesson.id} interactive flag must match its dom capability`
      )
    }
    if (!lessonContent.interactive && !lessonContent.hasExpectedOutput) {
      throw new Error(
        `Non-interactive Tour lesson ${lesson.id} requires expected output`
      )
    }
    if (lessonContent.formatNextLessonId !== undefined) {
      const expectedNextLessonId = lessons[index + 1]?.id ?? null
      if (lessonContent.formatNextLessonId !== expectedNextLessonId) {
        throw new Error(
          `Tour lesson ${lesson.id} next connection must be ${expectedNextLessonId ?? "null"}`
        )
      }
    }
  }

  assertUnique(
    "sample id",
    samples.map(({ id }) => id)
  )
  assertUnique(
    "sample audit id",
    curriculum.sampleAudit.map(({ sampleId }) => sampleId)
  )
  const samplesById = new Map(samples.map((sample) => [sample.id, sample]))
  const auditIds = curriculum.sampleAudit.map(({ sampleId }) => sampleId).sort()
  const sampleIds = samples.map(({ id }) => id).sort()
  if (!sameStrings(auditIds, sampleIds)) {
    throw new Error("Tour sample audit must contain every sample exactly once")
  }
  const auditedLessons = new Set<string>()
  for (const entry of curriculum.sampleAudit) {
    const sample = samplesById.get(entry.sampleId)!
    if (sample.kind !== entry.currentKind) {
      throw new Error(
        `Tour sample audit kind for ${entry.sampleId} is ${entry.currentKind}; actual kind is ${sample.kind}`
      )
    }
    const expectedTourLessons = lessons
      .filter((lesson) => lesson.seedSamples.includes(entry.sampleId))
      .map(({ id }) => id)
    if (!sameStrings(entry.tourLessons, expectedTourLessons)) {
      throw new Error(
        `Tour sample audit ${entry.sampleId} tourLessons must be ${JSON.stringify(expectedTourLessons)}`
      )
    }
    const expectedDecision = sampleDecision(
      sample.kind,
      expectedTourLessons.length > 0
    )
    if (entry.decision !== expectedDecision) {
      throw new Error(
        `Tour sample audit ${entry.sampleId} decision must be ${expectedDecision}`
      )
    }
    for (const lessonId of entry.tourLessons) {
      if (!lessonIds.includes(lessonId)) {
        throw new Error(
          `Tour sample audit ${entry.sampleId} references missing lesson ${lessonId}`
        )
      }
      auditedLessons.add(lessonId)
    }
  }
  const missingAuditedLessons = lessonIds.filter(
    (lessonId) => !auditedLessons.has(lessonId)
  )
  if (missingAuditedLessons.length > 0) {
    throw new Error(
      `Tour lesson(s) missing sample audit coverage: ${missingAuditedLessons.join(", ")}`
    )
  }
  for (const lesson of lessons) {
    for (const sampleId of lesson.seedSamples) {
      if (!samplesById.has(sampleId)) {
        throw new Error(
          `Tour lesson ${lesson.id} references missing seed sample ${sampleId}`
        )
      }
    }
  }
  if (curriculum.currentPathDuplicates.length > 0) {
    throw new Error("Tour curriculum still contains unresolved sample paths")
  }

  assertUnique(
    "excluded design surface",
    curriculum.excludedDesignSurfaces.map(({ surface }) => surface)
  )
  const forbiddenTopics = new Map<string, string>()
  const forbiddenImports = new Map<string, string>()
  for (const excluded of curriculum.excludedDesignSurfaces) {
    for (const topic of excluded.forbiddenTopics) {
      const previous = forbiddenTopics.get(topic)
      if (previous) {
        throw new Error(
          `Forbidden Tour topic ${topic} is duplicated in ${previous} and ${excluded.surface}`
        )
      }
      forbiddenTopics.set(topic, excluded.surface)
    }
    for (const moduleName of excluded.forbiddenImports) {
      const previous = forbiddenImports.get(moduleName)
      if (previous) {
        throw new Error(
          `Forbidden Tour import ${moduleName} is duplicated in ${previous} and ${excluded.surface}`
        )
      }
      forbiddenImports.set(moduleName, excluded.surface)
    }
  }
  for (const [topic, surface] of forbiddenTopics) {
    const lessonId = introducedBy.get(topic)
    if (lessonId) {
      throw new Error(
        `Tour lesson ${lessonId} introduces excluded ${surface} topic ${topic}`
      )
    }
  }
  for (const lessonContent of content) {
    const material = `${lessonContent.source}\n${lessonContent.guide}`
    for (const [moduleName, surface] of forbiddenImports) {
      if (material.includes(`"${moduleName}`)) {
        throw new Error(
          `Tour lesson ${lessonContent.id} imports excluded ${surface} module ${moduleName}`
        )
      }
    }
  }
}

export function tourCurriculumChapters(
  curriculum: TourCurriculum
): readonly CurriculumChapter[] {
  return curriculum.categories.flatMap(({ chapters }) => chapters)
}

export function tourCurriculumLessons(
  curriculum: TourCurriculum
): readonly CurriculumLesson[] {
  return tourCurriculumChapters(curriculum).flatMap(({ lessons }) => lessons)
}

function validatePrerequisiteGraph(lessons: readonly CurriculumLesson[]): void {
  const lessonsById = new Map(lessons.map((lesson) => [lesson.id, lesson]))
  const orderById = new Map(
    lessons.map((lesson, index) => [lesson.id, index] as const)
  )
  for (const lesson of lessons) {
    assertUnique(`Tour lesson ${lesson.id} prerequisite`, lesson.prerequisites)
    for (const prerequisite of lesson.prerequisites) {
      if (!lessonsById.has(prerequisite)) {
        throw new Error(
          `Tour lesson ${lesson.id} references missing prerequisite ${prerequisite}`
        )
      }
    }
  }

  const visiting = new Set<string>()
  const visited = new Set<string>()
  const visit = (lessonId: string, path: readonly string[]): void => {
    if (visited.has(lessonId)) return
    if (visiting.has(lessonId)) {
      const cycleStart = path.indexOf(lessonId)
      const cycle = [...path.slice(cycleStart), lessonId]
      throw new Error(`Tour prerequisite cycle: ${cycle.join(" -> ")}`)
    }
    visiting.add(lessonId)
    const lesson = lessonsById.get(lessonId)!
    for (const prerequisite of lesson.prerequisites) {
      visit(prerequisite, [...path, lessonId])
    }
    visiting.delete(lessonId)
    visited.add(lessonId)
  }
  for (const lesson of lessons) visit(lesson.id, [])

  for (const lesson of lessons) {
    const lessonOrder = orderById.get(lesson.id)!
    for (const prerequisite of lesson.prerequisites) {
      if (orderById.get(prerequisite)! >= lessonOrder) {
        throw new Error(
          `Tour lesson ${lesson.id} prerequisite ${prerequisite} must appear earlier`
        )
      }
    }
  }
}

async function loadSampleRoles(
  repositoryRoot: string
): Promise<readonly TourSampleRole[]> {
  const samplesRoot = resolve(repositoryRoot, "examples/samples")
  const entries = await readdir(samplesRoot, { withFileTypes: true })
  const directories = entries
    .filter((entry) => entry.isDirectory())
    .map(({ name }) => name)
    .sort()
  return Promise.all(
    directories.map(async (directoryId) => {
      const metadata = parseSampleMetadata(
        JSON.parse(
          await readFile(
            resolve(samplesRoot, directoryId, "sample.json"),
            "utf8"
          )
        ),
        directoryId
      )
      return { id: metadata.id, kind: metadata.kind }
    })
  )
}

function parseCategory(value: unknown, index: number): CurriculumCategory {
  const label = `Tour category ${index}`
  const record = expectRecord(value, label)
  expectKeys(record, ["id", "order", "title", "summary", "chapters"], label)
  return {
    id: expectSlug(record.id, `${label}.id`),
    order: expectInteger(record.order, `${label}.order`),
    title: expectString(record.title, `${label}.title`),
    summary: expectString(record.summary, `${label}.summary`),
    chapters: expectArray(record.chapters, `${label}.chapters`).map(
      parseChapter
    ),
  }
}

function parseChapter(value: unknown, index: number): CurriculumChapter {
  const label = `Tour chapter ${index}`
  const record = expectRecord(value, label)
  expectKeys(record, ["id", "order", "title", "summary", "lessons"], label)
  return {
    id: expectSlug(record.id, `${label}.id`),
    order: expectInteger(record.order, `${label}.order`),
    title: expectString(record.title, `${label}.title`),
    summary: expectString(record.summary, `${label}.summary`),
    lessons: expectArray(record.lessons, `${label}.lessons`).map(parseLesson),
  }
}

function parseLesson(value: unknown, index: number): CurriculumLesson {
  const label = `Tour lesson ${index}`
  const record = expectRecord(value, label)
  expectKeys(
    record,
    [
      "id",
      "order",
      "title",
      "summary",
      "goal",
      "focus",
      "introducedSurfaces",
      "requiredSurfaces",
      "prerequisites",
      "capabilities",
      "outputMode",
      "content",
      "deliveryIssue",
      "seedSamples",
    ],
    label
  )
  const capabilities = expectStrings(
    record.capabilities,
    `${label}.capabilities`
  )
  if (
    capabilities.some(
      (capability) => !["console", "stdin", "dom"].includes(capability)
    )
  ) {
    throw new Error(`${label}.capabilities contains an unknown capability`)
  }
  const outputMode = expectString(record.outputMode, `${label}.outputMode`)
  if (outputMode !== "text" && outputMode !== "html") {
    throw new Error(`${label}.outputMode must be text or html`)
  }
  return {
    id: expectLessonId(record.id, `${label}.id`),
    order: expectInteger(record.order, `${label}.order`),
    title: expectString(record.title, `${label}.title`),
    summary: expectString(record.summary, `${label}.summary`),
    goal: expectString(record.goal, `${label}.goal`),
    focus: expectStrings(record.focus, `${label}.focus`),
    introducedSurfaces: expectStrings(
      record.introducedSurfaces,
      `${label}.introducedSurfaces`
    ),
    requiredSurfaces: expectStrings(
      record.requiredSurfaces,
      `${label}.requiredSurfaces`
    ),
    prerequisites: expectLessonIds(
      record.prerequisites,
      `${label}.prerequisites`
    ),
    capabilities: capabilities as CurriculumLesson["capabilities"],
    outputMode,
    content: expectTourContentPath(record.content, `${label}.content`),
    deliveryIssue: expectInteger(
      record.deliveryIssue,
      `${label}.deliveryIssue`
    ),
    seedSamples: expectSlugs(record.seedSamples, `${label}.seedSamples`),
  }
}

function parseSampleAudit(value: unknown, index: number): SampleAuditEntry {
  const label = `Tour sample audit ${index}`
  const record = expectRecord(value, label)
  expectKeys(
    record,
    ["sampleId", "currentKind", "decision", "tourLessons", "reason"],
    label
  )
  const currentKind = expectString(record.currentKind, `${label}.currentKind`)
  if (
    !(["lesson", "recipe", "showcase"] as const).includes(
      currentKind as SampleKind
    )
  ) {
    throw new Error(`${label}.currentKind is invalid`)
  }
  const decision = expectString(record.decision, `${label}.decision`)
  const decisions = [
    "tour-seed-only",
    "tour-seed-and-recipe",
    "tour-seed-and-showcase",
    "discover-recipe",
    "discover-showcase",
  ] as const
  if (!decisions.includes(decision as (typeof decisions)[number])) {
    throw new Error(`${label}.decision is invalid`)
  }
  return {
    sampleId: expectSlug(record.sampleId, `${label}.sampleId`),
    currentKind: currentKind as SampleKind,
    decision: decision as SampleAuditEntry["decision"],
    tourLessons: expectLessonIds(record.tourLessons, `${label}.tourLessons`),
    reason: expectString(record.reason, `${label}.reason`),
  }
}

function parsePathDuplicate(value: unknown, index: number): PathDuplicate {
  const label = `Tour path duplicate ${index}`
  const record = expectRecord(value, label)
  expectKeys(record, ["sampleId", "paths"], label)
  return {
    sampleId: expectSlug(record.sampleId, `${label}.sampleId`),
    paths: expectSlugs(record.paths, `${label}.paths`),
  }
}

function parseExcludedSurface(
  value: unknown,
  index: number
): ExcludedDesignSurface {
  const label = `Excluded design surface ${index}`
  const record = expectRecord(value, label)
  expectKeys(
    record,
    [
      "surface",
      "designLessons",
      "forbiddenTopics",
      "forbiddenImports",
      "reason",
    ],
    label
  )
  return {
    surface: expectSlug(record.surface, `${label}.surface`),
    designLessons: expectArray(
      record.designLessons,
      `${label}.designLessons`
    ).map((lesson, lessonIndex) =>
      expectInteger(lesson, `${label}.designLessons.${lessonIndex}`)
    ),
    forbiddenTopics: expectSlugs(
      record.forbiddenTopics,
      `${label}.forbiddenTopics`
    ),
    forbiddenImports: expectStrings(
      record.forbiddenImports,
      `${label}.forbiddenImports`
    ),
    reason: expectString(record.reason, `${label}.reason`),
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
  allowed: readonly string[],
  label: string
): void {
  const allowedKeys = new Set(allowed)
  const unknown = Object.keys(record).filter((key) => !allowedKeys.has(key))
  if (unknown.length > 0) {
    throw new Error(`${label} has unknown field(s): ${unknown.join(", ")}`)
  }
}

function expectArray(value: unknown, label: string): readonly unknown[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`)
  return value
}

function expectString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${label} must be a non-empty string`)
  }
  return value
}

function expectInteger(value: unknown, label: string): number {
  if (!Number.isInteger(value)) throw new Error(`${label} must be an integer`)
  return value as number
}

function expectStrings(value: unknown, label: string): readonly string[] {
  return expectArray(value, label).map((item, index) =>
    expectString(item, `${label}.${index}`)
  )
}

function expectSlug(value: unknown, label: string): string {
  const result = expectString(value, label)
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/u.test(result)) {
    throw new Error(`${label} must be a stable slug`)
  }
  return result
}

function expectSlugs(value: unknown, label: string): readonly string[] {
  return expectArray(value, label).map((item, index) =>
    expectSlug(item, `${label}.${index}`)
  )
}

function expectLessonId(value: unknown, label: string): string {
  return expectSlug(value, label)
}

function expectTourContentPath(value: unknown, label: string): string {
  const result = expectString(value, label)
  if (!/^lessons\/[a-z0-9]+(?:-[a-z0-9]+)*\/lesson\.json$/u.test(result)) {
    throw new Error(`${label} must reference a Tour lesson descriptor`)
  }
  return result
}

function expectLessonIds(value: unknown, label: string): readonly string[] {
  return expectArray(value, label).map((item, index) =>
    expectLessonId(item, `${label}.${index}`)
  )
}

function assertUnique(label: string, values: readonly string[]): void {
  const seen = new Set<string>()
  for (const value of values) {
    if (seen.has(value)) throw new Error(`Duplicate ${label}: ${value}`)
    seen.add(value)
  }
}

function assertOrdered(
  label: string,
  entries: readonly Readonly<{ id: string; order: number }>[]
): void {
  assertUnique(
    `${label} order`,
    entries.map(({ order }) => String(order))
  )
  for (const [index, entry] of entries.entries()) {
    const expected = index + 1
    if (entry.order !== expected) {
      throw new Error(
        `${label} ${entry.id} has order ${entry.order}; expected ${expected}`
      )
    }
  }
}

function sampleDecision(
  kind: SampleKind,
  hasTourSeed: boolean
): SampleAuditEntry["decision"] {
  if (kind === "lesson") return "tour-seed-only"
  if (kind === "recipe") {
    return hasTourSeed ? "tour-seed-and-recipe" : "discover-recipe"
  }
  return hasTourSeed ? "tour-seed-and-showcase" : "discover-showcase"
}

function sameStrings(
  left: readonly string[],
  right: readonly string[]
): boolean {
  return (
    left.length === right.length &&
    left.every((value, index) => value === right[index])
  )
}
