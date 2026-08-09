export type TourSourceRange = Readonly<{
  startLine: number
  endLine: number
}>

declare const tourInlineRichTextBrand: unique symbol

export type TourInlineRichText = string & {
  readonly [tourInlineRichTextBrand]: "TourInlineRichText"
}

export type TourWalkthroughStep = Readonly<{
  heading: string
  body: TourInlineRichText
  sourceRange: TourSourceRange
}>

export type TourIntroducedSurface = Readonly<{
  kind: "syntax" | "type" | "api"
  name: string
  body: TourInlineRichText
}>

export type TourLessonFormat = Readonly<{
  prerequisite: TourInlineRichText
  walkthrough: readonly TourWalkthroughStep[]
  introduced: readonly TourIntroducedSurface[]
  exercise: Readonly<{
    instruction: TourInlineRichText
    reset: "restore-lesson-source"
  }>
  diagnostic: Readonly<{
    heading: string
    body: TourInlineRichText
  }>
  recap: readonly TourInlineRichText[]
  next: Readonly<{
    lessonId: string | null
    body: TourInlineRichText
  }>
  notes?: readonly TourInlineRichText[]
}>

export type GeneratedTourLessonContent = Readonly<{
  id: string
  challenge: string
  interactive: boolean
  sourcePath: string
  source: string
  guide: string
  stdin: string
  expectedOutput: string
  expectedFailure: string
  format?: TourLessonFormat
  exerciseSource: string
  exerciseExpectedOutput: string
  diagnosticSource: string
  diagnosticOutput: string
}>
