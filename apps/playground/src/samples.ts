import rockPaperScissors from "../../../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg?raw"
import helloWorld from "../../../examples/spec/lessons/01-hello-world.ssrg?raw"
import miniAdventure from "../../../examples/spec/playground/01-mini-adventure.ssrg?raw"
import shippingAdvisor from "../../../examples/spec/playground/02-shipping-advisor.ssrg?raw"
import seseragiQuiz from "../../../examples/spec/playground/03-seseragi-quiz.ssrg?raw"
import arrayScoreboard from "../../../examples/spec/playground/04-array-scoreboard.ssrg?raw"
import traitBadges from "../../../examples/spec/playground/05-trait-badges.ssrg?raw"
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
  "mini-adventure": miniAdventure,
  "shipping-advisor": shippingAdvisor,
  "seseragi-quiz": seseragiQuiz,
  "array-scoreboard": arrayScoreboard,
  "trait-badges": traitBadges,
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
