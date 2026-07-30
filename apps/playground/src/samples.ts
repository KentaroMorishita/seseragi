import {
  generatedDiscoverGroups,
  generatedSamples,
} from "./generated/sample-manifest"
import type {
  DiscoverGroupDefinition,
  PlaygroundSampleDefinition,
} from "./sample-catalog"
import type { WorkspaceSeed } from "./workspace/model"

export type PlaygroundSample = PlaygroundSampleDefinition & {
  readonly source: string
  readonly workspace: WorkspaceSeed
  readonly guide: string
  readonly stdin: string
  readonly expectedOutput: string
}

export const samples: readonly PlaygroundSample[] = generatedSamples.map(
  ({ definition, projectFiles, ...content }) => {
    const firstFile = projectFiles[0]
    if (firstFile === undefined) {
      throw new Error(`sample ${definition.id} has no workspace files`)
    }
    const project = definition.project
    return {
      ...definition,
      ...content,
      workspace: {
        files: projectFiles,
        entryFile: project?.entryFile ?? firstFile.path,
        activeFile: project?.activeFile ?? firstFile.path,
        openFiles: project?.openFiles ?? [firstFile.path],
        expandedFolders: project?.expandedFolders ?? [],
        explorer: { visible: project !== undefined },
      },
    }
  }
)

export const discoverGroups: readonly DiscoverGroupDefinition[] =
  generatedDiscoverGroups
