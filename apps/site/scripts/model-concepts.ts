import { canonicalExample } from "./canonical-example"

export const modelConceptPrograms = [
  {
    id: "model-reader-programs",
    file: "model-reader-programs",
    output: "Hello, Aki!\n",
  },
  {
    id: "model-reader-non-features",
    file: "model-reader-non-features",
    output: "79, 80, pass\n",
  },
  {
    id: "model-reader-grammar",
    file: "model-reader-grammar",
    output: "3, 4\n",
  },
  {
    id: "language-program-entry",
    file: "program-entry",
    output: "Hello, Seseragi!\n",
  },
] as const

export function modelConceptExamples(playgroundUrl: string) {
  return [
    ...modelConceptPrograms
      .slice(0, 3)
      .map((example) =>
        canonicalExample(
          example.id,
          `apps/site/examples/src/language/${example.file}.ssrg`,
          playgroundUrl
        )
      ),
    // This is a valid declaration rejected only as an executable entry point.
    canonicalExample(
      "model-reader-entry-rejected",
      "apps/site/examples/src/language/model-reader-entry-rejected.ssrg",
      playgroundUrl,
      false
    ),
    canonicalExample(
      "model-reader-grammar-invalid",
      "apps/site/examples/invalid/src/language/model-reader-grammar.ssrg",
      playgroundUrl,
      false
    ),
  ]
}

export const modelConceptPages = [
  {
    key: "programs",
    id: "language.model.programs",
    route: "/docs/language/model/programs/",
    module: "pages/language/model/programs/page",
    titles: ["Programs and entry points", "プログラムと実行入口"],
    anchors: ["core-rule", "consequences", "related-rules"],
    related: [
      "/docs/first-run/",
      "/docs/language/modules/entry-points/",
      "/docs/language/effects/runtime-boundaries/",
    ],
  },
  {
    key: "non-features",
    id: "language.model.non-features",
    route: "/docs/language/model/non-features/",
    module: "pages/language/model/non-features/page",
    titles: ["Features the language does not have", "中核に存在しない機能"],
    anchors: ["core-rule", "consequences", "related-rules"],
    related: [
      "/docs/language/model/no-hidden-danger/",
      "/docs/language/model/immutable-by-default/",
      "/docs/language/model/readable-density/",
    ],
  },
  {
    key: "design-principles",
    id: "language.model.design-principles",
    route: "/docs/language/model/design-principles/",
    module: "pages/language/model/design-principles/page",
    titles: ["Design principles", "設計原則"],
    anchors: ["principles", "together"],
    related: [
      "expression-oriented",
      "immutable-by-default",
      "no-hidden-danger",
      "backend-independent-semantics",
      "diagnosable-behavior",
      "visible-costs",
      "readable-density",
    ].map((key) => `/docs/language/model/${key}/`),
  },
  {
    key: "grammar",
    id: "language.grammar",
    route: "/docs/language/grammar/",
    module: "pages/language/grammar/page",
    titles: ["Grammar reference", "文法リファレンス"],
    anchors: [
      "reading",
      "productions",
      "semantic-boundaries",
      "related",
      "source",
    ],
    related: [
      "syntax/source-text",
      "syntax/function-application",
      "syntax/operator-precedence",
      "expressions/blocks-and-local-declarations",
      "patterns/match",
      "types/type-constructors",
      "modules/top-level",
    ].map((key) => `/docs/language/${key}/`),
  },
] as const
