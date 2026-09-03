# #306 Pinned Unicode, scalar text and graphemes

Base: `219cf853bb5f9641d5509a44699c7717f749938e` (v0.49.0).
#304's canonical release and local CLI/LSP/official extension synchronization
completed before this leaf. #291 and the next Regex, codecs, BigInt and Decimal
leaves were inspected; this change implements only #306 and its release boundary.

## Contract

- `std/text`, `std/char`, `std/text/unicode` and `std/text/grapheme` use the
  canonical standard-module interfaces, lowering, runtime feature metadata,
  embedded native runtime and manifest-generated browser resolver. Curried and
  first-class operations follow the same source-arity lowering as Bytes.
- String indices are Unicode scalar indices; UTF-8 byte lengths and extended
  grapheme cluster indices are separate APIs. Slices are checked, end-exclusive,
  copy their results and preserve the original normalization and leading BOM.
  Slice errors retain start, end and length in both Show and Debug.
- Literal search/replacement never invokes a regular expression. Empty needles
  operate at scalar boundaries, replacement dollars are literal, CRLF is one
  line separator, and words/trim use pinned White_Space (not ECMAScript's BOM
  whitespace behavior). Casing is default, locale-independent Unicode casing.
- `runtime/unicode/manifest.json` pins Unicode 17.0.0, eleven upstream data/test
  files and their SHA-256 digests. Generation and verification are offline by
  default. Normalization, properties, casing and UAX #29 segmentation do not
  delegate semantics to host ICU. Normalization handles all four forms and
  Hangul; graphemes include Indic conjunct, emoji-ZWJ and regional-indicator rules.
- Compiler XID/NFC dependency versions are pinned; generated Rust casing,
  uppercase and whitespace projections serve syntax, formatter/driver and LSP
  consumers. Version metadata, runtime tables, locks and browser/native artifacts
  agree. Unicode's license accompanies runtime, WASM, site, VSIX and native archive
  distributions; archive verification checks the exact notice content and VSIX
  verification requires the packaged notice.
- Every generated source module imports a lightweight version guard and calls it
  before its own source initializers. The guard name starts with `$`, which source
  identifiers cannot contain. Pure modules do not need to load the large tables.
  Generated metadata declares `unicodeVersion`; conformance validates it, and an
  incompatible runtime or imported module fails with a runtime ABI mismatch.
- Primitive String ABI is preserved. With the user's approval, the old `O(1)`
  UTF-8 byte-length promise is corrected to `O(n)`; no unbounded String cache or
  boxed-String ABI is introduced. Case contexts, normalization ordering and
  literal search avoid repeated scans on long adversarial input.

## Evidence

- Runtime tests consume all official NormalizationTest rows (all four forms and
  five input columns), check unlisted assigned scalars, all GraphemeBreakTest
  cases with UTF-8 offsets, and every default simple/full CaseFolding mapping.
  Separate tests disable host normalization/casing, exercise 40,000 combining
  marks and long Sigma contexts, and cover empty inputs, checked ranges, NUL,
  BOM, supplementary scalars, literal replacement and invalid Char code points.
- Rust tests compare all scalar XID/Uppercase/White_Space properties against the
  pinned data/dependencies. A newly added Unicode uppercase pair, combining
  identifier continuations and Final_Sigma are covered. LSP stdio tests verify
  canonical text/Char/grapheme/normalization signatures and a combining namespace
  alias, including UTF-16 cursor positions.
- `schema-1/unicode-text` exercises the full public surface, constructors,
  dictionaries, errors and first-class operations through actual source. Native
  execution produces 23 expected output/Console-trace lines with empty stderr.
  `project-schema-1/imported-unicode` additionally compiles and runs as a normal
  manifest/lock package; its two imported modules produce five expected lines.
  Generated TypeScript, source maps, tokens, CST, interfaces and metadata come
  from the canonical writers, not handwritten expected compiler output.
- Artifact tests stage those real compiled imported modules with the canonical
  runtime. Matching versions evaluate both source initializers; runtimes 16/18
  and a mismatched imported dependency reject before the incompatible module's
  source initialization. Test instrumentation observes initialization without
  replacing the generated/runtime guards. A user function bearing the old guard
  spelling cannot collide with the emitted binding.
- Lesson 28 runs through the ordinary CLI using the current `do {}` Effect
  syntax. The portable parity package includes Unicode text for native/LSP/WASM.
  Dedicated WASM tests execute the comprehensive single module and imported
  package, and reject a mutated dependency Unicode requirement in the actual
  browser bundle evaluator.
- Browser QA used installed `browser-use` with headless Chromium because
  `agent-browser` was unavailable. On a fresh local Vite server at 1710x1112,
  entering the canonical 23-line fixture, Run, then Format and Run again produced
  the native output. Completed returned with Run enabled, editor and Output both
  visibly nonzero-width, and no captured error/unhandled-rejection events. Both
  screenshots were visually inspected; the QA browser and server were closed.

## Explicit remaining boundaries

- Char values work through `fromCodePoint` and `scalarAt`, but character-literal
  syntax itself was already disconnected from lexer/semantics/lowering. The
  independently reproduced gap is tracked as #513 under the language-core Epic;
  it is not hidden by a fixture shim and does not reprioritize #291.
- General Show for Ordering is not added; the fixture renders its comparison
  result with an ordinary pure match. Existing generic inference (#503), HKT
  (#508), Effect-do match (#510), deriving and Regex boundaries remain separate.
- The pre-existing foreign `string` codec accepts an unpaired surrogate because
  it only checks the host primitive type. A real `invokeForeignPure` probe returns
  code point 55296, while the adapter file is unchanged from v0.49.0. This broken
  input invariant is tracked separately as #514 under Interop; raw `Js.String`
  and validated String must remain distinct. These text APIs require the
  specified scalar-only String, not arbitrary ill-formed host UTF-16.
- There is no locale collation, Regex implementation or #305 codec expansion.
  The older combined `bytes-and-unicode.ssrg` fixture still imports hex/Base64
  from #305; the new text-only compile fixture verifies this leaf independently.
  The portable property tables can support later Regex work without claiming it
  already exists. A module's version check precedes its own initializers; this
  is not a graph-wide preflight before compatible ESM dependencies initialize.

## Verification gate

`bun run check` is explicitly required by #306 and necessary for this
compiler/runtime/WASM and release-wide integration. Focused suites precede it.
Runtime/artifact tests passed 12 tests / 5,539 assertions; dedicated WASM tests
passed both cases, and LSP stdio passed all 19 tests. Unicode generated freshness
and release-version synchronization passed. Earlier full-gate failures exposed
old exact-output expectations and synthetic runtime metadata missing the new
required version; those fixtures were updated without relaxing their assertions.
The final full-gate and canonical release/dogfood results must be recorded from
successful execution, not inferred from these focused results.

`bun run check` passed on 2026-09-04 (`/tmp/seseragi-306-full-final3.log`):
all Rust workspace tests, canonical conformance (141 generated modules, 35
project compilations, 29 project executions, 105 single-module executions and
41 Analysis documents), runtime ABI and standard-library parity, native samples,
committed WASM freshness, release contracts/native archive smoke, all 546
Playground tests, TypeScript, the isolated production build and official/legacy
VSIX packaging and smoke verification. The earlier portable-parity import-list
expectation was updated for the new text imports; the complete final run passes
without filtering that test or relaxing its import-order assertion.

The generator also passed a focused Biome lint/format check. Canonical release
publication and all three installed components remain a post-merge handoff,
tracked on the Issue/PR rather than claimed from local development packages.
Narrow Git attributes preserve hash-pinned upstream UCD bytes (including its
original trailing whitespace) and the byte-exact empty-String rendering fixture;
no Unicode input or expected stdout was trimmed to satisfy whitespace checks.
