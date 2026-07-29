import {
  generatedDiscoverGroups,
  generatedSamples,
} from "./generated/sample-manifest"
import type {
  DiscoverGroupDefinition,
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

export const discoverGroups: readonly DiscoverGroupDefinition[] =
  generatedDiscoverGroups
