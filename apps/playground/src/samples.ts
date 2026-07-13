import effectBind from "../../../examples/spec/artifacts/schema-1/effect-do-bind/main.ssrg?raw"
import effectSequence from "../../../examples/spec/artifacts/schema-1/effect-do-multiple/main.ssrg?raw"
import effectPureLet from "../../../examples/spec/artifacts/schema-1/effect-do-pure-let/main.ssrg?raw"
import rockPaperScissors from "../../../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg?raw"
import helloWorld from "../../../examples/spec/lessons/01-hello-world.ssrg?raw"
import { sampleCatalog } from "./sample-catalog"

export type PlaygroundSample = {
  readonly id: string
  readonly label: string
  readonly source: string
  readonly stdin: string
}

const sourceById: Readonly<Record<string, string>> = {
  "hello-world": helloWorld,
  "rock-paper-scissors": rockPaperScissors,
  "effect-sequence": effectSequence,
  "effect-pure-let": effectPureLet,
  "effect-bind": effectBind,
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
      source,
      stdin: definition.stdin,
    }
  }
)
