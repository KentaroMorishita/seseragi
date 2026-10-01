# Syntax / evaluation reader verification (#707)

Verified 2026-10-01 for [#707](https://github.com/KentaroMorishita/seseragi/issues/707).
This is implementation evidence and bounded agent reading review. It does not
claim a first-time human reader, visual/browser acceptance, or a full-site pass.
The ten syntax routes below and the evaluation route retain their existing IDs,
EN/JA paths, and language-reference ownership. Function application remains owned
by the earlier values/functions pilot.

## Executable article coverage

Each primary program is stored once at
`apps/site/examples/src/language/syntax-reader-<key>.ssrg`. Its corresponding
rejection is under `apps/site/examples/invalid/src/language/` with the same
basename. `scripts/syntax-examples.ts` supplies canonical metadata through the
existing helper; article modules reference IDs rather than copying source text.
The rejected metadata uses `standalone=false` and has no runnable Playground link.

| Key | Route under `/docs/language/` | Exact native stdout | Rejected diagnostic |
| --- | --- | --- | --- |
| source-text | syntax/source-text/ | `Aki: 42 (primary)\n` | SES-P0101 |
| literals | syntax/literals/ | `3, 2.5, True, S, Seseragi\n` | SES-P0203 |
| escapes | syntax/character-string-escapes/ | `"Seseragi"\ndocs\\guide\nλ\nfirst\nsecond\n` | SES-P0201 |
| layout | syntax/layout-and-line-continuation/ | `7, 7\n` | SES-P0001 |
| names | syntax/reserved-words-and-names/ | `Hello, Aki\n` | SES-N0001 |
| optional-fields | syntax/optional-record-fields/ | `Aki, A\n` | SES-T0101 |
| methods | syntax/method-calls/ | `10\n15\n` | SES-T0503 |
| pipelines | syntax/pipelines-and-low-precedence-application/ | `7\n7\n7\n` | SES-T0101 |
| precedence | syntax/operator-precedence/ | `7, 9, True\n` | SES-T0101 |
| custom-operators | syntax/custom-operators/ | `Seseragi docs\n` | SES-P0001 |
| evaluation | expressions/evaluation/ | `23\n` | SES-T0101 |

The native test saves each program as an independent `main.ssrg` in a fresh
temporary directory. All eleven `lint` and `run` invocations succeed with exact
stdout; all eleven invalid files fail lint with status 2 and the stated code.
The eleven documented repairs also run successfully. The precedence repair uses
`println ordered`, whose direct Bool output is `true\n`; the primary example
uses template interpolation and outputs `True`. The test preserves this actual
formatting difference rather than changing the compiler's behavior.

Every primary Playground URL decodes to the exact source bytes and compiles with
the committed WASM driver. Executing the generated module through the existing
Playground runtime matches the native result (the capture API trims its final
newline). This tests the source/compiler/runtime boundaries without a browser.
Metadata hashes and token highlights are also checked against the file bytes.

## Reading structure and preserved rules

Each article presents its purpose, one complete program and output, a local
reading of the notation, rules, a relevant rejection and correction, and related
reading. Existing supplementary forms and rejection panels remain after the
basic example. `reader-details.ssrg` only inserts existing semantic Blocks;
it is not a second renderer or a new content format.

- Source text explains UTF-8, Unicode identifier categories, apostrophes,
  line comments, case sensitivity, and the distinction between identifier and
  type errors. The names article explicitly scopes lowercase to cased letters;
  uncased Japanese identifiers such as `次の値` are valid value/function names.
  Types/constructors/traits still require an uppercase initial character.
- Literals retain radix and separator rules, Float notation, safe Int bounds,
  Bool, Char, String, templates, List-versus-template syntax, and Unit.
  Escape examples contain actual escaped quote, backslash, Unicode character,
  and newline spellings; the output panel contains the corresponding characters.
- Layout and pipeline examples preserve actual multiline pipelines. Tests
  reject accidental flattening of the source. Delimiters, continuation tokens,
  explicit semicolons, and indentation rules are explained separately.
- Optional fields connect missing fields to `Maybe` and `??`; method calls
  connect the receiver's static type, `impl`, `self`, and immutable return values.
- Precedence starts with arithmetic/comparisons, then lists each of the thirteen
  levels from 9 through -3 with exact operators and associativity. Level 4
  distinguishes left-grouping `+`/`-` from right-grouping `:`. The relation of
  `infixr 4` to that same level and conflicting grouping directions is explicit.
  Short circuiting and function references remain documented.
- Custom operators retain spelling, whitespace, fixity, export/import, and
  ambiguity rules, with a local bridge from an ordinary two-input function.
- Evaluation uses a concrete calculation to explain strict left-to-right
  evaluation. Value/storage identity is a later detail, not the main example.

The precedence list was checked against `docs/spec/01-syntax.md` §1.7,
`crates/seseragi-syntax/src/surface/expression.rs`,
`crates/seseragi-syntax/src/standard_operator.rs`, and
`crates/seseragi-semantics/src/resolve/body/operator_chains.rs`.
A same-level `infixr 4 <+>` plus `+` probe produces SES-P0102 without explicit
grouping; parenthesizing the right input runs and prints `Seseragi docs!`.
Unicode classification was checked in `lexer.rs::scan_identifier` and `unicode.rs`.
A program declaring `次の値`, calling a function named `表示` with parameter `値`,
and printing its result runs successfully with output `42\n`.

## Current compiler limitations are not language permissions

No compiler or normative-spec behavior was changed for these articles.
Minimal lint/build/run probes were checked against both current development
and published Linux v0.61.19 binaries:

- Some unused digit-leading/reserved declarations are accepted, as are missing
  parameter/return/public-value annotations found in the earlier pilot. These
  related conformance gaps are recorded with exact repros in
  [#709](https://github.com/KentaroMorishita/seseragi/issues/709).
- A comparison chain is rejected with SES-T0101, rather than being described
  as a currently observed parser diagnostic.
- `1__0` is rejected with SES-P0203, but the current message describes a range
  error. The page identifies the separator placement as the cause.
- Recovery around an ignored fragment after a semicolon and a missing closing
  parenthesis is recorded in the [#707 probe comment](https://github.com/KentaroMorishita/seseragi/issues/707#issuecomment-5931191242),
  alongside the existing #679 context. These probes are not taught as valid
  alternatives. The layout rejection uses a reproducibly rejected missing brace.

## Final checks and review limits

The final focused run used the official Linux x64 release CLI, version
`0.61.19 (release, commit a8641b5a81a4)`, with Bun 1.3.9. Earlier native/debug
checks used `0.61.19 (development, commit ab42da841d76)`. The documentation
checkout's base was `90cd575c1dc42566bb622b8e9baf794f5c71cf3b`; compiler/runtime
sources were unchanged. Release versus development provenance is not hidden by
the identical numeric version.

- `apps/site/tests/syntax-reader.test.ts`: **15 passed, 0 failed, 1,042 assertions**.
  This includes native examples/rejections/repairs, WASM execution, and 22 real
  production-module EN/JA renders. The final focused run took 18.32 seconds.
- Scoped TypeScript typecheck, Biome, changed-page Seseragi formatting, and
  `git diff --check` passed. Canonical multiline fixtures were not reformatted.
- `target/site-syntax-review` contains the final 22 HTML pages for reading.
  The render uses actual module import closures, `renderDocument`, canonical
  sources/highlights, and locale switches. Its small catalog has no sidebar
  areas: it does not prove full navigation, route closure, CSS/mobile layout,
  keyboard behavior, or deployed-site behavior.
- The independent agent reviewer read all 22 bodies and requested two precise
  corrections: exact precedence levels and uncased-name clarification. Both
  were made in EN/JA and guarded in the rendered test. Final rereading is tracked
  separately in the integration review ledger.
- The pinned yomiyasu guidance and two bounded advisory lint passes were used
  on rendered Japanese article prose, excluding code/output panels. The final
  report covers 11 Japanese pages. Remaining flags mainly concern repeated
  polite endings and necessary type/syntax contrasts. The precedence reference
  list intentionally contains thirteen entries; it is after the worked example,
  and was retained despite a list-density warning. Lint scores are not reader
  acceptance and were not optimized by deleting language rules.
- Browser, full-site generation, visual QA, and human first-reader acceptance
  were not run in this scoped task. Whole-site integration and lock regeneration
  are owned by the integration task; no push, deploy, release, or issue closure
  was performed here.
