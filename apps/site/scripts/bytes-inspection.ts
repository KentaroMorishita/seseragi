import { canonicalExample } from "./canonical-example"

export const bytesInspectionCases = [
  {
    slug: "validate-one-byte",
    output: "Byte: 0\nByte: 255\nOut of range: -1\nOut of range: 256\n",
  },
  {
    slug: "empty-and-singleton",
    output: "Empty\nBytes: [65]\n",
  },
  {
    slug: "inspect-byte",
    output: "Index 0: 0\nIndex 2: 255\nIndex 3: missing\nIndex -1: missing\n",
  },
  {
    slug: "checked-range",
    output:
      "[20, 30]\n[]\nInvalid range 3..1 for 4 bytes\nInvalid range -1..2 for 4 bytes\nInvalid range 0..5 for 4 bytes\n[10, 20, 30, 40]\n",
  },
  {
    slug: "append-chunk",
    output:
      "[10, 20, 30, 40]\n[10, 20, 30, 40]\n[30, 40, 10, 20]\n[10, 20]\n[30, 40]\n",
  },
  {
    slug: "join-chunks",
    output: "[10, 20, 30]\n[]\n[10, 20]\n[30]\n",
  },
] as const

export const bytesInspectionRoutes = [
  {
    identity: "std/bytes::Byte",
    module: "std/bytes",
    namespace: "type",
    kind: "opaque-type",
    name: "Byte",
    slug: "byte-type",
    example: "validate-one-byte",
    route: "/docs/library/bytes/opaque-type/byte/",
  },
  {
    identity: "std/bytes::BytesSliceError",
    module: "std/bytes",
    namespace: "type",
    kind: "opaque-type",
    name: "BytesSliceError",
    slug: "bytessliceerror",
    example: "checked-range",
    route: "/docs/library/bytes/opaque-type/bytessliceerror/",
  },
  {
    identity: "std/bytes::byte",
    module: "std/bytes",
    namespace: "value",
    kind: "function",
    name: "byte",
    slug: "byte",
    example: "validate-one-byte",
    route: "/docs/library/bytes/function/byte/",
  },
  {
    identity: "std/bytes::toInt",
    module: "std/bytes",
    namespace: "value",
    kind: "function",
    name: "toInt",
    slug: "toint",
    example: "validate-one-byte",
    route: "/docs/library/bytes/function/toint/",
  },
  {
    identity: "std/bytes::empty",
    module: "std/bytes",
    namespace: "value",
    kind: "function",
    name: "empty",
    slug: "empty",
    example: "empty-and-singleton",
    route: "/docs/library/bytes/function/empty/",
  },
  {
    identity: "std/bytes::singleton",
    module: "std/bytes",
    namespace: "value",
    kind: "function",
    name: "singleton",
    slug: "singleton",
    example: "empty-and-singleton",
    route: "/docs/library/bytes/function/singleton/",
  },
  {
    identity: "std/bytes::isEmpty",
    module: "std/bytes",
    namespace: "value",
    kind: "function",
    name: "isEmpty",
    slug: "isempty",
    example: "empty-and-singleton",
    route: "/docs/library/bytes/function/isempty/",
  },
  {
    identity: "std/bytes::get",
    module: "std/bytes",
    namespace: "value",
    kind: "function",
    name: "get",
    slug: "get",
    example: "inspect-byte",
    route: "/docs/library/bytes/function/get/",
  },
  {
    identity: "std/bytes::slice",
    module: "std/bytes",
    namespace: "value",
    kind: "function",
    name: "slice",
    slug: "slice",
    example: "checked-range",
    route: "/docs/library/bytes/function/slice/",
  },
  {
    identity: "std/bytes::append",
    module: "std/bytes",
    namespace: "value",
    kind: "function",
    name: "append",
    slug: "append",
    example: "append-chunk",
    route: "/docs/library/bytes/function/append/",
  },
  {
    identity: "std/bytes::concat",
    module: "std/bytes",
    namespace: "value",
    kind: "function",
    name: "concat",
    slug: "concat",
    example: "join-chunks",
    route: "/docs/library/bytes/function/concat/",
  },
] as const

export function bytesInspectionExamples(playgroundUrl: string) {
  return bytesInspectionCases.flatMap(({ slug }) => [
    canonicalExample(
      `bytes-inspection-${slug}`,
      `apps/site/examples/src/bytes-inspection/${slug}.ssrg`,
      playgroundUrl
    ),
    canonicalExample(
      `bytes-inspection-${slug}-ts`,
      `apps/site/examples/src/bytes-inspection/${slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}
