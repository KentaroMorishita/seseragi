import { canonicalExample } from "./canonical-example"

export const typeLimits = [
  {
    path: "erasure-and-runtime-representation",
    example: "erasure",
    output: "43\nready!",
    diagnostic: "SES-N0001",
  },
  {
    path: "generic-impls-and-methods",
    example: "methods",
    output: "43\nready!",
    diagnostic: "SES-T0101",
  },
  {
    path: "kinds",
    example: "kinds",
    output: "Just 42\nNothing\n[1, 2]",
    diagnostic: "SES-T0604",
  },
  {
    path: "let-polymorphism-and-rank",
    example: "rank",
    output: "42\nready\nJust 42",
    diagnostic: "SES-T0101",
  },
  {
    path: "recursive-declarations",
    example: "recursion",
    output: "True\nFalse\nTrue",
    diagnostic: "SES-N0001",
  },
  {
    path: "requirement-merge",
    example: "requirements",
    output: "Hello!",
    diagnostic: "SES-E0001",
  },
  {
    path: "variance",
    example: "variance",
    output: "Aki\n[Aki]",
    diagnostic: "SES-T0101",
  },
] as const

export const typeLimitComparisons = [
  {
    example: "methods",
    directory: "generic-methods",
    output: "43\nready!",
    mutation: "\nconst wrong: string = number.get()\n",
    diagnostic: "TS2322",
  },
  {
    example: "variance",
    directory: "record-array-view",
    output: "Aki\n[Aki]",
    mutation: "\nconst wrong: Named[] = [{ name: 42 }]\n",
    diagnostic: "TS2322",
  },
] as const

export function typeLimitExamples(playgroundUrl: string) {
  return [
    ...typeLimits.flatMap((entry) => [
      canonicalExample(
        `type-limits-${entry.example}`,
        `apps/site/examples/src/language/type-limits-${entry.example}.ssrg`,
        playgroundUrl
      ),
      canonicalExample(
        `type-limits-${entry.example}-invalid`,
        `apps/site/examples/invalid/src/language/type-limits-${entry.example}.ssrg`,
        playgroundUrl,
        false
      ),
    ]),
    ...typeLimitComparisons.map((entry) =>
      canonicalExample(
        `type-limits-${entry.example}-typescript`,
        `apps/site/examples/comparisons/${entry.directory}/typescript.ts`,
        playgroundUrl,
        false,
        "typescript"
      )
    ),
  ]
}
