# Char, whitespace and Unicode-property reader verification

## Bounded scope

Base checkpoint: `afe906eee87dd0b588730a578554851f0e2404ac`.
Thirteen existing page identities, with 26 Japanese/English bodies:

- `/docs/library/char/`
- `/docs/library/char/function/{codepoint,fromcodepoint,tostring}/`
- `/docs/library/text/function/{trimstart,trimend,words}/`
- `/docs/library/text/unicode/function/{generalcategory,isalphabetic,isdecimaldigit,ismark,iswhitespace,simplecasefold}/`

Japanese routes add `/ja`. This removes twelve remaining generic callable
explanations from the bounded Text-family inventory; it does not accept every
Text route. Existing Text/Unicode/grapheme introductions and prior overlays are
preserved. isEmpty and the three UTF-8 operation pages remain separate work.
No compiler, runtime, specification or canonical reference metadata changes
belong to this batch.

## Ownership and semantic sources

Typed explanation ownership is `src/reference/editorial/char-text-reader/`:
page/en/ja modules for each leaf and Char module, with a typed shared model and
an exact module/identity/value-namespace/function-kind dispatcher. The existing
semantic Block renderer remains the only article renderer.

`apps/site/scripts/char-text-reader.ts` provides twelve exact identity/result
records and 24 canonical descriptors: one complete Seseragi and one complete
TypeScript program per operation under `examples/src/api-char-text/`. The Char
module reuses these verified sources. Canonical source, hash, highlighting and
Playground seeds stay connected; TS panels have no Seseragi Playground seed.
Parent owns shared catalogs, build/check registries, aggregate imports and locks.

Normative/runtime traceability:

- `docs/spec/01-syntax.md` §1.2.1: one-scalar Char literals, escapes and diagnostics
- `docs/spec/02-types.md`: immutable scalar/String definitions
- `docs/spec/10-library-surface.md` §10.7: Char conversions, White_Space
  trimming/splitting, punctuation and empty-input rules, text costs
- The same specification's `std/text/unicode` section: category alternatives,
  single-scalar predicates, simple/full folding, declaration-order Ord and
  pinned Unicode contract
- `runtime/unicode/manifest.json`: Unicode 17.0.0 and pinned data files
- `runtime/ts/src/{char,text,unicode,unicode-case,unicode-properties}.ts`
- `scripts/generate-unicode.ts`: derived properties and C/S folding mappings
- `runtime/ts/tests/unicode.test.ts`: existing behavior used for traceability

## Reader and comparison choices

The first Char example reads an ordinary symbol's number before introducing
Maybe. fromCodePoint locally explains Just, Nothing, typed parameters and
match. Surrogate and two-scalar literal mistakes are distinguished from a
perfectly valid but currently unassigned scalar. String-preserving repairs do
not quietly normalize a decomposed spelling.

Whitespace pages start with ordinary padding or a short sentence, then show
NEL/FEFF boundaries and empty input. Punctuation remains visible through JSON
array display. words is not described as language-aware word segmentation.

Property pages accept one Char and explain that unit locally. Alphabetic is
broader than ASCII or category Letter; Nd does not parse a number; Mn/Mc/Me do
not cover every component of a grapheme; White_Space excludes some invisible
characters. A False result is ordinary data. Full category details and Ord
ordering follow the initial concrete result.

TS comparisons use the same known one-scalar inputs or the same whitespace
strings. They preserve ordinary built-in solutions and explain specific limits:
String.fromCodePoint accepts a surrogate; trim/regex whitespace differs from
Unicode White_Space; a property test returns Bool, not a 29-way category; /iu
comparison does not return a simple-folded Char. Full category lookup can use
category-specific property tests or Unicode data/library code; no external
library is claimed mandatory for a task JS properties can express.

Selected host results are verified independently of version reporting. Bun
reports Unicode 15.1 and ICU 75.1 here, while a property regex recognizes a
Unicode-17 character. Those fields do not establish the Unicode version of
every host operation. The prose does not invent an engine mismatch from them
or claim universal Unicode equivalence from these examples.

## Verification procedure and current state

Toolchain: published Linux CLI `seseragi 0.61.19 (release, commit a8641b5a81a4,
target x86_64-unknown-linux-gnu)`, Bun 1.3.9, repository TypeScript/Biome, and the
committed Playground WASM adapter. Native examples are copied into isolated
/tmp folders so a changing examples-package lock is not mistaken for a snippet
failure.

Dedicated test: `apps/site/tests/char-text-reader.test.ts`.

It covers canonical bytes/hash/highlights/seeds; twelve native lint/run pairs
with exact stdout including newline; the same twelve programs through WASM and
the existing runtime harness; all twelve strict-checked/executed TS counterparts;
six complete invalid/repair cases; exact dispatcher guards; and the production
reference renderer. Expected diagnostic families are SES-T0101 for String/Char
mismatch, SES-P0202 for a two-scalar Char literal, and SES-P0201 for a surrogate
escape. Missing values and predicate results are not called compiler errors.

The repair for the surrogate input displays the returned Maybe using show.
An initial test-only direct println of that object produced `[object Object]`;
explicit Show formatting fixes the test wrapper. No displayed program or
compiler/runtime contract was changed to fit the expected output.

The focused renderer is designed to emit 38 bodies: 26 new bodies plus twelve
unchanged controls (existing Text, Unicode and grapheme module bodies; concat,
fullCaseFold and isEmpty, each localized). Every authored page-name anchor is
checked across the entire new article body against the rendered target H1;
missing targets fail, expected per-page counts are fixed, and decorated-label
mutations must fail. The expected total is 44 links, with purposes adjacent.
Generated API indexes and section-fragment wording have their own boundaries.

The earlier Unicode test used isMark as a no-overlay control. Because this
batch intentionally authors isMark, the parent authorized a test-only switch
to still-untouched Text.isEmpty, retaining signature/no-overlay checks and the
same 34-body count. No existing explanation was removed to satisfy that test.

After parent-owned production registration, the complete focused command was:

```sh
source ../seseragi-env.sh
SESERAGI_BIN="$(cat /tmp/seseragi-first-run-699-path)/bin/seseragi" \
  CHAR_TEXT_RENDER_DIR=target/site-char-text-review \
  bun test apps/site/tests/char-text-reader.test.ts
```

Result: **18 tests passed / 1,067 assertions / 25.09 seconds**. Retained HTML is
under `target/site-char-text-review/` with actual route directories and
`index.html`. The 38-body count, 26 new bodies, 12 unchanged controls and 44
authored page-name links all passed. Each edited body has the exact canonical
native and TS source panels, declared output, signature and same-identity
locale link.

The authorized earlier-Unicode control regression also passed:
`bun test apps/site/tests/unicode-reader.test.ts -t Fourteen` with the same CLI
produced **1 test / 762 assertions / 18.53 seconds**, with 17 unrelated tests
filtered out. Its 34 bodies retain the same coverage while the negative
control is now Text.isEmpty.

Scoped TypeScript checking with the site gate flags, Biome over the new helper,
tests and all twelve TS examples, all 53 new Seseragi file format checks, and
diff whitespace checking pass. No compiler/runtime/specification files were
changed.

## Reading evidence

The author read all 26 complete rendered bodies, including the generated
signature/footer sections and native/TS panels. Reading extracts are retained
as `target/char-text-{ja,en}-reading.txt`; the prose-only grouped extracts are
additional review aids, not substitutes for the original HTML. Before rendering,
the generalCategory comparison was corrected to acknowledge that JS can
combine category-specific property tests; Unicode data or a library is an
alternative, not a mandatory dependency for that task.

The foreground Japanese advisory run is retained at
`target/char-text-yomiyasu.json`: 42 sentence-ending notices, 12 emoji notices
and six technical-alternative notes over 13 articles. The emoji are the actual
Unicode inputs under explanation, and scalar/byte/grapheme, Maybe/failure and
JS-whitespace distinctions are meaningful. No rule, result or example was
weakened to chase a prose score. One advisory pass was used.

A separate agent read all 26 edited bodies sequentially in Japanese and
English, including examples, output, declarations and comparison limits. It
reported no reader-blocking correction. Its independent structural checks
covered 44 exact-title authored anchors, 28 canonical native/Playground matches,
26 TS panels, 54 output panels and 184 local fragments, with no errors. The
twelve control bodies were excluded from new reader acceptance.

The exact reviewed-file manifest is retained at
`/tmp/char-text-independent-reviewed26.sha256`, and the independent structural
record at `/tmp/char-text-independent-review-structure.json`. Manifest paths
are relative to `target/site-char-text-review/` and sorted lexicographically.
Each UTF-8 line is the lowercase SHA-256 of the complete HTML bytes, two ASCII
spaces, the relative path and LF. The SHA-256 of those complete manifest bytes
is `db12e585b1ff98550e1adb0423d6cfabbae78c7135ecfd526fa66e427c3b2523`.
The author rechecked that all 26 retained files still match those hashes before
freezing this batch. This is independent agent text/HTML review, not actual
human first-time-reader or browser-layout acceptance.

## Limits

Focused selected-module generation is not complete-route/link/navigation
closure. WASM execution through the runtime harness is not an actual browser
session. Browser/layout checks remain blocked and no human first-time-reader
acceptance is claimed. Full generation, locks and integration are parent-owned.
No public issue/comment, push, commit, release, deployment or issue closure is
part of this authoring work.
