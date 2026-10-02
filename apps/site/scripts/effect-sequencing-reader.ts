import { canonicalExample } from "./canonical-example"

// Native and committed-WASM source programs share these exact observations.
export const effectSequencingReaderCases = [
  {
    slug: "succeed-fail",
    output: "Ready: 42\nRejected: not approved\nCaller continues\n",
  },
  {
    slug: "validate-preview",
    output:
      "Ready: Release: Search improvements\nRejected: title is required\nRejected: template is missing\nReady: : Search improvements\n",
  },
  {
    slug: "from-maybe",
    output: "Count: 2\nCount: 0\nRejected: count is missing\n",
  },
  {
    slug: "map-error",
    output:
      "Ready: Search improvements\nRejected: preview: title is required\n",
  },
  {
    slug: "recover",
    output: "Title: Release\nTitle: Preview\nRejected: invalid setting\n",
  },
  {
    slug: "defer",
    output:
      "Before execution\nPreview: Search improvements\nPreview: Search improvements\n",
  },
  {
    slug: "deferred-unrun",
    output: "Constructed without executing\n",
  },
  {
    slug: "loop-control",
    output:
      "Continue permits the next item\nBreak stops successfully\nBreak stops successfully\n",
  },
  {
    slug: "foreach-until",
    output:
      "visit: one\nvisit: stop\nCompleted\nvisit: one\nvisit: two\nCompleted\nCompleted\nvisit: stop\nCompleted\nvisit: one\nvisit: bad\nRejected: bad label\n",
  },
] as const

// Existing compiler identities only; no new reference routes.
export const effectSequencingReaderRoutes = [
  ["std/effect", "module", "module", "module", "validate-preview"],
  [
    "std/effect::succeed",
    "value",
    "effect-function",
    "succeed",
    "succeed-fail",
  ],
  ["std/effect::fail", "value", "effect-function", "fail", "succeed-fail"],
  [
    "std/effect::fromEither",
    "value",
    "effect-function",
    "fromeither",
    "validate-preview",
  ],
  [
    "std/effect::fromMaybe",
    "value",
    "effect-function",
    "frommaybe",
    "from-maybe",
  ],
  [
    "std/effect::attempt",
    "value",
    "effect-function",
    "attempt",
    "succeed-fail",
  ],
  ["std/effect::mapError", "value", "effect-function", "maperror", "map-error"],
  ["std/effect::recover", "value", "effect-function", "recover", "recover"],
  ["std/effect::defer", "value", "effect-function", "defer", "defer"],
  [
    "std/effect::forEachUntil",
    "value",
    "effect-function",
    "foreachuntil",
    "foreach-until",
  ],
  [
    "std/effect::LoopControl",
    "type",
    "opaque-type",
    "loopcontrol",
    "loop-control",
  ],
  ["std/effect::Continue", "value", "constructor", "continue", "loop-control"],
  ["std/effect::Break", "value", "constructor", "break", "loop-control"],
].map(([identity, namespace, kind, slug, example]) => ({
  identity: identity!,
  module: "std/effect",
  namespace: namespace!,
  kind: kind!,
  slug: slug!,
  name: identity!.split("::")[1] ?? "",
  example: example!,
  route:
    namespace === "module"
      ? "/docs/library/effect/"
      : `/docs/library/effect/${kind}/${slug}/`,
}))

export const effectSequencingReaderFailures = [
  {
    slug: "narrow-result",
    diagnostics: ["SES-T0101", "Either<Never, Int>", "Either<String, Never>"],
    output: "Ready: 42\nRejected: not approved\nCaller continues\n",
  },
  {
    slug: "mixed-do",
    diagnostics: ["SES-T0101", "Do monad constructor mismatch"],
    output: "Preview: Search improvements\n",
  },
  {
    slug: "break-as-effect",
    diagnostics: ["SES-T0101"],
    output: "stopped successfully\n",
  },
  {
    slug: "fromeither-string",
    diagnostics: ["SES-T0101"],
    output: "Search improvements\n",
  },
] as const

export function effectSequencingReaderExamples(playgroundUrl: string) {
  return effectSequencingReaderCases.flatMap(({ slug }) => [
    canonicalExample(
      `effect-sequencing-${slug}`,
      `apps/site/examples/src/effect-sequencing/${slug}.ssrg`,
      playgroundUrl
    ),
    canonicalExample(
      `effect-sequencing-${slug}-ts`,
      `apps/site/examples/comparisons/effect-sequencing/${slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}
