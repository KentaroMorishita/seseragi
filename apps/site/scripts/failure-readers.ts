import { canonicalExample } from "./canonical-example"

export const failureReaders = [
  {
    path: "pure-expressions",
    example: "pure",
    output: "60",
    stage: "lint",
    diagnostic: "SES-T0101",
  },
  {
    path: "maybe",
    example: "maybe",
    output: "Aki\nanonymous\nempty:",
    stage: "lint",
    diagnostic: "SES-T0101",
  },
  {
    path: "either",
    example: "either",
    output: "ok: 2\nerror: must be positive",
    stage: "lint",
    diagnostic: "SES-T0101",
  },
  {
    path: "effect-type",
    example: "effect",
    output: "before\nrun\nrun",
    stage: "lint",
    diagnostic: "SES-T0101",
  },
  {
    path: "effect-functions-contract-form",
    example: "contract",
    output: "ready",
    stage: "lint",
    diagnostic: "SES-E0001",
  },
  {
    path: "effect-functions-inferred-form",
    example: "inferred",
    output: "ready",
    stage: "lint",
    diagnostic: "SES-P0001",
  },
  {
    path: "runtime-boundaries",
    example: "boundary",
    output: "only main",
    stage: "run",
    diagnostic: "`main` must be public",
  },
  {
    path: "error-channels",
    example: "errors",
    output: "Aki\nfallback: missing",
    stage: "lint",
    diagnostic: "SES-E0001",
  },
  {
    path: "environment-requirements",
    example: "environment",
    output: "Hello, Aki",
    stage: "lint",
    diagnostic: "SES-T0101",
  },
  {
    path: "defects",
    example: "defects",
    output: "3\ncannot divide by zero",
    stage: "run",
    diagnostic: "runtime defect",
  },
] as const

export function failureReaderExamples(playgroundUrl: string) {
  return [
    ...failureReaders.flatMap((entry) => [
      canonicalExample(
        `failure-reader-${entry.example}`,
        `apps/site/examples/src/language/failure-reader-${entry.example}.ssrg`,
        playgroundUrl
      ),
      canonicalExample(
        `failure-reader-${entry.example}-invalid`,
        `apps/site/examples/${entry.stage === "lint" ? "invalid/src/language" : "runtime-errors"}/failure-reader-${entry.example}.ssrg`,
        playgroundUrl,
        false
      ),
    ]),
    canonicalExample(
      "failure-reader-positive-typescript",
      "apps/site/examples/comparisons/positive-result/typescript.ts",
      playgroundUrl,
      false,
      "typescript"
    ),
  ]
}
