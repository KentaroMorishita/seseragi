# #307 Portable regular-expression semantics

Base: `2a01ee4c05bc8de81e2e53823d5d5033a453abf8` (v0.50.0).
#306's canonical release and local CLI/LSP/official extension synchronization
completed before this leaf. This change implements only #307 and its v0.51.0
release boundary; it does not advance the remaining #291 queue items.

## Contract

- `std/regex` is an available portable standard module backed by an immutable
  opaque `Regex`. `compile` and `compileWith` return typed
  `Either<RegexCompileError, Regex>` values; ordinary invalid patterns do not
  throw or become runtime defects. Error offsets are zero-based UTF-8 bytes in
  the pattern.
- The runtime parses the specified literal, escape, dot, class, alternation,
  capture, named-capture, greedy quantifier and anchor syntax into an ordered
  counted Thompson NFA. It never invokes or wraps JavaScript `RegExp`.
  Backreferences, look-around, atomic groups, conditionals, recursion, inline
  flags and lazy or possessive quantifiers return
  `UnsupportedRegexFeature`.
- Search is leftmost-first. At one start position, earlier alternatives win and
  greedy quantifiers consume as much as the selected ordered path permits.
  Counted repetitions keep compact counters rather than expanding `{m,n}` into
  pattern-sized instruction copies. Nullable repetitions terminate, preserve
  the last participating capture and do not iterate a huge empty count.
- Patterns and inputs are matched as Unicode scalar sequences. Match and capture
  spans are zero-based, end-exclusive UTF-8 byte offsets. Captures remain in
  opening-parenthesis order without group zero; named captures retain declaration
  order and retain `Nothing` for a group that did not participate. Returned text
  is copied from scalar-safe boundaries rather than retaining a large source.
- `findAll`, `split` and both replacements use non-overlapping source order.
  After an empty match they advance exactly one Unicode scalar, including the
  terminal empty match. `replaceAll` treats dollar and backslash characters as
  literal text; capture-aware replacement is explicit through `replaceAllWith`.
  `escape` emits a literal fragment, including unambiguous NUL handling.
- `\\d` is ASCII, while `\\s`, `\\w`, `\\p{Property}` and
  `\\P{Property}` use the pinned Unicode 17.0.0 property data introduced by
  #306. Case-insensitive matching uses the pinned default simple-fold
  equivalence class, including property/class complements, and is independent
  of host ICU. Existing generated-module Unicode guards reject incompatible
  runtime data before source initialization.
- Regex records and error kinds provide their specified Eq/Ord/Show evidence,
  plus the normal Debug companion used by diagnostics and executable evidence.
  The compiler-owned interface, lowering operations, runtime ABI, embedded
  native package, manifest-generated browser registry, Reference catalog and
  LSP completion all share the same canonical identities.

## Evidence

- `runtime/ts/tests/regex.test.ts` covers typed UTF-8 compile errors, duplicate
  names and excluded constructs; leftmost alternative and greedy counted
  behavior; nullable/repeated captures; named captures and multibyte spans;
  Unicode word/digit/space/property and simple-fold behavior; CRLF multiline and
  absolute anchors; empty-match progress; literal/callback replacement; escaping;
  and a 40,000-scalar ambiguous failure. It is part of the full conformance lane,
  not an ad-hoc test that future `bun run check` executions skip.
- An additional development oracle compared 270 supported ASCII pattern/input
  match and capture cases against the equivalent host behavior and found no
  mismatches. This was only comparative review evidence: production matching
  does not delegate to that host engine.
- `schema-1/regex` exercises the full public surface, public constructors,
  records and dictionaries through ordinary Seseragi source. Its generated
  TypeScript, source map, typed stages and Analysis document come from the
  canonical fixture writer. `execution-schema-1/regex` runs the result through
  the CLI/conformance environment and fixes 16 exact output/Console-trace lines
  with empty stderr.
- The portable standard-parity project imports `std/regex`, compiles a pattern
  and executes literal replacement through normal manifest/lock package routes.
  Compiler, driver, LSP and stdlib parity tests assert the complete module,
  runtime import and Reference signatures. All 42 Analysis artifacts were
  regenerated because the shared catalog now includes the Regex category.
- Browser QA used `browser-use` against a fresh local Vite server. A source
  program importing `std/regex` analyzed without diagnostics and completed Run
  with exact `order <number>` output. Reference search returned the opaque type,
  records, constructors and operations. The editor and output remained visible,
  Run returned to its enabled state, and the browser/server were closed.
- The canonical Playground resolver contains 94 browser-capable modules including
  `@seseragi/runtime/regex`. The committed v0.51.0 WASM was regenerated through
  `bun run build:playground:wasm` and passed the freshness check. Native archive
  and official/legacy VSIX packaging both executed their version/architecture
  smoke checks.

## Explicit remaining boundaries

- This leaf does not add regex literal syntax or any host-specific PCRE/JS
  extension. Word-boundary assertions are outside the specified required syntax
  and are reported as unsupported rather than inheriting host behavior.
- The implementation is the shared TypeScript runtime engine used by current
  process and browser products. There is no second backend with an independently
  drifting regular-expression implementation; compiler/runtime/WASM/Playground
  routes all select this same portable ABI contract.
- Lesson 23 remains the design curriculum source. Removing Regex from the Tour
  exclusion list allows the now-real import without manufacturing a second
  executable lesson or changing course order. The next #291 queue leaf remains
  separate and must wait for v0.51.0 release and installed dogfood proof.

## Verification gate

`bun run check` is explicitly required by #307 because this change crosses the
compiler, runtime, WASM, Playground, LSP and release boundaries. The final run on
2026-09-04 passed after the dedicated Regex suite was made a permanent part of
the conformance lane: 11 Regex tests / 44 assertions; all Rust workspace tests;
canonical conformance with 42 Analysis, 142 GeneratedModule, 35 ProjectCompile,
29 ProjectExecution and 106 Execution fixtures; 24 native samples, three Web
packages, 146 Tour lessons and 148 exercises/diagnostics; committed WASM
freshness; release contract/native archive checks; all 546 Playground tests /
12,720 assertions; isolated TypeScript/Vite production build; and official plus
legacy VSIX packaging and execution smoke.

The first full attempt exposed provider-backed fixture locks still naming the
v0.50.0 toolchain runtime. Every affected lock was refreshed with the v0.51.0
official CLI, including PostgreSQL, SQLite, browser and process provider fixtures;
no build was allowed to rewrite a stale lock implicitly. A later freshness run
correctly required the regenerated WASM and package metadata to be staged. The
final complete run passed without filtering tests or relaxing expected output.
Canonical release publication and installed CLI/LSP/extension dogfood remain the
post-merge handoff and must not be inferred from local development artifacts.
