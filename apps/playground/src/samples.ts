import applicativeValidation from "../../../examples/spec/artifacts/schema-1/applicative-validation/main.ssrg?raw"
import comprehensionPatternFilter from "../../../examples/spec/artifacts/schema-1/comprehension-pattern-filter/main.ssrg?raw"
import genericStruct from "../../../examples/spec/artifacts/schema-1/generic-struct/main.ssrg?raw"
import monadEither from "../../../examples/spec/artifacts/schema-1/monad-either/main.ssrg?raw"
import monadMaybe from "../../../examples/spec/artifacts/schema-1/monad-maybe/main.ssrg?raw"
import newtypeUserId from "../../../examples/spec/artifacts/schema-1/newtype-user-id/main.ssrg?raw"
import pipelineApplication from "../../../examples/spec/artifacts/schema-1/pipeline-application/main.ssrg?raw"
import rangeComprehension from "../../../examples/spec/artifacts/schema-1/range-comprehension/main.ssrg?raw"
import rockPaperScissors from "../../../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg?raw"
import signalApplicative from "../../../examples/spec/artifacts/schema-1/signal-applicative/main.ssrg?raw"
import signalSwitchMap from "../../../examples/spec/artifacts/schema-1/signal-switch-map/main.ssrg?raw"
import signalTransaction from "../../../examples/spec/artifacts/schema-1/signal-transaction/main.ssrg?raw"
import templateInterpolation from "../../../examples/spec/artifacts/schema-1/template-interpolation/main.ssrg?raw"
import userAddOperator from "../../../examples/spec/artifacts/schema-1/user-add-operator/main.ssrg?raw"
import webDomCounter from "../../../examples/spec/artifacts/schema-1/web-dom-counter/main.ssrg?raw"
import webHtmlComponentsStyle from "../../../examples/spec/artifacts/schema-1/web-html-components-style/main.ssrg?raw"
import webHtmlSsr from "../../../examples/spec/artifacts/schema-1/web-html-ssr/main.ssrg?raw"
import helloWorld from "../../../examples/spec/lessons/01-hello-world.ssrg?raw"
import miniAdventure from "../../../examples/spec/playground/01-mini-adventure.ssrg?raw"
import {
  type PlaygroundSampleDefinition,
  sampleCatalog,
} from "./sample-catalog"

export type PlaygroundSample = {
  readonly id: string
  readonly label: string
  readonly category: PlaygroundSampleDefinition["category"]
  readonly summary: string
  readonly concepts: readonly string[]
  readonly sourcePath: string
  readonly source: string
  readonly stdin: string
  readonly outputMode: "text" | "html"
}

const sourceById: Readonly<Record<string, string>> = {
  "hello-world": helloWorld,
  "pipeline-application": pipelineApplication,
  "template-interpolation": templateInterpolation,
  "range-comprehension": rangeComprehension,
  "comprehension-pattern-filter": comprehensionPatternFilter,
  "rock-paper-scissors": rockPaperScissors,
  "mini-adventure": miniAdventure,
  "signal-transaction": signalTransaction,
  "signal-switch-map": signalSwitchMap,
  "newtype-user-id": newtypeUserId,
  "generic-struct": genericStruct,
  "user-add-operator": userAddOperator,
  "applicative-validation": applicativeValidation,
  "monad-maybe": monadMaybe,
  "monad-either": monadEither,
  "signal-applicative": signalApplicative,
  "web-html-ssr": webHtmlSsr,
  "web-html-components-style": webHtmlComponentsStyle,
  "web-dom-counter": webDomCounter,
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
      summary: definition.summary,
      concepts: definition.concepts,
      sourcePath: definition.sourcePath,
      source,
      stdin: definition.stdin,
      outputMode: definition.outputMode ?? "text",
    }
  }
)
