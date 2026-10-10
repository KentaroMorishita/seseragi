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

export const publishedExecutions = [
  ...functionChapterExamples.map((example) => ({
    ...example,
    sourcePath: `apps/site/examples/src/language/${example.id}.ssrg`,
  })),
  ...compositionExamples,
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
