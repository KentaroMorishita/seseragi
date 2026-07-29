import curriculumJson from "../../../../examples/tour/curriculum.json"
import { generatedTourLessons } from "../generated/tour-manifest"
import { samples } from "../samples"

type CurriculumChapter = Readonly<{
  id: string
  title: string
  summary: string
}>

type CurriculumLesson = Readonly<{
  id: string
  order: number
  chapter: string
  title: string
  focus: readonly string[]
  introduces: readonly string[]
  prerequisites: readonly string[]
  capabilities: readonly ("console" | "stdin" | "dom")[]
  outputMode: "text" | "html"
  seedSamples: readonly string[]
}>

type Curriculum = Readonly<{
  title: string
  chapters: readonly CurriculumChapter[]
  lessons: readonly CurriculumLesson[]
}>

export type TourChapter = CurriculumChapter

export type TourLesson = CurriculumLesson &
  Readonly<{
    source: string
    guide: string
    stdin: string
    expectedOutput: string
    interactive: boolean
    sourcePath: string
    contentKind: "canonical" | "seed"
    summary: string
    challenge: string
  }>

const curriculum = curriculumJson as Curriculum
const generatedContentById = new Map(
  generatedTourLessons.map((content) => [content.id, content] as const)
)

if (generatedContentById.size !== generatedTourLessons.length) {
  throw new Error("Canonical Tour lesson ids must be unique")
}
for (const id of generatedContentById.keys()) {
  if (!curriculum.lessons.some((lesson) => lesson.id === id)) {
    throw new Error(`Canonical Tour lesson ${id} is not in the curriculum`)
  }
}

export const tourChapters: readonly TourChapter[] = curriculum.chapters

export const tourLessons: readonly TourLesson[] = curriculum.lessons.map(
  (lesson) => {
    const content = generatedContentById.get(lesson.id)
    const sample = lesson.seedSamples
      .map((sampleId) => samples.find(({ id }) => id === sampleId))
      .find((candidate) => candidate !== undefined)
    const resolvedContent = content ?? sample
    if (resolvedContent === undefined) {
      throw new Error(`Tour lesson ${lesson.id} has no available seed sample`)
    }
    return {
      ...lesson,
      source: resolvedContent.source,
      guide: resolvedContent.guide,
      stdin: resolvedContent.stdin,
      expectedOutput: resolvedContent.expectedOutput,
      interactive: resolvedContent.interactive,
      sourcePath: resolvedContent.sourcePath,
      contentKind: content === undefined ? "seed" : "canonical",
      summary: `${lesson.focus.join("と")}を、動くsourceで確かめます。`,
      challenge:
        content?.challenge ??
        `${lesson.introduces.join("、")}に注目して、値を変えてもう一度Runしてみましょう。`,
    }
  }
)

export const tourTitle = curriculum.title

export function findTourLesson(id: string | null): TourLesson {
  return tourLessons.find((lesson) => lesson.id === id) ?? tourLessons[0]!
}
