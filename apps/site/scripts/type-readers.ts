import { canonicalExample } from "./canonical-example"

export const typeReaders = [
  {
    path: "type-system",
    example: "system",
    output: "1200",
    diagnostic: "SES-T0101",
  },
  {
    path: "optional-record-fields",
    example: "optional",
    output: "Aki\nA",
    diagnostic: "SES-T0101",
  },
  {
    path: "generic-aliases",
    example: "aliases",
    output: "42\nready",
    diagnostic: "SES-T0101",
  },
  {
    path: "newtypes",
    example: "newtypes",
    output: "user: 42",
    diagnostic: "SES-T0101",
  },
  {
    path: "type-identity-and-coercion",
    example: "coercion",
    output: "21.0",
    diagnostic: "SES-T0101",
  },
  {
    path: "generic-functions",
    example: "functions",
    output: "21, 21\nready, ready",
    diagnostic: "SES-T0101",
  },
  {
    path: "polymorphism",
    example: "polymorphism",
    output: "1\nAki",
    diagnostic: "SES-T0101",
  },
  {
    path: "type-parameter-scope",
    example: "scope",
    output: "42\nready, ready",
    diagnostic: "SES-N0001",
  },
  {
    path: "generic-structs",
    example: "structs",
    output: "43\n42!",
    diagnostic: "SES-T0101",
  },
  {
    path: "generic-adts",
    example: "adts",
    output: "ok: 42\nerror: missing",
    diagnostic: "SES-T0101",
  },
] as const

export const typeComparisons = [
  {
    example: "aliases",
    directory: "named-value",
    output: "42\nready",
    mutation:
      '\nconst wrong: Named<number> = { label: "count", value: "42" }\n',
    diagnostic: "TS2322",
  },
  {
    example: "functions",
    directory: "duplicate-value",
    output: "21, 21\nready, ready",
    mutation: "\nduplicate<string>(21)\n",
    diagnostic: "TS2345",
  },
  {
    example: "newtypes",
    directory: "distinct-id",
    output: "user: 42",
    mutation:
      '\nconst order: OrderId = { kind: "order", value: 42 }; userLabel(order)\n',
    diagnostic: "TS2345",
  },
] as const

export function typeReaderExamples(playgroundUrl: string) {
  return [
    ...typeReaders.flatMap((entry) => [
      canonicalExample(
        `type-reader-${entry.example}`,
        `apps/site/examples/src/language/type-reader-${entry.example}.ssrg`,
        playgroundUrl
      ),
      canonicalExample(
        `type-reader-${entry.example}-invalid`,
        `apps/site/examples/invalid/src/language/type-reader-${entry.example}.ssrg`,
        playgroundUrl,
        false
      ),
    ]),
    ...typeComparisons.map((entry) =>
      canonicalExample(
        `type-reader-${entry.example}-typescript`,
        `apps/site/examples/comparisons/${entry.directory}/typescript.ts`,
        playgroundUrl,
        false,
        "typescript"
      )
    ),
  ]
}
