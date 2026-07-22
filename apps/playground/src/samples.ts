import {
  generatedLearningPaths,
  generatedSamples,
} from "./generated/sample-manifest"
import type {
  LearningPathDefinition,
  PlaygroundSampleDefinition,
} from "./sample-catalog"

export type PlaygroundSample = PlaygroundSampleDefinition & {
  readonly source: string
  readonly guide: string
  readonly stdin: string
  readonly expectedOutput: string
}

export const samples: readonly PlaygroundSample[] = generatedSamples.map(
  ({ definition, ...content }) => ({ ...definition, ...content })
)

export const learningPaths: readonly LearningPathDefinition[] =
  generatedLearningPaths
