import { canonicalExample } from "./canonical-example"

// Example identities and expected results only. Prose is owned by typed locale modules.
export const textEditorialCases = [
  {
    name: "concat",
    slug: "concat",
    identity: "std/text::concat",
    expected: "[AB]\n[]",
  },
  {
    name: "join",
    slug: "join",
    identity: "std/text::join",
    expected: "[A,,B]\n[]\n[AB]",
  },
  {
    name: "split",
    slug: "split",
    identity: "std/text::split",
    expected: '["a","","b",""]\n[""]\n["A","😀"]\n[]',
  },
  {
    name: "contains",
    slug: "contains",
    identity: "std/text::contains",
    expected: "True\nFalse\nTrue\nFalse\nFalse",
  },
  {
    name: "startsWith",
    slug: "startswith",
    identity: "std/text::startsWith",
    expected: "True\nFalse\nTrue\nFalse",
  },
  {
    name: "endsWith",
    slug: "endswith",
    identity: "std/text::endsWith",
    expected: "True\nFalse\nTrue\nFalse",
  },
  {
    name: "replace",
    slug: "replace",
    identity: "std/text::replace",
    expected: "bXnana\nc$&t\n:AB\ncat\n[]",
  },
  {
    name: "replaceAll",
    slug: "replaceall",
    identity: "std/text::replaceAll",
    expected: "bXnXnX\nXa\n:A:😀:\nc$&t\n[]",
  },
  {
    name: "trim",
    slug: "trim",
    identity: "std/text::trim",
    expected: "[A B]\n[A]\n[]\n3\n[]",
  },
  {
    name: "lines",
    slug: "lines",
    identity: "std/text::lines",
    expected: '["a","","b"]\n[]\n[""]\n["a","b"]',
  },
  {
    name: "lengthScalars",
    slug: "lengthscalars",
    identity: "std/text::lengthScalars",
    expected: "5\n2\n0",
  },
  {
    name: "lengthBytes",
    slug: "lengthbytes",
    identity: "std/text::lengthBytes",
    expected: "10\n2\n0",
  },
  {
    name: "scalarAt",
    slug: "scalarat",
    identity: "std/text::scalarAt",
    expected: "found: 😀\nout of range\nout of range\nout of range",
  },
  {
    name: "sliceScalars",
    slug: "slicescalars",
    identity: "std/text::sliceScalars",
    expected:
      "slice: [é😀]\nslice: []\ninvalid: InvalidScalarRange { start: 0, end: 4, length: 3 }\ninvalid: InvalidScalarRange { start: 2, end: 1, length: 3 }\nslice: []",
  },
] as const

export const textUnitCase = {
  name: "units",
  slug: "units",
  expected: "scalars: 5\nUTF-8 bytes: 10\ngraphemes: 4",
} as const
export const textTypeScriptOutput =
  "UTF-16 units: 6\nscalars: 5\nUTF-8 bytes: 10\ngraphemes: 4"

export function textEditorialExamples(playgroundUrl: string) {
  return [
    ...[...textEditorialCases, textUnitCase].map(({ slug }) =>
      canonicalExample(
        `text-${slug}`,
        `apps/site/examples/src/api-text/${slug}.ssrg`,
        playgroundUrl
      )
    ),
    canonicalExample(
      "text-units-typescript",
      "apps/site/examples/src/api-text/units.ts",
      playgroundUrl,
      false,
      "typescript"
    ),
  ]
}
