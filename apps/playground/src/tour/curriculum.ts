import curriculumJson from "../../../../examples/tour/curriculum.json"
import { generatedTourLessons } from "../generated/tour-manifest"
import type { TourLessonFormat } from "./content"

type CurriculumLesson = Readonly<{
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
  seedSamples: readonly string[]
}>

type CurriculumChapter = Readonly<{
  id: string
  order: number
  title: string
  summary: string
  lessons: readonly CurriculumLesson[]
}>

type CurriculumCategory = Readonly<{
  id: string
  order: number
  title: string
  summary: string
  chapters: readonly CurriculumChapter[]
}>

type Curriculum = Readonly<{
  title: string
  categories: readonly CurriculumCategory[]
}>

export type TourCategory = Omit<CurriculumCategory, "chapters">

export type TourChapter = Omit<CurriculumChapter, "lessons"> &
  Readonly<{
    categoryId: string
  }>

export type TourLesson = CurriculumLesson &
  Readonly<{
    categoryId: string
    chapterId: string
    position: number
    source: string
    guide: string
    stdin: string
    expectedOutput: string
    interactive: boolean
    sourcePath: string
    challenge: string
    format?: TourLessonFormat
    exerciseSource: string
    exerciseExpectedOutput: string
    diagnosticSource: string
    diagnosticOutput: string
  }>

const curriculum = curriculumJson as Curriculum
const generatedContentById = new Map(
  generatedTourLessons.map((content) => [content.id, content] as const)
)

if (generatedContentById.size !== generatedTourLessons.length) {
  throw new Error("Canonical Tour lesson ids must be unique")
}

export const tourCategories: readonly TourCategory[] =
  curriculum.categories.map(({ chapters: _chapters, ...category }) => category)

export const tourChapters: readonly TourChapter[] =
  curriculum.categories.flatMap((category) =>
    category.chapters.map(({ lessons: _lessons, ...chapter }) => ({
      ...chapter,
      categoryId: category.id,
    }))
  )

const curriculumLessons = curriculum.categories.flatMap((category) =>
  category.chapters.flatMap((chapter) =>
    chapter.lessons.map((lesson) => ({
      ...lesson,
      categoryId: category.id,
      chapterId: chapter.id,
    }))
  )
)

for (const id of generatedContentById.keys()) {
  if (!curriculumLessons.some((lesson) => lesson.id === id)) {
    throw new Error(`Canonical Tour lesson ${id} is not in the curriculum`)
  }
}

export const tourLessons: readonly TourLesson[] = curriculumLessons.map(
  (lesson, index) => {
    const content = generatedContentById.get(lesson.id)
    if (content === undefined) {
      throw new Error(`Tour lesson ${lesson.id} has no canonical content`)
    }
    return {
      ...lesson,
      position: index + 1,
      source: content.source,
      guide: content.guide,
      stdin: content.stdin,
      expectedOutput: content.expectedOutput,
      interactive: content.interactive,
      sourcePath: content.sourcePath,
      challenge: content.challenge,
      ...(content.format ? { format: content.format } : {}),
      exerciseSource: content.exerciseSource,
      exerciseExpectedOutput: content.exerciseExpectedOutput,
      diagnosticSource: content.diagnosticSource,
      diagnosticOutput: content.diagnosticOutput,
    }
  }
)

export const tourTitle = curriculum.title

export function findTourLesson(id: string | null): TourLesson {
  return tourLessons.find((lesson) => lesson.id === id) ?? tourLessons[0]!
}
