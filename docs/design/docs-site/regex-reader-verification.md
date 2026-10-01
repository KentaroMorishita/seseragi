# Regex reader verification — eleven existing pages

This local batch, based on `78c3ff955b8194e0efa9f1eb04b4ca12a28f9533`, covers the
existing `std/regex` module page and ten callable identities under #601/#630.
It preserves routes, titles, namespace, kinds, signatures, constraints and
canonical compiler metadata. No compiler/runtime change, public issue/comment,
remote write, browser session or deployment is part of this work.

## Scope and reading path

The module starts by extracting `ORD-42` from a message. Its complete program
introduces compilation, `Left`/`Right`, `find`, `Just`/`Nothing`, and the matched
text before formal matching rules. The callable pages are:

- `compile`, `compileWith`, `defaultOptions`, `escape`, `isMatch`
- `find`, `findAll`, `split`, `replaceAll`, `replaceAllWith`

Every callable retains an exact declaration panel. `find` now describes a
`RegexMatch` record with text, byte span and optional captures, rather than the
upstream collection-style “first matching element” description. The upstream
reference JSON remains unchanged; its misleading description is a separate
metadata debt. The other already-specific descriptions receive concrete local
explanations rather than being counted as generic-description fixes.

Typed EN/JA copy and pages are under
`apps/site/src/reference/editorial/regex`. The isolated `regex-catalog.ssrg`
requires exact module, identity, value namespace and function kind. Parent-owned
registration integrates its callable dispatcher and module blocks. The module
selection is made as `Maybe<Editorial>` before the existing `editorialBlocks`
helper; this avoids the observed nominal-type conflict from separately inferring
an empty `Array<Block>` beside an imported record-field projection. No type or
compiler behavior was changed to accommodate the helper.

## Canonical examples and TypeScript comparisons

`apps/site/scripts/regex-readers.ts` describes eleven complete Seseragi programs
and eleven TypeScript counterparts. Sources are under
`apps/site/examples/src/api-regex` and
`apps/site/examples/comparisons/api-regex`. All eleven Seseragi programs are
process-compatible, with exact seeded-source Playground links. TypeScript
panels have no Seseragi launch links. Both actual outputs are displayed, including
differences rather than normalized false equivalence.

The examples establish:

- Typed compile rejection, UTF-8 compile-error offset 3 for `é[`, unsupported
  look-ahead, and the difference between source-string and regex escaping
- All three Boolean options, simple rather than expanding full case folding,
  line matching, dot-newline behavior and all-false defaults
- Literal punctuation search; the TypeScript escape-page bridge uses the simpler
  `String.includes` task and explicitly does not produce a regex fragment
- ASCII `\d` versus Seseragi's Unicode `\w`; the JS final word-character result
  intentionally differs
- `find` over `é ORD-42`: Seseragi bytes `3..9` versus JavaScript UTF-16 `2..8`
- Capture slots excluding group 0, nested optional lookup, named absent captures,
  and direct text extraction without mixing byte and UTF-16 indexing
- Non-overlap, leftmost/alternative priority, greediness and zero-width progress:
  the empty pattern on `a👍` yields byte positions `0,1,5` rather than UTF-16 `0,1,3`
- Dot matching the two scalar components of `👍🏽`, not one visible grapheme
- Split preserving edge/adjacent empty pieces and not injecting captured
  separators; its JavaScript counterpart deliberately shows different results
- Literal `$1`, `$&` and backslash replacement; JavaScript uses a callback for
  the same literal task and separately shows its replacement-string expansion
- Pure capture-dependent replacement, explicit absent capture handling, source
  order, unchanged unmatched text, and a visibly labeled empty-input result

The TS examples create fresh regexes. `test`/`exec` examples avoid `g` and `y`;
`matchAll`/replacement examples use explicit `gu`. No stateful repeated-exec loop
or timing-dependent assertion is used. JavaScript RegExp is a comparison, not
the specification or implementation of Seseragi's portable engine.

An initial RegExp.escape comparison executed on Bun, but the repository's
TypeScript 5.8.3 definitions lack that API. The final published comparison uses
ordinary `String.includes` without installing a dependency, changing compiler
configuration, or supplying a pretend polyfill. Its limited task is stated in
both locales.

## Executed checks

Tests use Bun 1.3.9 and the official Linux CLI 0.61.19, release commit
`a8641b5a81a4`. The compiler/runtime source match was established at checkpoint4;
this batch edits neither area.

```sh
source ../seseragi-env.sh
export SESERAGI_BIN="$(cat /tmp/seseragi-first-run-699-path)/bin/seseragi"
REGEX_READER_RENDER_DIR=/tmp/seseragi-regex-rendered \
  bun test apps/site/tests/regex-readers.test.ts
```

The complete scoped suite passed **6 tests / 987 assertions**. Paired look-ahead
and simple-versus-full-folding clarifications were then added; each current
22-body rerender passed **1 test / 763 assertions**. Execution sources were
unchanged by those prose clarifications. Exact native and TypeScript source/output records were
also recaptured after the final fixture changes.

Coverage includes:

- CLI lint/run of every standalone canonical program, exact stdout/stderr and
  source-seed equality. Temporary single-file directories avoid modifying the
  shared package locks while other authors work
- Strict typecheck/run of every exact TS counterpart with the repository's
  existing TypeScript compiler
- Compilation of every source seed through the committed WASM compiler and
  execution through the existing runtime harness, with matching captured stdout
- Direct runtime contract probes for rejected backreferences, look-around,
  inline flags, lazy/possessive quantifiers, atomic groups, conditionals,
  recursion and external `\b`/`\B` assertions
- Distinct absent versus empty participating captures; terminal empty matches;
  scalar byte spans; split captures/empties; literal replacement markers;
  callback source order and no rescanning of inserted text
- Two real function-context type rejections, both `SES-T0101`: supplying String
  where Regex is required, and supplying an Effect-returning replacement where
  `RegexMatch -> String` is required. The published compiled-pattern and pure
  String-callback examples are executable repairs
- Fifty exact/negative dispatcher probes, including wrong module, namespace,
  kind and same-name foreign identity
- Twenty-two actual production bodies plus two untouched RegexSpan control
  pages. Tests check exact declarations, outputs, canonical source panels,
  scalar locale fields, every paired paragraph and absence of the opposite
  locale's paragraphs, local headings, module links and preserved controls

Every subprocess is bounded to 30 seconds; focused production closures retain
the existing 90-second command bound. No repository lock was changed by this
worker. Parent integration owns the complete catalog, locks and full build.

Scoped static checks passed: Biome over 13 files, helper/test TypeScript checking,
Seseragi format checks over 46 owned files, and diff whitespace. The advisory Japanese prose
pass before the final folding clarification covered the eleven owned bodies and reported 39 sentence-ending repetitions, three
technical negative contrasts, five emoji findings from the actual Unicode
examples, and one generated public-API list finding. These advisory findings
were reviewed without changing verified examples or removing required distinctions.

## Rendered review evidence and limits

Current HTML is retained at `/tmp/seseragi-regex-rendered`. The author self-read
the local explanations and current Japanese bodies. A 22-body route-shaped copy,
execution logs, per-source hashes and advisory report are under
`/workspace/shared/seseragi-regex-authoring`. The RegexSpan controls are excluded
from that 22-body review copy.

Independent reading covered all 22 complete bodies. The sole requested revision
was the compileWith simple/full folding sentence, now corrected and reread in
both locales: simple folding does not expand ß to ss, while full folding maps
Straße to strasse. No reader blocker remains. All 22 seeded links match their
first source panel and local fragments resolve. The reader’s sorted-path/NUL/
HTML-bytes/NUL digest is
`f01233dc185c8999d1ac1b9e96f4a13015ca07508425cb350fa410c31fa034a9`.
The independently maintained reader ledger records this bounded acceptance. WASM/runtime execution
is not a browser interaction or visual check. Browser execution remains blocked;
no workaround, publication, deployment, issue closure or complete-site acceptance
is claimed by the scoped tests.
