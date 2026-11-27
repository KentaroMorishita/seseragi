import type { PlaygroundSample } from "../samples"
import { playgroundHref, type UiLocale } from "../ui/i18n"
import { playgroundUrlForSource } from "../workspace/source-link"

const maximumLinkedSourceBytes = 64 * 1024

export const tourChapterExampleMap: Readonly<
  Record<string, readonly string[]>
> = {
  "foundations-basics": ["weather-advice", "fizzbuzz"],
  "functions-core": ["fibonacci-sequence", "local-functions"],
  "data-shapes-and-match": ["records", "expression-calculator"],
  "collections-basics": ["collections", "collection-patterns"],
  "fallibility-maybe": ["collections", "expression-calculator"],
  "fallibility-either": ["either-and-monad", "json-decode"],
  "effects-basics": ["stdin-greeting", "type-aliases"],
  "abstraction-traits": ["newtypes", "impl-and-operators"],
  "signals-derivation": ["signal-state", "interactive-app"],
  "web-ui-pure-html": ["html-components", "interactive-app"],
  "applications-capstone": ["form-todo", "project-flow-app"],
  "side-trips-abstraction-concepts": ["impl-and-operators", "newtypes"],
  "side-trips-abstraction-design": ["newtypes", "project-greeting"],
  "side-trips-web-ui": ["feature-composition", "seseragi-landing-page"],
  "side-trips-collections": ["collection-patterns", "collections"],
}

export function chapterExampleIds(chapterId: string): readonly string[] {
  return tourChapterExampleMap[chapterId] ?? []
}

export function isChapterEnd(
  lessons: readonly Readonly<{ id: string; chapterId: string }>[],
  lessonId: string
): boolean {
  const current = lessons.find((lesson) => lesson.id === lessonId)
  if (current === undefined) return false
  let lastInChapter: string | undefined
  for (const lesson of lessons) {
    if (lesson.chapterId === current.chapterId) lastInChapter = lesson.id
  }
  return lastInChapter === lessonId
}

export function chapterExamplesForLesson(
  lessons: readonly Readonly<{ id: string; chapterId: string }>[],
  lessonId: string,
  allSamples: readonly PlaygroundSample[]
): readonly PlaygroundSample[] {
  if (!isChapterEnd(lessons, lessonId)) return []
  const current = lessons.find((lesson) => lesson.id === lessonId)
  if (current === undefined) return []
  const byId = new Map(allSamples.map((sample) => [sample.id, sample]))
  return chapterExampleIds(current.chapterId)
    .map((id) => byId.get(id))
    .filter((sample): sample is PlaygroundSample => sample !== undefined)
}

export function tourOpenInPlaygroundUrl(
  origin: string,
  locale: UiLocale,
  source: string
): string {
  const base = new URL(playgroundHref(locale), origin).toString()
  if (new TextEncoder().encode(source).byteLength > maximumLinkedSourceBytes) {
    return base
  }
  return playgroundUrlForSource(base, source)
}
