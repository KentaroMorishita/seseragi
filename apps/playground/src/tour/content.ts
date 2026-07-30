export type TourSourceRange = Readonly<{
  startLine: number
  endLine: number
}>

export type TourWalkthroughStep = Readonly<{
  heading: string
  body: string
  sourceRange: TourSourceRange
}>

export type TourIntroducedSurface = Readonly<{
  kind: "syntax" | "type" | "api"
  name: string
  body: string
}>

export type TourLessonFormat = Readonly<{
  prerequisite: string
  walkthrough: readonly TourWalkthroughStep[]
  introduced: readonly TourIntroducedSurface[]
  exercise: Readonly<{
    instruction: string
    reset: "restore-lesson-source"
  }>
  diagnostic: Readonly<{
    heading: string
    body: string
  }>
  recap: readonly string[]
  next: Readonly<{
    lessonId: string | null
    body: string
  }>
  notes?: readonly string[]
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
  format?: TourLessonFormat
  exerciseSource: string
  exerciseExpectedOutput: string
  diagnosticSource: string
  diagnosticOutput: string
}>
