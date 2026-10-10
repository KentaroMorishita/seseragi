import { canonicalExample } from "./canonical-example"

// Executable sources and results for the chapter; article content stays in Seseragi.
export const functionChapterExamples = [
  { id: "chapter-functions", output: "build\n" },
  {
    id: "chapter-function-values",
    output: "[ Build , bundle]\n[bundle]\n",
  },
  { id: "chapter-label-lengths", output: "[5, 6, 5, 3]\n19\n" },
  { id: "chapter-dollar-grouping", output: "READY\n" },
] as const

export function chapterExamples(playgroundUrl: string) {
  return functionChapterExamples.map(({ id }) =>
    canonicalExample(
      id,
      `apps/site/examples/src/language/${id}.ssrg`,
      playgroundUrl
    )
  )
}
