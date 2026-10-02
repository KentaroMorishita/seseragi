import { canonicalExample } from "./canonical-example"

export const bytesReaderCases = [
  {
    slug: "text-to-portable-payload",
    output: "5\n636166c3a9\nY2Fmw6k=\nY2Fmw6k\ncafé\n",
    typescriptOutput: "5\n636166c3a9\nY2Fmw6k=\nY2Fmw6k\ncafé\n",
  },
  {
    slug: "validate-byte-input",
    output: "[0, 127, 255]\nOut of range: 256\nOut of range: -1\n[]\n",
    typescriptOutput:
      "[0, 127, 255]\nOut of range: 256\nOut of range: -1\n[]\n",
  },
  {
    slug: "inspect-binary-payload",
    output: "[0, 15, 255]\n3\n",
    typescriptOutput: "[0, 15, 255]\n3\n",
  },
  {
    slug: "encode-text-utf8",
    output: "41:1\n636166c3a9:5\nf09f8c8a:4\n:0\n",
    typescriptOutput: "41:1\n636166c3a9:5\nf09f8c8a:4\n:0\n",
  },
  {
    slug: "decode-text-utf8",
    output: "Text: café\nText: 🌊\nText: \nInvalid UTF-8\n",
    typescriptOutput: "Text: café\nText: 🌊\nText: \nInvalid UTF-8\n",
  },
  {
    slug: "explain-utf8-failure",
    output: "Invalid UTF-8 at byte 1\n",
    typescriptOutput: "Invalid UTF-8\n",
  },
  {
    slug: "preview-damaged-text",
    output: "a�(�\n",
    typescriptOutput: "a�(�\n",
  },
  {
    slug: "hexadecimal-payload",
    output: "000f10abff\n\n",
    typescriptOutput: "000f10abff\n\n",
  },
  {
    slug: "read-hexadecimal-payload",
    output:
      "Accepted: 00ff\nAccepted: \nRejected\nRejected\nRejected\nRejected\nRejected\nRejected\n",
    typescriptOutput:
      "Accepted: 00ff\nAccepted: \nRejected\nRejected\nRejected\nRejected\nRejected\nRejected\n",
  },
  {
    slug: "standard-base64-payload",
    output: "\nZg==\nZm8=\nZm9v\nZm9vYg==\nZm9vYmE=\nZm9vYmFy\n",
    typescriptOutput: "\nZg==\nZm8=\nZm9v\nZm9vYg==\nZm9vYmE=\nZm9vYmFy\n",
  },
  {
    slug: "read-standard-base64",
    output:
      "Accepted: 66\nAccepted: \nRejected\nRejected\nRejected\nRejected\nRejected\nRejected\nAccepted: 666f6f\n",
    typescriptOutput:
      "Accepted: 66\nAccepted: \nRejected\nRejected\nRejected\nRejected\nRejected\nRejected\nAccepted: 666f6f\n",
  },
  {
    slug: "url-safe-base64-payload",
    output:
      "-_8\nAccepted: fbff\nAccepted: \nRejected\nRejected\nRejected\nRejected\n",
    typescriptOutput:
      "-_8\nAccepted: fbff\nAccepted: \nRejected\nRejected\nRejected\nRejected\n",
  },
  {
    slug: "hex-decode-errors",
    output:
      "Decoded: 00ff\nOdd byte length: 1\nInvalid hex at byte 2\nOdd byte length: 3\n",
    typescriptOutput: "Decoded: 00ff\nInvalid hex\nInvalid hex\nInvalid hex\n",
  },
  {
    slug: "base64-decode-errors",
    output:
      "Decoded: 66\nInvalid byte length: 2\nInvalid digit at byte 2\nInvalid padding at byte 2\nUnused bits at byte 1\n",
    typescriptOutput:
      "Decoded: 66\nInvalid Base64\nInvalid Base64\nInvalid Base64\nInvalid Base64\n",
  },
] as const

export const bytesReaderRoutes = [
  {
    identity: "std/bytes",
    module: "std/bytes",
    namespace: "module",
    kind: "module",
    name: "std/bytes",
    slug: "bytes-module",
    example: "text-to-portable-payload",
    route: "/docs/library/bytes/",
  },
  {
    identity: "std/bytes::Bytes",
    module: "std/bytes",
    namespace: "type",
    kind: "opaque-type",
    name: "Bytes",
    slug: "bytes",
    example: "text-to-portable-payload",
    route: "/docs/library/bytes/opaque-type/bytes/",
  },
  {
    identity: "std/bytes::ByteError",
    module: "std/bytes",
    namespace: "type",
    kind: "opaque-type",
    name: "ByteError",
    slug: "byteerror",
    example: "validate-byte-input",
    route: "/docs/library/bytes/opaque-type/byteerror/",
  },
  {
    identity: "std/bytes::fromInts",
    module: "std/bytes",
    namespace: "value",
    kind: "function",
    name: "fromInts",
    slug: "fromints",
    example: "validate-byte-input",
    route: "/docs/library/bytes/function/fromints/",
  },
  {
    identity: "std/bytes::toInts",
    module: "std/bytes",
    namespace: "value",
    kind: "function",
    name: "toInts",
    slug: "toints",
    example: "inspect-binary-payload",
    route: "/docs/library/bytes/function/toints/",
  },
  {
    identity: "std/bytes::length",
    module: "std/bytes",
    namespace: "value",
    kind: "function",
    name: "length",
    slug: "length",
    example: "inspect-binary-payload",
    route: "/docs/library/bytes/function/length/",
  },
  {
    identity: "std/text::Utf8DecodeError",
    module: "std/text",
    namespace: "type",
    kind: "opaque-type",
    name: "Utf8DecodeError",
    slug: "utf8decodeerror",
    example: "explain-utf8-failure",
    route: "/docs/library/text/opaque-type/utf8decodeerror/",
  },
  {
    identity: "std/text::encodeUtf8",
    module: "std/text",
    namespace: "value",
    kind: "function",
    name: "encodeUtf8",
    slug: "encodeutf8",
    example: "encode-text-utf8",
    route: "/docs/library/text/function/encodeutf8/",
  },
  {
    identity: "std/text::decodeUtf8",
    module: "std/text",
    namespace: "value",
    kind: "function",
    name: "decodeUtf8",
    slug: "decodeutf8",
    example: "decode-text-utf8",
    route: "/docs/library/text/function/decodeutf8/",
  },
  {
    identity: "std/text::decodeUtf8Lossy",
    module: "std/text",
    namespace: "value",
    kind: "function",
    name: "decodeUtf8Lossy",
    slug: "decodeutf8lossy",
    example: "preview-damaged-text",
    route: "/docs/library/text/function/decodeutf8lossy/",
  },
  {
    identity: "std/bytes/hex",
    module: "std/bytes/hex",
    namespace: "module",
    kind: "module",
    name: "std/bytes/hex",
    slug: "hex-module",
    example: "hexadecimal-payload",
    route: "/docs/library/bytes/hex/",
  },
  {
    identity: "std/bytes/hex::HexDecodeError",
    module: "std/bytes/hex",
    namespace: "type",
    kind: "opaque-type",
    name: "HexDecodeError",
    slug: "hexdecodeerror",
    example: "hex-decode-errors",
    route: "/docs/library/bytes/hex/opaque-type/hexdecodeerror/",
  },
  {
    identity: "std/bytes/hex::encode",
    module: "std/bytes/hex",
    namespace: "value",
    kind: "function",
    name: "encode",
    slug: "hex-encode",
    example: "hexadecimal-payload",
    route: "/docs/library/bytes/hex/function/encode/",
  },
  {
    identity: "std/bytes/hex::decode",
    module: "std/bytes/hex",
    namespace: "value",
    kind: "function",
    name: "decode",
    slug: "hex-decode",
    example: "read-hexadecimal-payload",
    route: "/docs/library/bytes/hex/function/decode/",
  },
  {
    identity: "std/bytes/base64",
    module: "std/bytes/base64",
    namespace: "module",
    kind: "module",
    name: "std/bytes/base64",
    slug: "base64-module",
    example: "text-to-portable-payload",
    route: "/docs/library/bytes/base64/",
  },
  {
    identity: "std/bytes/base64::Base64DecodeError",
    module: "std/bytes/base64",
    namespace: "type",
    kind: "opaque-type",
    name: "Base64DecodeError",
    slug: "base64decodeerror",
    example: "base64-decode-errors",
    route: "/docs/library/bytes/base64/opaque-type/base64decodeerror/",
  },
  {
    identity: "std/bytes/base64::encode",
    module: "std/bytes/base64",
    namespace: "value",
    kind: "function",
    name: "encode",
    slug: "base64-encode",
    example: "standard-base64-payload",
    route: "/docs/library/bytes/base64/function/encode/",
  },
  {
    identity: "std/bytes/base64::decode",
    module: "std/bytes/base64",
    namespace: "value",
    kind: "function",
    name: "decode",
    slug: "base64-decode",
    example: "read-standard-base64",
    route: "/docs/library/bytes/base64/function/decode/",
  },
  {
    identity: "std/bytes/base64::encodeUrl",
    module: "std/bytes/base64",
    namespace: "value",
    kind: "function",
    name: "encodeUrl",
    slug: "base64-encodeurl",
    example: "url-safe-base64-payload",
    route: "/docs/library/bytes/base64/function/encodeurl/",
  },
  {
    identity: "std/bytes/base64::decodeUrl",
    module: "std/bytes/base64",
    namespace: "value",
    kind: "function",
    name: "decodeUrl",
    slug: "base64-decodeurl",
    example: "url-safe-base64-payload",
    route: "/docs/library/bytes/base64/function/decodeurl/",
  },
] as const

export function bytesReaderExamples(playgroundUrl: string) {
  return bytesReaderCases.flatMap(({ slug }) => [
    canonicalExample(
      `bytes-reader-${slug}`,
      `apps/site/examples/src/bytes-reader/${slug}.ssrg`,
      playgroundUrl
    ),
    canonicalExample(
      `bytes-reader-${slug}-ts`,
      `apps/site/examples/src/bytes-reader/${slug}.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
    canonicalExample(
      `bytes-reader-${slug}-node-ts`,
      `apps/site/examples/src/bytes-reader/${slug}.node.ts`,
      playgroundUrl,
      false,
      "typescript"
    ),
  ])
}
