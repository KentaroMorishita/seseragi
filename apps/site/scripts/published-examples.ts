import { functionChapterExamples } from "./function-chapter-examples"

// Only examples actually used by the new public catalog belong to this gate.
// Outputs are claims made by the articles, checked against a real execution.
export const compositionExamples = [
  {
    id: "composition-transform",
    sourcePath: "apps/site/examples/src/composition/composition-transform.ssrg",
    output: "Just > build\nNothing\n[> build, > check]\n",
  },
  {
    id: "composition-apply",
    sourcePath: "apps/site/examples/src/composition/composition-apply.ssrg",
    output: "Just build site\nNothing\nNothing\n",
  },
  {
    id: "composition-bind",
    sourcePath: "apps/site/examples/src/composition/composition-bind.ssrg",
    output:
      "Just Just Compile the project.\nJust Compile the project.\nJust Check the types.\nNothing\nNothing\n",
  },
] as const

export const effectExamples = [
  {
    id: "effect-notice",
    route: "/docs/effects/actions/",
    sourcePath: "apps/site/examples/src/effects/effect-notice.ssrg",
    output: "before\n> ready\n> ready\n",
  },
  {
    id: "effect-error",
    route: "/docs/effects/errors/",
    sourcePath: "apps/site/examples/src/effects/effect-error.ssrg",
    output: "Hello, Aki.\nNo name: Blank\n",
  },
] as const

export const typeExamples = [
  {
    id: "types-records",
    route: "/docs/types/records/",
    sourcePath: "apps/site/examples/src/types/types-records.ssrg",
    output: "build site\nbuild tests\ncheck types\n",
  },
  {
    id: "types-variants",
    route: "/docs/types/variants/",
    sourcePath: "apps/site/examples/src/types/types-variants.ssrg",
    output: "build: waiting\nbuild: 40%\nbuild: done\n",
  },
  {
    id: "types-generic",
    route: "/docs/types/generics/",
    sourcePath: "apps/site/examples/src/types/types-generic.ssrg",
    output: "build\n3\n(empty)\n",
  },
] as const

export const typeDiagnostics = [
  {
    id: "types-match-incomplete",
    route: "/docs/types/variants/",
    sourcePath:
      "apps/site/examples/invalid/src/types/types-match-incomplete.ssrg",
    code: "SES-T0301",
    message: "This match does not cover every possible value",
    detail: "missing patterns: Cancelled _",
    display:
      "SES-T0301: This match does not cover every possible value\nmissing patterns: Cancelled _",
  },
  {
    id: "types-show-unconstrained",
    route: "/docs/types/generics/",
    sourcePath:
      "apps/site/examples/invalid/src/types/types-show-unconstrained.ssrg",
    code: "SES-T0201",
    message: "A required trait instance is not available",
    detail: "no Show<A> instance matches the inferred call arguments",
    display:
      "SES-T0201: A required trait instance is not available\nno Show<A> instance matches the inferred call arguments",
  },
] as const

export const publishedDiagnostics: readonly {
  id: string
  sourcePath: string
  code: string
  message?: string
  detail?: string
}[] = [
  ...[
    "pilot-function-application",
    "pilot-currying",
    "syntax-reader-pipelines",
  ].map((id) => ({
    id: `${id}-invalid`,
    sourcePath: `apps/site/examples/invalid/src/language/${id}.ssrg`,
    code: "SES-T0101",
  })),
  ...typeDiagnostics,
]

export const articleExecutions = [
  ...compositionExamples.map((example) => ({
    ...example,
    route: `/docs/composition/${example.id.slice("composition-".length)}/`,
  })),
  ...effectExamples,
  ...typeExamples,
] as const

export const publishedExecutions = [
  ...functionChapterExamples.map((example) => ({
    ...example,
    sourcePath: `apps/site/examples/src/language/${example.id}.ssrg`,
  })),
  ...compositionExamples,
  ...effectExamples,
  ...typeExamples,
  {
    id: "pilot-function-application",
    sourcePath:
      "apps/site/examples/src/language/pilot-function-application.ssrg",
    output: "3\n",
  },
  {
    id: "pilot-currying",
    sourcePath: "apps/site/examples/src/language/pilot-currying.ssrg",
    output: "3, 3\n",
  },
  {
    id: "syntax-reader-pipelines",
    sourcePath: "apps/site/examples/src/language/syntax-reader-pipelines.ssrg",
    output: "[> build, > bundle]\nTrue\nJust > build\nNothing\n",
  },
  {
    id: "first-run-hello",
    sourcePath: "examples/samples/hello-world/main.ssrg",
    output: "Hello, Seseragi!\n",
  },
] as const
