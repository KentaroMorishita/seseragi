import curriculumJson from "../../../../examples/tour/curriculum.json"
import { type PlaygroundSample, samples } from "../samples"

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
    sample: PlaygroundSample
    summary: string
    challenge: string
  }>

const curriculum = curriculumJson as Curriculum

export const tourChapters: readonly TourChapter[] = curriculum.chapters

export const tourLessons: readonly TourLesson[] = curriculum.lessons.map(
  (lesson) => {
    const sample = lesson.seedSamples
      .map((sampleId) => samples.find(({ id }) => id === sampleId))
      .find((candidate) => candidate !== undefined)
    if (sample === undefined) {
      throw new Error(`Tour lesson ${lesson.id} has no available seed sample`)
    }
    return {
      ...lesson,
      sample,
      summary: `${lesson.focus.join("と")}を、動くsourceで確かめます。`,
      challenge: `${lesson.introduces.join("、")}に注目して、値を変えてもう一度Runしてみましょう。`,
    }
  }
)

export const tourTitle = curriculum.title

export function findTourLesson(id: string | null): TourLesson {
  return tourLessons.find((lesson) => lesson.id === id) ?? tourLessons[0]!
}
