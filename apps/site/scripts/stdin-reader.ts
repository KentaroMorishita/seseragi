import { canonicalExample } from "./canonical-example"

// Native readline comparisons cover these finite, valid LF/CRLF inputs.
// Only the exact Prelude source has a browser Playground input proof.
export const stdinReaderCases = [
  {
    slug: "qualified-one-line",
    portable: false,
    input: "parcel\nignored\n",
    output: "Line: parcel\n",
  },
  {
    slug: "three-lines",
    portable: false,
    input: "parcel\n\n",
    output: "Line: parcel\nBlank line\nEOF\n",
  },
  {
    slug: "limited-lines",
    portable: false,
    input: "abcd\nabcde\nok\n",
    output: "Line: abcd\nLine exceeds 4 bytes\nLine: ok\n",
  },
  {
    slug: "limit-config",
    portable: false,
    input: "",
    output:
      "Use at least 1 byte: 0\nUse at least 1 byte: -1\nAccepted\nAccepted\nUse at most 67108864 bytes: 67108865\n",
  },
  {
    slug: "default-limit",
    portable: false,
    input: "parcel\n",
    output: "Line accepted\n",
  },
  {
    slug: "one-line",
    portable: true,
    input: "parcel\nignored\n",
    output: "Line: parcel\n",
  },
] as const

// Owner is explicit: the two Prelude identities are cataloged under std/stdin.
export const stdinReaderRoutes = [
  [
    "std/stdin",
    "std/stdin",
    "module",
    "module",
    "stdin-module",
    "qualified-one-line",
  ],
  [
    "std/prelude::Stdin",
    "std/stdin",
    "type",
    "opaque-type",
    "stdin-type",
    "three-lines",
  ],
  [
    "std/prelude::StdinError",
    "std/stdin",
    "type",
    "opaque-type",
    "stdin-error",
    "three-lines",
  ],
  [
    "std/stdin::readLine",
    "std/stdin",
    "value",
    "effect-function",
    "readline",
    "three-lines",
  ],
  [
    "std/stdin::LineLimit",
    "std/stdin",
    "type",
    "opaque-type",
    "line-limit",
    "limited-lines",
  ],
  [
    "std/stdin::lineLimit",
    "std/stdin",
    "value",
    "function",
    "line-limit-create",
    "limit-config",
  ],
  [
    "std/stdin::defaultLineLimit",
    "std/stdin",
    "value",
    "function",
    "default-line-limit",
    "default-limit",
  ],
  [
    "std/stdin::readLineWith",
    "std/stdin",
    "value",
    "effect-function",
    "readline-with",
    "limited-lines",
  ],
  [
    "std/stdin::StdinConfigError",
    "std/stdin",
    "type",
    "opaque-type",
    "stdin-config-error",
    "limit-config",
  ],
  [
    "std/stdin::InvalidStdinUtf8",
    "std/stdin",
    "value",
    "constructor",
    "invalid-stdin-utf8",
    "three-lines",
  ],
  [
    "std/stdin::StdinLineTooLong",
    "std/stdin",
    "value",
    "constructor",
    "stdin-line-too-long",
    "limited-lines",
  ],
  [
    "std/prelude::readLine",
    "std/prelude",
    "value",
    "function",
    "prelude-readline",
    "one-line",
  ],
].map(([identity, owner, namespace, kind, slug, example]) => {
  const name = identity!.split("::")[1] ?? owner!
  const modulePath = owner!.slice("std/".length)
  return {
    identity: identity!,
    owner: owner!,
    module: owner!,
    name,
    namespace: namespace!,
    kind: kind!,
    slug: slug!,
    example: example!,
    route:
      namespace === "module"
        ? `/docs/library/${modulePath}/`
        : `/docs/library/${modulePath}/${kind}/${name.toLowerCase()}/`,
  }
})

export function stdinReaderExamples(playgroundUrl: string) {
  return stdinReaderCases.flatMap((item) => [
    canonicalExample(
      `stdin-reader-${item.slug}`,
      `apps/site/examples/src/stdin-reader/${item.slug}.ssrg`,
      playgroundUrl,
      item.portable
    ),
    canonicalExample(
      `stdin-reader-${item.slug}-ts`,
      `apps/site/examples/src/stdin-reader/${item.slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}
