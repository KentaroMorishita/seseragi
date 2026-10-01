# Array editorial pilot verification (#704)

## Scope and ownership

This changes twelve existing page identities: `/docs/library/array/` and its
`function/` pages for `chunksOf`, `dropWhile`, `findIndex`, `groupBy`, `init`,
`last`, `reduceRight`, `sort`, `sortBy`, `takeWhile`, and `windows`. Function
route slugs are lowercase. Each identity has the same Japanese route under
`/ja/`.

Prose lives in typed `src/reference/editorial/array/<slug>/{en,ja,page}.ssrg`
modules, with a separate module introduction. The dispatcher matches the exact
compiler identity, module, namespace and item kind. Original compiler signatures,
type parameters, constraints, descriptions and provenance are unchanged. The
existing semantic Block renderer remains the only renderer. Untargeted items
retain their existing summary and metadata presentation.

The host helper `scripts/array-editorial.ts` owns only example identifiers,
source paths and expected execution output; it does not own page prose.

## Semantic evidence

- `docs/spec/10-library-surface.md`, Array/List operation contract
- `runtime/ts/src/array.ts`, actual operation implementations
- `runtime/ts/src/sequence.ts`, stable ordering and once-only sort key evaluation
- `runtime/ts/tests/sequence.test.ts`, callback order, short circuiting,
  size-error payloads, grouping order, stable sorting and non-retention tests
- Compiler-owned reference JSON at
  `examples/spec/artifacts/stdlib-schema-1/reference/module.json`

Thirteen standalone executable examples (eleven operation samples plus two complete result-consumer samples) are displayed verbatim and linked to the
Playground through the existing canonical source helper. They cover successful,
empty and invalid-size results as applicable. `findIndex` handles `Just` and
`Nothing` explicitly. The module also demonstrates complete Maybe and Either
consumers, and the affected operation pages link directly to those explained
branches. Pages using Maybe or Either locally explain the named
alternatives before referring readers to deeper material. `sortBy` demonstrates
stable ties; `reduceRight` spells out its element-first step order.

A formatter check exposed spacing changes around an explicit record-type argument
in the first `sortBy` snippet. The final example uses an ordinary annotated empty
record array instead. No compiler, formatter, specification or runtime change was
made to hide this discrepancy.

## Verification environment

Working base: `90cd575c1dc42566bb622b8e9baf794f5c71cf3b`, with the concurrent
Docs worktree changes. CLI: development `0.61.19`, commit `ab42da841d76`,
`x86_64-unknown-linux-gnu`. These are local checks, not a published release claim.

Commands (after loading the environment's Bun path):

- `bun test apps/site/tests/array-editorial.test.ts apps/site/tests/explanations.test.ts`
- `bun test runtime/ts/tests/sequence.test.ts`
- `biome check apps/site/scripts/array-editorial.ts apps/site/tests/array-editorial.test.ts`
- `git diff --check`

Latest local result: 8/8 tests passed across Array and existing explanation tests
(13,854 assertions); the runtime sequence suite passed 16/16 tests (317 assertions).
Biome and whitespace checks passed. The final foreground prose report covered
13 Japanese routes (12 changed plus the control), with 17 sentence-ending warnings,
three technical-alternative notices and one module-index list-ratio notice. These
remain advisory, not reader acceptance. No lint process was left running.

The Array tests execute each source outside the shared example package, assert
exact output, and decode the Playground source link back to that source. They
also verify the overlay identity inventory, missing/duplicate metadata targets,
and rejection of the wrong namespace, item kind, module and symbol.

## Rendering and review limits

Focused production rendering uses `tests/render-page-closure.ts` and the actual
`renderDocument`, page/catalog code, canonical examples and signatures. It renders
both languages for the twelve changed pages plus one untouched `Array.length`
control. Assertions check exact displayed source, output, declarations, removal
of the targeted generic placeholder, module-to-API links, and preservation of
the control's exact prior summary.

The first independent simulated-reader review found three issues: wrapped results
needed complete consumer syntax, the shared empty-array explanation did not match
the annotated record array in sortBy, and singular type-parameter grammar was
incorrect. The implementation adds the two executable consumer examples and
labelled anchors, specializes sortBy setup copy, and corrects singular/plural
English in the shared signature reader. An independent simulated-reader reread of the fresh EN/JA HTML on 2026-10-01
confirmed the complete consumers, all five direct links and their target anchors,
the sortBy annotation, reduceRight wording and singular English correction. No
blocking explanation issue remained in this bounded text/HTML review. This is not
a browser/layout result or actual first-time-reader sign-off.

The focused fixture includes the selected Array entries, not the entire standard
library. It therefore does not prove the full 63-module navigation, complete API
catalog, global previous/next behavior or full-site generation. No browser,
desktop/mobile screenshot or visual layout pass is claimed here. Those remain
separate integration gates. Rendered-text self-review and an independent simulated
reader review are not actual first-time-reader acceptance.

Japanese prose is extracted from rendered HTML with `check-prose.py`, excluding
code blocks, and checked in the foreground using the pinned yomiyasu tech guidance.
Sentence-ending and genuine technical-alternative warnings are advisory. The
module's operation list and canonical API index are intentionally parallel lookup
lists, so the list-ratio warning is not addressed by removing useful navigation.

No route was marked reader-accepted; no issue was closed, and no commit, push,
release or deployment was performed for this work item.

## Checkpoint 5 exact-title repair

The extended library audit found 22 pilot-footer anchors using an action instead
of the destination title std/array, plus 34 module-selector anchors whose action
text was inside the page-name link. Footers now link std/array and retain the
compare/choose purpose beside it. Each selector links the exact function title
and keeps its existing action in adjacent text. All destinations, programs,
outputs, signatures and unrelated prose are unchanged.

The focused fixture now includes all seventeen Array overlays, the module and
an untouched length control (38 locale pages), allowing every selector target's
real H1 to be checked. The repair affects the module plus eleven original pilot
functions, 24 locale bodies. The full authored selector and leaf sections check
all 56 repaired links; the render passes 362 assertions. TypeScript, Biome,
Seseragi formatting and whitespace checks pass. Fresh artifact:
`/tmp/array-title-repair-rendered` (flat route-derived HTML filenames).

Independent rereading cleared all 34 selector purposes and 22 footer changes,
checked exact destination headings, and confirmed the 24 selected panel sets are
unchanged. Current 24-file link-review digest:
056b81a08ac71a6dc33edf2361e6c616e873ea537e876f8371ec82537b9305f6.
The original #704 twenty-file partial-English subset now has digest:
a0c02113db30a03ced22f3806c20875d3329b6e6fc3906544fc47bb1105346cf.
This is a bounded correction;
it preserves the earlier whole-body review scope, including its partial English
coverage, and does not establish additional browser or human acceptance.

## Final English coverage and output attribution

The reviewer subsequently read the four previously omitted English bodies in
full: dropWhile, takeWhile, findIndex and sort. This closes the earlier English
coverage gap for the twelve-identity pilot. One sort sentence attributed output
to show; it now says “The second println call prints”. The final production
refresh passes 362 assertions, and the targeted reread confirms that this exact
English sentence replacement is the only change in all 38 fixture files.
No body blocker remains in the completed bounded review. This does not establish
browser layout or real-human acceptance.

Current selected 24-file digest:
a3ac36f25f1d4e7a0d2dbde6e64be1706306563c75e074a38da2e41076198623.
The four newly completed English bodies have digest:
e7b0d0994d97a64e5853d880865b1e77f1d90a2dba1d1cb04161fd8b4929a36c.
These records supersede the earlier current-artifact and coverage limitations;
the original twenty-file subset digest remains unchanged.
