import { canonicalExample } from "./canonical-example"

export const lifecycleReaders = [
  {
    path: "effectful-for",
    id: "language.effects.effectful-for",
    example: "for",
    output: "Aki\nMio\ndone",
    diagnostic: "SES-T0101",
  },
  {
    path: "task",
    id: "language.effects.task",
    example: "task",
    output: "[0, 1, 2]",
    diagnostic: "SES-T0101",
  },
  {
    path: "sequential-and-parallel",
    id: "language.effects.sequential-and-parallel",
    example: "parallel",
    output: "[10, 20]\n[10, 20]\n[]",
    diagnostic: "SES-T0101",
  },
  {
    path: "cancellation-and-resources",
    id: "language.effects.cancellation-resources",
    example: "resources",
    output: "42\nBA\nexpected\nBAC",
    diagnostic: "SES-T0101",
  },
  {
    path: "scheduler-fairness",
    id: "language.effects.scheduler-fairness",
    example: "fairness",
    output: "[3, 3]",
    diagnostic: "SES-T0101",
  },
  {
    path: "fiber-supervision",
    id: "language.effects.fiber-supervision",
    example: "fibers",
    output: "42\ntrue\n21",
    diagnostic: "SES-T0101",
  },
  {
    path: "signals-and-transactions",
    id: "language.effects.signals-transactions",
    example: "transactions",
    output: "[2, 12]\n12",
    diagnostic: "SES-T0101",
  },
  {
    path: "derived-signals",
    id: "language.effects.derived-signals",
    example: "derived",
    output: "22\n45",
    diagnostic: "SES-T0201",
  },
  {
    path: "signal-operators",
    id: "language.effects.signal-operators",
    example: "operators",
    output: "0\n5\n10",
    diagnostic: "SES-T0101",
  },
  {
    path: "subscriptions-and-lifetime",
    id: "language.effects.subscriptions-lifetime",
    example: "subscriptions",
    output: "[1, 2, 2]\n3",
    diagnostic: "SES-T0101",
  },
  {
    path: "exceptions-and-algebraic-effects",
    id: "language.effects.exceptions-algebraic",
    example: "exceptions",
    output: "ok: 2\nerror: must be positive",
    diagnostic: "SES-N0001",
  },
] as const

export function lifecycleReaderExamples(playgroundUrl: string) {
  return lifecycleReaders.flatMap((entry) => [
    canonicalExample(
      `lifecycle-reader-${entry.example}`,
      `apps/site/examples/src/language/lifecycle-reader-${entry.example}.ssrg`,
      playgroundUrl
    ),
    canonicalExample(
      `lifecycle-reader-${entry.example}-invalid`,
      `apps/site/examples/invalid/src/language/lifecycle-reader-${entry.example}.ssrg`,
      playgroundUrl,
      false
    ),
  ])
}
