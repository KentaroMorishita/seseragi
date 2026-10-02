# JSON boundary reader batch

## Reader task

An ordinary TypeScript developer reads notification settings, checks the required
name and retry count, and writes chosen application fields back to JSON text.
Every page starts with that kind of recognizable task, presents a real TypeScript
solution, shows Seseragi, identifies the concrete benefit, and explains new syntax
locally. Functional-programming terms, algebraic data types, match, and trait
knowledge are not prerequisites.

The first example intentionally reports only ready/rejected settings. Detailed
errors, exact-number behavior, duplicate keys and field-order caveats appear on
the pages where they change the task's contract, after the useful case.

## Exact existing scope

`apps/site/scripts/json-reader.ts` owns the twenty identities, exact routes,
namespace/kind guards, source IDs and expected output. This batch covers:

- `std/json`
- `std/json::{Json,Decoder,Encoder,DecodeError,JsonParseError,JsonReadError}`
- `std/json::{parse,stringify,encodeString,decodeString,field,optionalField,array,record,map}`
- `std/prelude::{JsonEncode,JsonDecode}`
- `std/prelude::JsonEncode::encodeJson`
- `std/prelude::JsonDecode::decodeJson`

The twenty existing identities have forty locale bodies and use twelve canonical
Seseragi programs with twelve TypeScript counterparts. No new route, compiler
signature, runtime behavior or normative rule is introduced. The distinct
`std/json::JsonEncode` and `std/json::JsonDecode` instance pages are not part of
this batch; explaining their purpose does not count them as authored coverage.

## Behavior and boundaries

- Parse text, check a selected application type, and write JSON are separate steps
- TypeScript parses to unknown and performs real validation; its type annotations
  are not presented as runtime validation
- Derived settings reject missing/unknown fields and wrong types; product rules,
  such as a nonnegative retry limit, still need explicit checks
- Missing fields report their parent path and name in the reason; invalid present
  fields add a path segment for the failing child
- optionalField distinguishes absence from present null according to its child
  decoder; empty strings are present
- array stops at the first failed element; record is a homogeneous declared field
  set returning ordered pairs, with unknown-key checks before child decoding
- map transforms only success and does not catch programming defects
- DecodeError's path/kind are readable, but its constructor and named struct
  pattern are private. Current JSON error/path types have no Eq/Show/Debug
  instances; examples format their public data explicitly
- Small fixed numeric examples do not establish universal Int/number equivalence.
  Duplicate keys, exact decimal values, UTF-8 offsets and integer-looking key
  ordering have bounded, explicitly separate evidence
- All displayed programs use in-memory values. Only output uses Console; no file,
  network or storage provider is needed

The exact site reading adapter clarifies public JSON constructors in Japanese,
DecodeError field access in both languages, and record's array-of-pairs argument.
A symbol-aware kind label corrects only DecodeError's Japanese label. It preserves canonical declarations,
all unrelated readings and English for the three Japanese-only adjustments.

## Verification and acceptance

Native/strict-TS output checks, exact committed-WASM source-seed execution,
negative compile fixtures, runtime callback tests, whole-body rendering and
independent reading are separate requirements. The work's verification document
records their actual results and remaining gates. Whole-site integration,
publication and real desktop/mobile Playground checks remain separate from this
bounded authoring work. Actual first-time-reader feedback is not replaced by
agent reading or automatic assertions.

## Deliberate backlog

Remaining JSON constructors, DecodeErrorKind/JsonPathSegment leaves, index,
oneOf, flatMap, and twenty-eight related codec instances retain their existing
coverage status. Custom error construction is not silently enabled to simplify a
docs example. Runtime/compiler repairs and public bug reports are outside scope.
