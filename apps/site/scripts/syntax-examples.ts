import { canonicalExample } from "./canonical-example"

// Executable evidence and page identities, not another page-content format.
export const syntaxExampleCases = [
  {
    id: "language.syntax.source-text",
    key: "source-text",
    route: "/docs/language/syntax/source-text/",
    module: "pages/language/syntax/source-text/page",
    source: "apps/site/examples/src/language/syntax-reader-source-text.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/syntax-reader-source-text.ssrg",
    expectedOutput: "Aki: 42 (primary)\n",
    expectedDiagnostic: "SES-P0101",
  },
  {
    id: "language.syntax.literals",
    key: "literals",
    route: "/docs/language/syntax/literals/",
    module: "pages/language/syntax/literals/page",
    source: "apps/site/examples/src/language/syntax-reader-literals.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/syntax-reader-literals.ssrg",
    expectedOutput: "3, 2.5, True, S, Seseragi\n",
    expectedDiagnostic: "SES-P0203",
  },
  {
    id: "language.syntax.character-string-escapes",
    key: "escapes",
    route: "/docs/language/syntax/character-string-escapes/",
    module: "pages/language/syntax/character-string-escapes/page",
    source: "apps/site/examples/src/language/syntax-reader-escapes.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/syntax-reader-escapes.ssrg",
    expectedOutput: '"Seseragi"\ndocs\\guide\nλ\nfirst\nsecond\n',
    expectedDiagnostic: "SES-P0201",
  },
  {
    id: "language.syntax.layout-and-line-continuation",
    key: "layout",
    route: "/docs/language/syntax/layout-and-line-continuation/",
    module: "pages/language/syntax/layout-and-line-continuation/page",
    source: "apps/site/examples/src/language/syntax-reader-layout.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/syntax-reader-layout.ssrg",
    expectedOutput: "7, 7\n",
    expectedDiagnostic: "SES-P0001",
  },
  {
    id: "language.syntax.reserved-words",
    key: "names",
    route: "/docs/language/syntax/reserved-words-and-names/",
    module: "pages/language/syntax/reserved-words-and-names/page",
    source: "apps/site/examples/src/language/syntax-reader-names.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/syntax-reader-names.ssrg",
    expectedOutput: "Hello, Aki\n",
    expectedDiagnostic: "SES-N0001",
  },
  {
    id: "language.syntax.optional-record-fields",
    key: "optional-fields",
    route: "/docs/language/syntax/optional-record-fields/",
    module: "pages/language/syntax/optional-record-fields/page",
    source:
      "apps/site/examples/src/language/syntax-reader-optional-fields.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/syntax-reader-optional-fields.ssrg",
    expectedOutput: "Aki, A\n",
    expectedDiagnostic: "SES-T0101",
  },
  {
    id: "language.syntax.method-calls",
    key: "methods",
    route: "/docs/language/syntax/method-calls/",
    module: "pages/language/syntax/method-calls/page",
    source: "apps/site/examples/src/language/syntax-reader-methods.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/syntax-reader-methods.ssrg",
    expectedOutput: "10\n15\n",
    expectedDiagnostic: "SES-T0503",
  },
  {
    id: "language.syntax.pipelines",
    key: "pipelines",
    route: "/docs/language/syntax/pipelines-and-low-precedence-application/",
    module:
      "pages/language/syntax/pipelines-and-low-precedence-application/page",
    source: "apps/site/examples/src/language/syntax-reader-pipelines.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/syntax-reader-pipelines.ssrg",
    expectedOutput: "[> build, > bundle]\nTrue\nJust > build\nNothing\n",
    expectedDiagnostic: "SES-T0101",
  },
  {
    id: "language.syntax.operator-precedence",
    key: "precedence",
    route: "/docs/language/syntax/operator-precedence/",
    module: "pages/language/syntax/operator-precedence/page",
    source: "apps/site/examples/src/language/syntax-reader-precedence.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/syntax-reader-precedence.ssrg",
    expectedOutput: "7, 9, True\n",
    expectedDiagnostic: "SES-T0101",
  },
  {
    id: "language.syntax.custom-operators",
    key: "custom-operators",
    route: "/docs/language/syntax/custom-operators/",
    module: "pages/language/syntax/custom-operators/page",
    source:
      "apps/site/examples/src/language/syntax-reader-custom-operators.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/syntax-reader-custom-operators.ssrg",
    expectedOutput: "Seseragi docs\n",
    expectedDiagnostic: "SES-P0001",
  },
  {
    id: "language.expressions.evaluation",
    key: "evaluation",
    route: "/docs/language/expressions/evaluation/",
    module: "pages/language/expressions/evaluation/page",
    source: "apps/site/examples/src/language/syntax-reader-evaluation.ssrg",
    invalidSource:
      "apps/site/examples/invalid/src/language/syntax-reader-evaluation.ssrg",
    expectedOutput: "23\n",
    expectedDiagnostic: "SES-T0101",
  },
] as const

export function syntaxExamples(playgroundUrl: string) {
  return syntaxExampleCases.flatMap((example) => [
    canonicalExample(
      `syntax-reader-${example.key}`,
      example.source,
      playgroundUrl
    ),
    canonicalExample(
      `syntax-reader-${example.key}-invalid`,
      example.invalidSource,
      playgroundUrl,
      false
    ),
  ])
}
