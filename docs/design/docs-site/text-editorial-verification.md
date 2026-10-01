# Text editorial batch verification (#708)

## Scope

Fifteen existing page identities: `/docs/library/text/` and its fourteen
`function/` pages for concat, join, split, contains, startsWith, endsWith,
replace, replaceAll, trim, lines, lengthBytes, lengthScalars, scalarAt, and
sliceScalars. Slugs are lowercase, with paired `/ja/` routes.

Prose is owned by typed `src/reference/editorial/text/<slug>/{en,ja,page}.ssrg`
modules. The #704 dispatcher was extended using exact module, identity,
namespace and kind. Original compiler metadata, signatures and descriptions are
unchanged. In particular, concat receives its own explanation that it combines
Array<String> into String; the shared collection description is not globally
replaced. Untargeted `std/text::isEmpty` remains a regression control.

## Sources and executable evidence

- `docs/spec/10-library-surface.md`, §10.7
- Compiler-owned reference JSON
- `runtime/ts/src/text.ts`, `text-search.ts`, `text-core.ts`
- Selected `runtime/ts/tests/unicode.test.ts` contracts

Fourteen standalone operation examples plus one Seseragi unit-comparison example
are compiled and executed. A separate TypeScript comparison is typechecked and
executed. The displayed source and Playground links use the existing canonical
helper; the TypeScript example intentionally has no Seseragi Playground link.

Examples cover ordinary and empty input, literal separators and replacements,
retained empty fields, non-overlapping matches, CRLF/interior/trailing line
handling, Unicode White_Space, retained U+FEFF, checked scalar indexing and
invalid non-clamped ranges. scalarAt and sliceScalars include complete Maybe and
Either consumers; the module repeats the verified consumers with local branch
and contained-value explanations.

The same string `Aé😀e\u{301}` is used to explain five scalars, ten UTF-8 bytes,
four graphemes and six UTF-16 code units. The TypeScript example uses
Array.from, TextEncoder and Intl.Segmenter to demonstrate the corresponding
units, rather than presenting `.length` as a language-wide limitation. Results
for this string do not claim universal cross-version Unicode conformance.

## Checks

- `bun test apps/site/tests/text-editorial.test.ts`
- Regression: `apps/site/tests/array-editorial.test.ts` and
  `apps/site/tests/explanations.test.ts`
- `bun test runtime/ts/tests/unicode.test.ts -t 'distinct units|literal search|BOM survives'`
- Biome for the helper, test and TypeScript comparison; `git diff --check`

Final text checks: four tests passed (481 assertions), including the stronger
module-source/consumer-output assertions and the trim/concat wording guards.
The Array, text and shared explanation regression run passed twelve tests
(14,321 assertions) after dispatcher extension; the final text run also covers
the later explicit empty-input cases and review fixes. The selected Unicode
runtime suite passed three tests (73 assertions), with six unrelated conformance
tests filtered out. Biome and whitespace checks passed.

The final foreground prose report covered sixteen Japanese routes (fifteen
changed plus the control): 34 sentence-ending warnings, eight emoji notices,
four technical-alternative notices and one API-index list-ratio notice. The emoji
is the literal supplementary-plane sample used to explain UTF-16/scalar/byte
counts, not decoration; removing it would remove the evidence. Unit distinctions,
literal-versus-regex behavior and parallel API lookup lists are intentional.

The focused renderer uses the actual production catalog, renderer, examples,
signatures and locale modules. Both locales are generated for fifteen changed
pages plus the untouched isEmpty control. Assertions check displayed source,
exact output, declarations, module/API links, unit anchors and identity guard
behavior. The module's complete consumers and both language-comparison sources
are checked against their canonical descriptors.

## Review limits

The focused fixture contains the selected text metadata, not all 63 library
modules or the full API index. It does not establish complete-site navigation,
full generation, browser layout, screenshots or actual first-time-reader
acceptance. Those remain separate gates. The #706 full-site generator work is
not replaced or rerun by these focused checks.

An independent simulated-reader review read all fifteen Japanese bodies and
their fifteen English counterparts, checking paired code/output and local links.
It found a trim example pointer made stale by the added empty-input case, and
an ambiguous Array.concat comparison. The text now identifies the U+FEFF input
directly and names Seseragi’s std/array module explicitly. A fresh HTML reread
confirmed both corrections and found no remaining blocking explanation issue
in this bounded review. This remains simulated text/HTML review, not actual
human or visual acceptance.

Rendered Japanese prose is checked in the foreground using the pinned yomiyasu
tech guidance. Its warnings are advisory; useful API lookup lists and accurate
technical distinctions are not removed merely to obtain a clean score.

No compiler/runtime changes, issue closure, commit, push, merge, release or
deployment are part of this work item.

## Checkpoint-5 exact-title repair (2026-10-01)

The extended integration audit identified 56 page-name anchors across this
already-authored batch: fourteen selector links per locale and fourteen leaf
footer links per locale. Each selector now links the exact function title,
with its existing action text in adjacent Words after a colon. Each footer
links the module's exact title `std/text`; the existing action and purpose are
preserved immediately afterward: “Choose another text operation. Find the
operation for the result you need.” / “ほかの文字列操作を選ぶ。必要な結果から操作を探せます。”
No operation explanation, canonical source, output, signature, route, metadata
or fragment-link wording was changed.

Production edits are limited to `src/reference/editorial/text/model.ssrg` and
`src/reference/editorial/text/module/page.ssrg`. The existing dedicated
`tests/text-editorial.test.ts` now scans each complete authored article up to
its explicit generated-declaration/API-index boundary. It requires all 56
page-name links to have a rendered destination title and match it exactly;
per-page counts protect against dropped coverage. Mutating the module selector
or any leaf footer back to an action/decorated label must fail in both locales.
Fragment links retain their section-descriptive text.

Final focused rerun:

```sh
source ../seseragi-env.sh
SESERAGI_BIN="$(cat /tmp/seseragi-first-run-699-path)/bin/seseragi" \
  TEXT_EDITORIAL_RENDER_DIR=target/site-text-title-review \
  bun test apps/site/tests/text-editorial.test.ts
```

Result: **4 tests passed / 600 assertions / 26.23 seconds**. This includes all
15 existing native snippets, the TS comparison, canonical panel contents,
original results and declarations, exact identity dispatch and the expanded
link guard. Scoped TypeScript checking with the site's check.ts flags, Biome,
individual Seseragi formatting and diff whitespace checks passed. An additional
read-only check confirms all 28 selector actions are byte-identical to their
original text after being moved outside the anchor.

Fresh production-rendered HTML is retained in `target/site-text-title-review/`
with the test's existing flattened filenames, for example
`_docs_library_text_.html` and `_ja_docs_library_text_function_concat_.html`.
There are 32 files: 30 affected bodies (the module and 14 functions, each EN/JA)
plus two unchanged isEmpty controls. The independent reader compared the prior
retained snapshot with this output and reread all 56 repaired links. Exact
anchor labels match destination H1s; all 28 selector purposes and 28 footer
purposes remain adjacent. URLs and panel arrays are unchanged, as are the two
isEmpty control bodies. No bounded repair blocker remained. The reported
selected 30-file digest is
`2e7657f9c368af0fac72b5d2e1b21c084855da8c94942709c790f93bafc628bd`.
This is independent agent review; the all-overlay integration audit remains a
separate gate.
Full route integration and browser checks remain separate; browser access is
still blocked, and this correction does not claim visual or human acceptance.
