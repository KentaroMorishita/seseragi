import collections from "../../../examples/playground/04-collections.ssrg?raw"
import dataAndPatterns from "../../../examples/playground/03-data-and-patterns.ssrg?raw"
import effectsAndDo from "../../../examples/playground/08-effects-and-do.ssrg?raw"
import eitherAndMonad from "../../../examples/playground/09-either-and-monad.ssrg?raw"
import functionsAndPipelines from "../../../examples/playground/02-functions-and-pipelines.ssrg?raw"
import genericStructs from "../../../examples/playground/06-generic-structs.ssrg?raw"
import helloWorld from "../../../examples/playground/01-hello-world.ssrg?raw"
import htmlComponents from "../../../examples/playground/14-html-components.ssrg?raw"
import implAndOperators from "../../../examples/playground/11-impl-and-operators.ssrg?raw"
import interactiveApp from "../../../examples/playground/15-interactive-app.ssrg?raw"
import newtypes from "../../../examples/playground/07-newtypes.ssrg?raw"
import records from "../../../examples/playground/05-records.ssrg?raw"
import signalComposition from "../../../examples/playground/12-signal-composition.ssrg?raw"
import signalState from "../../../examples/playground/13-signal-state.ssrg?raw"
import traitsAndInstances from "../../../examples/playground/10-traits-and-instances.ssrg?raw"
import {
  type PlaygroundSampleDefinition,
  sampleCatalog,
} from "./sample-catalog"

export type PlaygroundSample = {
  readonly id: string
  readonly label: string
  readonly level: PlaygroundSampleDefinition["level"]
  readonly sequence: number
  readonly summary: string
  readonly concepts: readonly string[]
  readonly sourcePath: string
  readonly source: string
  readonly stdin: string
  readonly outputMode: "text" | "html"
}

const sourceById: Readonly<Record<string, string>> = {
  "hello-world": helloWorld,
  "functions-and-pipelines": functionsAndPipelines,
  "data-and-patterns": dataAndPatterns,
  collections,
  records,
  "generic-structs": genericStructs,
  newtypes,
  "effects-and-do": effectsAndDo,
  "either-and-monad": eitherAndMonad,
  "traits-and-instances": traitsAndInstances,
  "impl-and-operators": implAndOperators,
  "signal-composition": signalComposition,
  "signal-state": signalState,
  "html-components": htmlComponents,
  "interactive-app": interactiveApp,
}

export const samples: readonly PlaygroundSample[] = sampleCatalog.map(
  (definition) => {
    const source = sourceById[definition.id]
    if (source === undefined) {
      throw new Error(`missing bundled source for sample: ${definition.id}`)
    }
    return {
      id: definition.id,
      label: definition.label,
      level: definition.level,
      sequence: definition.sequence,
      summary: definition.summary,
      concepts: definition.concepts,
      sourcePath: definition.sourcePath,
      source,
      stdin: definition.stdin,
      outputMode: definition.outputMode ?? "text",
    }
  }
)
