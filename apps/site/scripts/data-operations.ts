import { canonicalExample } from "./canonical-example"

// Canonical sources and observable results for six data/expression articles.
export const dataOperationCases = [
  {
    id: "language.data.tuples-arrays-and-lists",
    key: "collections",
    route: "/docs/language/data/tuples-arrays-and-lists/",
    module: "pages/language/data/tuples-arrays-and-lists/page",
    source: "apps/site/examples/src/language/data-operations-collections.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/data-operations-collections.ssrg",
    expectedOutput: "answer\n42\nJust 20\nNothing\n20\n`[Ren, Aki, Mio]\n",
    expectedDiagnostic: "SES-T0101",
  },
  {
    id: "language.expressions.lambdas",
    key: "lambdas",
    route: "/docs/language/expressions/lambdas/",
    module: "pages/language/expressions/lambdas/page",
    source: "apps/site/examples/src/language/data-operations-lambdas.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/data-operations-lambdas.ssrg",
    expectedOutput: "[12, 22, 32]\n8\n",
    expectedDiagnostic: "SES-T0101",
  },
  {
    id: "language.expressions.ranges-and-comprehensions",
    key: "comprehensions",
    route: "/docs/language/expressions/ranges-and-comprehensions/",
    module: "pages/language/expressions/ranges-and-comprehensions/page",
    source:
      "apps/site/examples/src/language/data-operations-comprehensions.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/data-operations-comprehensions.ssrg",
    expectedOutput:
      "[4, 16, 36, 64]\n[4, 16, 36]\n[(1, 1), (1, 2), (2, 1), (2, 2)]\n",
    expectedDiagnostic: "SES-T0101",
  },
  {
    id: "language.data.newtypes",
    key: "newtypes",
    route: "/docs/language/data/newtypes/",
    module: "pages/language/data/newtypes/page",
    source: "apps/site/examples/src/language/data-operations-newtypes.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/data-operations-newtypes.ssrg",
    expectedOutput: "42\n43\n",
    expectedDiagnostic: "SES-T0101",
  },
  {
    id: "language.data.impls-and-methods",
    key: "methods",
    route: "/docs/language/data/impls-and-methods/",
    module: "pages/language/data/impls-and-methods/page",
    source: "apps/site/examples/src/language/data-operations-methods.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/data-operations-methods.ssrg",
    expectedOutput: "2\n3\n",
    expectedDiagnostic: "SES-T0503",
  },
  {
    id: "language.data.operator-overloads",
    key: "operators",
    route: "/docs/language/data/operator-overloads/",
    module: "pages/language/data/operator-overloads/page",
    source: "apps/site/examples/src/language/data-operations-operators.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/data-operations-operators.ssrg",
    expectedOutput: "42\n",
    expectedDiagnostic: "SES-T0101",
  },
] as const

export function dataOperationExamples(playgroundUrl: string) {
  return dataOperationCases.flatMap((example) => [
    canonicalExample(
      `data-operations-${example.key}`,
      example.source,
      playgroundUrl
    ),
    canonicalExample(
      `data-operations-${example.key}-invalid`,
      example.invalidSource,
      playgroundUrl,
      false
    ),
  ])
}
