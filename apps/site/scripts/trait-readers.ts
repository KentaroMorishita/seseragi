import { canonicalExample } from "./canonical-example"

export const traitReaders = [
  {
    key: "model",
    output: "ticket-42\nAki\n",
    diagnostic: "SES-T0201",
    message: "A required trait instance is not available",
  },
  {
    key: "declarations",
    output: "ticket-42\n#42\n",
    diagnostic: "SES-N0001",
    message: "Name could not be resolved",
  },
  {
    key: "instances",
    output: "ticket-42\nticket-7\n",
    diagnostic: "SES-T0101",
    message: "Trait instance method signature mismatch",
  },
  {
    key: "constraints",
    output: "value: ticket-42\nvalue: [ticket-42]\n",
    diagnostic: "SES-T0201",
    message: "A required trait instance is not available",
  },
  {
    key: "methods-versus-traits",
    output: "Aki\nUser(Aki)\n",
    diagnostic: "SES-T0101",
    message: "This record has no such field",
  },
  {
    key: "method-calls",
    output: "ticket-42\nlog: ticket-42\n",
    diagnostic: "SES-T0201",
    message: "A required trait instance is not available",
  },
  {
    key: "deriving",
    output: "True\nTicket { number: 42 }\n",
    diagnostic: "SES-N0001",
    message: "Name could not be resolved",
  },
  {
    key: "standard-operators",
    output: "42\n42\n",
    diagnostic: "SES-T0201",
    message: "A required trait instance is not available",
  },
  {
    key: "coherence",
    output: "ticket-42\nlog: ticket-42\n",
    diagnostic: "SES-T0202",
    message: "Trait instance duplicate",
  },
  {
    key: "laws",
    output: "True\nTrue\n",
    diagnostic: "SES-T0101",
    message: "Expression type does not match the expected type",
  },
  {
    key: "do-notation",
    output: "1 River Road, Tokyo\naddress incomplete\n",
    diagnostic: "SES-T0101",
    message: "Do refutable bind pattern",
  },
  {
    key: "do-block-typing",
    output: "Just Aki: 2\nNothing\n",
    diagnostic: "SES-T0101",
    message: "Do monad constructor mismatch",
  },
  {
    key: "do-desugaring",
    output: "True\nTrue\n",
    diagnostic: "SES-T0101",
    message: "Argument type does not match the parameter type",
  },
] as const

export const traitReaderExtras = [
  { key: "laws-broken", output: "False\nTrue\n" },
  { key: "effect-do", output: "ready\n(42, 42)\n42\n-1\nJust (2, 2)\n" },
] as const

export function traitReaderExamples(playgroundUrl: string) {
  return [
    ...traitReaders.flatMap(({ key }) => [
      canonicalExample(
        `trait-reader-${key}`,
        `apps/site/examples/src/language/trait-reader-${key}.ssrg`,
        playgroundUrl
      ),
      canonicalExample(
        `trait-reader-${key}-invalid`,
        `apps/site/examples/invalid/src/language/trait-reader-${key}.ssrg`,
        playgroundUrl,
        false
      ),
    ]),
    ...traitReaderExtras.map(({ key }) =>
      canonicalExample(
        `trait-reader-${key}`,
        `apps/site/examples/src/language/trait-reader-${key}.ssrg`,
        playgroundUrl
      )
    ),
    canonicalExample(
      "trait-reader-generic-monad-invalid",
      "apps/site/examples/invalid/src/language/trait-reader-generic-monad.ssrg",
      playgroundUrl,
      false
    ),
    canonicalExample(
      "trait-reader-model-typescript",
      "apps/site/examples/comparisons/trait-label/typescript.ts",
      playgroundUrl,
      false,
      "typescript"
    ),
  ]
}
