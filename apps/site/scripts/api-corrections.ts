import { canonicalExample } from "./canonical-example"

// Exact identities, canonical paths, and executable expectations only.
export const apiCorrectionCases = [
  {
    slug: "map-get",
    path: "map/get",
    identity: "std/map::get",
    kind: "function",
    expected: "found: [Aki]\nfound: []\nmissing\nmissing",
    typescriptExpected: "found: [Aki]\nfound: []\nmissing\nmissing",
    standalone: true,
    typescriptStandalone: true,
  },
  {
    slug: "state-get",
    path: "transformer/state/get",
    identity: "std/transformer/state::get",
    kind: "function",
    expected: "value=7, state=7\nvalue=0, state=0",
    typescriptExpected: "value=7, state=7\nvalue=0, state=0",
    standalone: true,
    typescriptStandalone: true,
  },
  {
    slug: "queue-take",
    path: "queue/take",
    identity: "std/queue::take",
    kind: "effect-function",
    expected: "10\n20\nclosed",
    typescriptExpected: "10\n20\nclosed",
    standalone: true,
    typescriptStandalone: false,
  },
  {
    slug: "storage-get",
    path: "web/storage/get",
    identity: "std/web/storage::get",
    kind: "effect-function",
    expected:
      'host: "Aki" -> found: [Aki]\nhost: "" -> found: []\nhost: null -> missing\nhost: SecurityError("denied by mock") -> denied by mock',
    typescriptExpected:
      'host: "Aki" -> found: [Aki]\nhost: "" -> found: []\nhost: null -> missing\nhost: SecurityError("denied by mock") -> denied by mock',
    standalone: false,
    typescriptStandalone: false,
  },
  {
    slug: "nonempty-head",
    path: "non-empty-list/head",
    identity: "std/non-empty-list::head",
    kind: "function",
    expected: "10\n10",
    typescriptExpected: "10\n10",
    standalone: true,
    typescriptStandalone: true,
  },
  {
    slug: "nonempty-tail",
    path: "non-empty-list/tail",
    identity: "std/non-empty-list::tail",
    kind: "function",
    expected: "`[]\n`[20, 30]",
    typescriptExpected: "[]\n[20,30]",
    standalone: true,
    typescriptStandalone: true,
  },
  {
    slug: "nonempty-tolist",
    path: "non-empty-list/tolist",
    identity: "std/non-empty-list::toList",
    kind: "function",
    expected: "`[10]\n`[10, 20, 30]\n10",
    typescriptExpected: "[10]\n[10,20,30]\n10",
    standalone: true,
    typescriptStandalone: true,
  },
  {
    slug: "set-toarray",
    path: "set/toarray",
    identity: "std/set::toArray",
    kind: "function",
    expected: "[2, 1]\n[1, 2]\n[]\n[2, 1]",
    typescriptExpected: "[2,1]\n[1,2]\n[]\n[2,1]",
    standalone: true,
    typescriptStandalone: true,
  },
  {
    slug: "set-tolist",
    path: "set/tolist",
    identity: "std/set::toList",
    kind: "function",
    expected: "`[2, 1]\n`[1, 2]\n`[]\n`[2, 1]",
    typescriptExpected: "[2,1]\n[1,2]\n[]\n[2,1]",
    standalone: true,
    typescriptStandalone: true,
  },
] as const

export function apiCorrectionExamples(playgroundUrl: string) {
  return apiCorrectionCases.flatMap((item) => [
    canonicalExample(
      `api-correction-${item.slug}`,
      `apps/site/examples/src/api-corrections/${item.slug}.ssrg`,
      playgroundUrl,
      item.standalone
    ),
    canonicalExample(
      `api-correction-${item.slug}-ts`,
      `apps/site/examples/comparisons/api-corrections/${item.slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}
