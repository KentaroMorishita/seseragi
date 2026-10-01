import { canonicalExample } from "./canonical-example"

// Exact identities and verified results; prose remains in typed locale modules.
export const unicodeReaderCases = [
  {
    slug: "at",
    name: "at",
    module: "std/text/grapheme",
    identity: "std/text/grapheme::at",
    route: "/docs/library/text/grapheme/function/at/",
    output: "found: 👍🏽\nfound: é\nnot found\nnot found\nnot found\n",
  },
  {
    slug: "byteboundaries",
    name: "byteBoundaries",
    module: "std/text/grapheme",
    identity: "std/text/grapheme::byteBoundaries",
    route: "/docs/library/text/grapheme/function/byteboundaries/",
    output: "boundaries: [0, 1, 9, 12]\nUTF-8 bytes: 12\nempty: [0]\n",
  },
  {
    slug: "casefold",
    name: "caseFold",
    module: "std/text",
    identity: "std/text::caseFold",
    route: "/docs/library/text/function/casefold/",
    output:
      "lowercase equal: False\nfolded equal: True\nfolded text: strasse\nsame API result: True\n",
  },
  {
    slug: "clusters",
    name: "clusters",
    module: "std/text/grapheme",
    identity: "std/text/grapheme::clusters",
    route: "/docs/library/text/grapheme/function/clusters/",
    output: "pieces: [A, 👍🏽, é]\nsame text: True\nempty: []\n",
  },
  {
    slug: "fullcasefold",
    name: "fullCaseFold",
    module: "std/text/unicode",
    identity: "std/text/unicode::fullCaseFold",
    route: "/docs/library/text/unicode/function/fullcasefold/",
    output:
      "folded: strasse\noriginal: Straße\nsame comparison: True\ndotted I: i̇\nsigmas: σσ\naccent scalars: 2\nempty: []\n",
  },
  {
    slug: "isnormalized",
    name: "isNormalized",
    module: "std/text/unicode",
    identity: "std/text/unicode::isNormalized",
    route: "/docs/library/text/unicode/function/isnormalized/",
    output:
      "before: False\nafter: True\noriginal still decomposed: True\nempty: True\n",
  },
  {
    slug: "length",
    name: "length",
    module: "std/text/grapheme",
    identity: "std/text/grapheme::length",
    route: "/docs/library/text/grapheme/function/length/",
    output: "visible pieces: 3\nscalars: 5\nUTF-8 bytes: 12\nempty: 0\n",
  },
  {
    slug: "normalize",
    name: "normalize",
    module: "std/text/unicode",
    identity: "std/text/unicode::normalize",
    route: "/docs/library/text/unicode/function/normalize/",
    output:
      "NFC: é\nNFD scalars: 2\nNFKC: ffi1\nNFKD: 1\nidempotent: True\noriginal scalars: 2\n",
  },
  {
    slug: "slice",
    name: "slice",
    module: "std/text/grapheme",
    identity: "std/text/grapheme::slice",
    route: "/docs/library/text/grapheme/function/slice/",
    output:
      "preview: [A👍🏽]\npreview: [👍🏽é]\npreview: []\ninvalid: InvalidGraphemeRange { start: 0, end: 4, length: 3 }\ninvalid: InvalidGraphemeRange { start: 2, end: 1, length: 3 }\ninvalid: InvalidGraphemeRange { start: -1, end: 2, length: 3 }\npreview: []\n",
  },
  {
    slug: "tolower",
    name: "toLower",
    module: "std/text",
    identity: "std/text::toLower",
    route: "/docs/library/text/function/tolower/",
    output: "ordinary: hello\nGreek: ος οσα σ\ndotted I: i̇\nempty: []\n",
  },
  {
    slug: "toupper",
    name: "toUpper",
    module: "std/text",
    identity: "std/text::toUpper",
    route: "/docs/library/text/function/toupper/",
    output:
      "ordinary: HELLO\nexpanded: STRASSE\nligature: FFI\naccent: É\nempty: []\n",
  },
  {
    slug: "version",
    name: "version",
    module: "std/text/unicode",
    identity: "std/text/unicode::version",
    route: "/docs/library/text/unicode/function/version/",
    output: "Unicode: 17.0.0\n",
  },
] as const

export const unicodeTypeScriptCases = [
  {
    slug: "graphemes",
    output:
      "visible pieces: 3\nscalars: 5\nUTF-8 bytes: 12\nUTF-16 units: 7\nempty: 0\n",
  },
  {
    slug: "normalization",
    output:
      "NFC: é\nNFD scalars: 2\nNFKC: ffi1\nNFKD: 1\noriginal scalars: 2\nlowercase: straße\nuppercase: STRASSE\n",
  },
] as const

export function unicodeReaderExamples(playgroundUrl: string) {
  return [
    ...unicodeReaderCases.map(({ slug }) =>
      canonicalExample(
        `unicode-${slug}`,
        `apps/site/examples/src/api-unicode/${slug}.ssrg`,
        playgroundUrl
      )
    ),
    ...unicodeTypeScriptCases.map(({ slug }) =>
      canonicalExample(
        `unicode-${slug}-typescript`,
        `apps/site/examples/src/api-unicode/${slug}.ts`,
        playgroundUrl,
        false,
        "typescript"
      )
    ),
  ]
}
