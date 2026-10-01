import { canonicalExample } from "./canonical-example"

// Existing exact-fit sources are reused; only changed demonstrations get new IDs.
export const modelReaderCases = [
  {
    key: "expression",
    slug: "expression-oriented",
    exampleId: "principle-expression-oriented",
    source: "principle-expression-oriented",
    expectedOutput: "pass\n",
    expectedDiagnostic: "SES-T0101",
  },
  {
    key: "absence",
    slug: "no-hidden-danger",
    exampleId: "principle-no-hidden-danger",
    source: "principle-no-hidden-danger",
    expectedOutput: "not found\n",
    expectedDiagnostic: "SES-T0301",
  },
  {
    key: "backend",
    slug: "backend-independent-semantics",
    exampleId: "model-reader-backend",
    source: "model-reader-backend",
    expectedOutput: "-3, -3.5\n",
    expectedDiagnostic: "SES-T0201",
  },
  {
    key: "diagnostics",
    slug: "diagnosable-behavior",
    exampleId: "model-reader-diagnostics",
    source: "model-reader-diagnostics",
    expectedOutput: "3\n",
    expectedDiagnostic: "SES-T0101",
  },
  {
    key: "costs",
    slug: "visible-costs",
    exampleId: "principle-visible-costs",
    source: "principle-visible-costs",
    expectedOutput: "[2, 4, 6]\n",
    expectedDiagnostic: "",
  },
  {
    key: "density",
    slug: "readable-density",
    exampleId: "model-reader-density",
    source: "model-reader-density",
    expectedOutput: "7\n",
    expectedDiagnostic: "",
  },
].map((entry) => ({
  ...entry,
  id: `language.model.${entry.slug}`,
  route: `/docs/language/model/${entry.slug}/`,
  module: `pages/language/model/${entry.slug}/page`,
  sourcePath: `apps/site/examples/src/language/${entry.source}.ssrg`,
  invalidSourcePath: entry.expectedDiagnostic
    ? `apps/site/examples/invalid/src/language/model-reader-${entry.key}.ssrg`
    : "",
}))

export const modelReaderSupplement = {
  id: "principle-diagnosable-behavior",
  sourcePath:
    "apps/site/examples/src/language/principle-diagnosable-behavior.ssrg",
  expectedOutput: "`[]\n",
}

export function modelReaderExamples(playgroundUrl: string) {
  return [
    ...modelReaderCases.flatMap((entry) => [
      canonicalExample(entry.exampleId, entry.sourcePath, playgroundUrl),
      ...(entry.invalidSourcePath
        ? [
            canonicalExample(
              `model-reader-${entry.key}-invalid`,
              entry.invalidSourcePath,
              playgroundUrl,
              false
            ),
          ]
        : []),
    ]),
    canonicalExample(
      modelReaderSupplement.id,
      modelReaderSupplement.sourcePath,
      playgroundUrl
    ),
  ]
}

// Build registration adds these seven sources only, avoiding duplicate old IDs.
export function newModelReaderExamples(playgroundUrl: string) {
  return modelReaderExamples(playgroundUrl).filter((entry) =>
    entry.id.startsWith("model-reader-")
  )
}
