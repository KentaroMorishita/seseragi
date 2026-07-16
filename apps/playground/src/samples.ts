import rockPaperScissors from "../../../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg?raw"
import pipelineApplication from "../../../examples/spec/artifacts/schema-1/pipeline-application/main.ssrg?raw"
import stringAdd from "../../../examples/spec/artifacts/schema-1/string-add/main.ssrg?raw"
import templateInterpolation from "../../../examples/spec/artifacts/schema-1/template-interpolation/main.ssrg?raw"
import rangeComprehension from "../../../examples/spec/artifacts/schema-1/range-comprehension/main.ssrg?raw"
import comprehensionPatternFilter from "../../../examples/spec/artifacts/schema-1/comprehension-pattern-filter/main.ssrg?raw"
import userIterableComprehension from "../../../examples/spec/artifacts/schema-1/user-iterable-comprehension/main.ssrg?raw"
import listComprehension from "../../../examples/spec/artifacts/schema-1/list-comprehension/main.ssrg?raw"
import helloWorld from "../../../examples/spec/lessons/01-hello-world.ssrg?raw"
import miniAdventure from "../../../examples/spec/playground/01-mini-adventure.ssrg?raw"
import shippingAdvisor from "../../../examples/spec/playground/02-shipping-advisor.ssrg?raw"
import seseragiQuiz from "../../../examples/spec/playground/03-seseragi-quiz.ssrg?raw"
import arrayScoreboard from "../../../examples/spec/playground/04-array-scoreboard.ssrg?raw"
import traitBadges from "../../../examples/spec/playground/05-trait-badges.ssrg?raw"
import genericInstance from "../../../examples/spec/playground/06-generic-instance.ssrg?raw"
import userAddOperator from "../../../examples/spec/artifacts/schema-1/user-add-operator/main.ssrg?raw"
import userEqOperator from "../../../examples/spec/artifacts/schema-1/user-eq-operator/main.ssrg?raw"
import partialFunctorValue from "../../../examples/spec/artifacts/schema-1/polymorphic-partial-functor/main.ssrg?raw"
import partialConstrainedFunction from "../../../examples/spec/artifacts/schema-1/partial-constrained-function/main.ssrg?raw"
import applicativeMaybe from "../../../examples/spec/artifacts/schema-1/applicative-maybe/main.ssrg?raw"
import applicativeValidation from "../../../examples/spec/artifacts/schema-1/applicative-validation/main.ssrg?raw"
import monadMaybe from "../../../examples/spec/artifacts/schema-1/monad-maybe/main.ssrg?raw"
import monadEither from "../../../examples/spec/artifacts/schema-1/monad-either/main.ssrg?raw"
import {
  sampleCatalog,
  type PlaygroundSampleDefinition,
} from "./sample-catalog"

export type PlaygroundSample = {
  readonly id: string
  readonly label: string
  readonly category: PlaygroundSampleDefinition["category"]
  readonly source: string
  readonly stdin: string
}

const sourceById: Readonly<Record<string, string>> = {
  "hello-world": helloWorld,
  "pipeline-application": pipelineApplication,
  "string-add": stringAdd,
  "template-interpolation": templateInterpolation,
  "range-comprehension": rangeComprehension,
  "comprehension-pattern-filter": comprehensionPatternFilter,
  "user-iterable-comprehension": userIterableComprehension,
  "list-comprehension": listComprehension,
  "rock-paper-scissors": rockPaperScissors,
  "mini-adventure": miniAdventure,
  "shipping-advisor": shippingAdvisor,
  "seseragi-quiz": seseragiQuiz,
  "array-scoreboard": arrayScoreboard,
  "trait-badges": traitBadges,
  "generic-instance": genericInstance,
  "user-add-operator": userAddOperator,
  "user-eq-operator": userEqOperator,
  "partial-functor-value": partialFunctorValue,
  "partial-constrained-function": partialConstrainedFunction,
  "applicative-maybe": applicativeMaybe,
  "applicative-validation": applicativeValidation,
  "monad-maybe": monadMaybe,
  "monad-either": monadEither,
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
      category: definition.category,
      source,
      stdin: definition.stdin,
    }
  }
)
