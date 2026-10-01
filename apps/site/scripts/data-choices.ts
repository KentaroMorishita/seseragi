import { canonicalExample } from "./canonical-example"

// Each record names a bounded reader question, its exact runnable source and
// one independently rejected source. Shared Delivery source is registered once.
export const dataChoices = [
  {
    path: "data/records",
    id: "language.data.records",
    example: "reader-records",
    source: "reader-records",
    output: "10\n42\nAki",
    invalid: "invalid-record-literal",
    invalidId: "data-invalid-record",
    diagnostic: "SES-T0101",
  },
  {
    path: "data/structs",
    id: "language.data.structs",
    example: "reader-structs",
    source: "reader-structs",
    output: "Aki\nMio\n1",
    invalid: "invalid-struct-literal",
    invalidId: "data-invalid-struct",
    diagnostic: "SES-T0101",
  },
  {
    path: "data/algebraic-data-types",
    id: "language.data.algebraic-data-types",
    example: "data-reader-alternatives",
    source: "data-reader-alternatives",
    output: "not shipped\ntracking: JP42",
    invalid: "data-reader-alternatives",
    invalidId: "data-reader-alternatives-invalid",
    diagnostic: "SES-T0101",
  },
  {
    path: "types/closed-records",
    id: "language.types.closed-records",
    example: "data-reader-closed-records",
    source: "data-reader-closed-records",
    output: "ready",
    invalid: "data-reader-closed-records",
    invalidId: "data-reader-closed-records-invalid",
    diagnostic: "SES-E0001",
  },
  {
    path: "types/nominal-and-structural-types",
    id: "language.types.nominal-structural",
    example: "data-reader-nominal-structural",
    source: "data-reader-nominal-structural",
    output: "Aki\nMio\nNotebook",
    invalid: "data-reader-nominal-structural",
    invalidId: "data-reader-nominal-structural-invalid",
    diagnostic: "SES-T0101",
  },
  {
    path: "types/type-constructors",
    id: "language.types.type-constructors",
    example: "data-reader-constructors",
    source: "data-reader-constructors",
    output: "42\nready",
    invalid: "data-reader-constructors",
    invalidId: "data-reader-constructors-invalid",
    diagnostic: "SES-T0101",
  },
  {
    path: "expressions/conditionals",
    id: "language.expressions.conditionals",
    example: "data-reader-conditionals",
    source: "data-reader-conditionals",
    output: "500\n0",
    invalid: "data-reader-conditionals",
    invalidId: "data-reader-conditionals-invalid",
    diagnostic: "SES-T0101",
  },
  {
    path: "patterns/binding-rules",
    id: "language.patterns.binding-rules",
    example: "data-reader-binding",
    source: "data-reader-binding",
    output: "Aki: 42",
    invalid: "data-reader-binding",
    invalidId: "data-reader-binding-invalid",
    diagnostic: "SES-N0002",
  },
  {
    path: "patterns/irrefutable-patterns",
    id: "language.patterns.irrefutable-patterns",
    example: "data-reader-irrefutable",
    source: "data-reader-irrefutable",
    output: "Notebook: 2\n0\n10",
    invalid: "data-reader-irrefutable",
    invalidId: "data-reader-irrefutable-invalid",
    diagnostic: "SES-T0101",
  },
  {
    path: "patterns/match",
    id: "language.patterns.match",
    example: "data-reader-alternatives",
    source: "data-reader-alternatives",
    output: "not shipped\ntracking: JP42",
    invalid: "data-reader-match",
    invalidId: "data-reader-match-invalid",
    diagnostic: "SES-T0301",
  },
] as const

export function dataChoiceExamples(playgroundUrl: string) {
  const examples = dataChoices
    .slice(2)
    .flatMap((entry) => [
      canonicalExample(
        entry.example,
        `apps/site/examples/src/language/${entry.source}.ssrg`,
        playgroundUrl
      ),
      canonicalExample(
        entry.invalidId,
        `apps/site/examples/invalid/src/language/${entry.invalid}.ssrg`,
        playgroundUrl,
        false
      ),
    ])
  examples.push(
    canonicalExample(
      "data-reader-delivery-typescript",
      "apps/site/examples/comparisons/delivery-state/typescript.ts",
      playgroundUrl,
      false,
      "typescript"
    )
  )
  return [...new Map(examples.map((example) => [example.id, example])).values()]
}
