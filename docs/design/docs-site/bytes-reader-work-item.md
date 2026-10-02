# Binary payload and text encoding reader batch

## Reader task

An ordinary TypeScript developer needs to inspect a small binary payload, put it
through a text-only field as hex or Base64, and check it before treating the
result as text. Introduce a byte as an integer from 0 through 255. Teach
Uint8Array, TextEncoder and TextDecoder locally rather than assuming prior
binary-data experience. A successful hex/Base64 decode returns bytes; UTF-8
validation is a separate step.

Japanese is authored first, then English. Each deep link explains its example's
names and syntax. The sequence is a useful task, idiomatic TypeScript, Seseragi,
the observable benefit, and local syntax/rules. No Rust, Haskell, advanced type
system or functional-programming background is required.

## Exact existing scope

`apps/site/scripts/bytes-reader.ts` owns twenty existing identities:

- `std/bytes`
- `std/bytes::{Bytes, ByteError, fromInts, toInts, length}`
- `std/text::{Utf8DecodeError, encodeUtf8, decodeUtf8, decodeUtf8Lossy}`
- `std/bytes/hex`
- `std/bytes/hex::{HexDecodeError, encode, decode}`
- `std/bytes/base64`
- `std/bytes/base64::{Base64DecodeError, encode, decode, encodeUrl, decodeUrl}`

That is three modules, five opaque-type identities and twelve functions: forty
EN/JA bodies. The existing std/text overview is not counted again. Fourteen
canonical source triples supply a Seseragi program, a Bun-native TypeScript
comparison and a separately checked Node TypeScript alternative. Existing
constructor and instance pages remain outside this batch. No new route,
standard module, compiler signature or runtime behavior is introduced.

The family dispatcher guards owner, namespace, kind and canonical identity in
shallow branches. A narrowly scoped declaration-reading adapter explains the
four public error-pattern types without changing their canonical opaque kinds
or unrelated generated reading.

## Runtime and comparison boundaries

All four owning modules are supported on process and browser targets. Examples
are pure in-memory conversions with Console printing and no filesystem, HTTP,
navigation or storage service. Their exact sources can be checked through the
process CLI and the committed WASM/current-runtime Playground path separately.

The primary TypeScript codec panels are labelled Bun 1.3.9. That runtime provides
Uint8Array.toHex/fromHex/toBase64/fromBase64, and the installed Bun declarations
make them available to the TypeScript 5.8.3 comparison harness. Node v24.19.0 does
not provide those methods. Its labelled alternative uses node:buffer and runs
in its own lane. TypeScript compilation never supplies a missing runtime API.

Native codecs and UTF-8 encoding are already concise. The comparison does not
manufacture TypeScript boilerplate. Native UTF-8 failure is an exception without
a public byte-offset payload; Seseragi's error provides that offset. Error pages
show distinct, separately verified outputs. Strict canonical Base64 acceptance
requires an explicit policy around native decoding, even with strict trailing
chunk handling. Node Buffer decoding is more permissive and gets the required
validation instead of being falsely equated with the Seseragi decoder.

## Contracts to retain

- fromInts checks values in source order and reports the first invalid integer;
  it does not wrap, truncate, or return a partial successful prefix
- Bytes is immutable through the language surface; this is not Object.freeze
  applied to an exposed mutable JavaScript typed array
- toInts copies; length counts bytes, not text characters
- Strict UTF-8 reports the start of the first invalid byte sequence. Lossy
  decoding substitutes U+FFFD and does not establish validity
- TextDecoder comparisons use ignoreBOM:true where needed to match preserved
  leading U+FEFF. Explain this only beside the relevant text examples
- Hex output is lowercase; input can use uppercase. UTF-8 byte-length parity is
  checked before digits, and length errors contain a length rather than an offset
- Standard Base64 is padded when needed and uses +/. URL-safe output uses -_ and
  omits padding. Each decoder has its own accepted spelling
- Whitespace, wrong alphabets/padding and nonzero unused trailing bits are
  rejected. Mixed-error priority is not universally the leftmost problem
- Codec errors contain zero-based byte offsets; successful binary decoding does
  not imply UTF-8, business-data validity, encryption or authentication

## Verification and acceptance

Keep exact-source process compilation/execution, strict Bun/Node comparisons,
committed-WASM seed execution, runtime boundary probes, precise rejection/repair
diagnostics, production-body rendering and independent reading as distinct
categories of evidence. Modified source must be rerun rather than inheriting an
older feasibility result. The verification document records actual outcomes.

Whole-site integration, deployment, real-browser checks and genuine first-time
human feedback remain separate from bounded authoring and independent model
reading. Compiler/runtime fixes and public issue creation are outside scope.
