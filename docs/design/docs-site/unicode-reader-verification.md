# Unicode and grapheme reader verification

Author evidence for the approved 14 existing Library page identities, based on
checkpoint `78c3ff955b8194e0efa9f1eb04b4ca12a28f9533`, on 2026-10-01.
This is bounded documentation/example verification, not acceptance of the
entire Text category or first-time-human-reader sign-off.

## Scope and ownership

All routes below have English and Japanese bodies:

- `/docs/library/text/grapheme/`
- Its `function/{length,clusters,at,slice,byteboundaries}/` pages
- `/docs/library/text/unicode/`
- Its `function/{normalize,isnormalized,fullcasefold,version}/` pages
- `/docs/library/text/function/{casefold,tolower,toupper}/`

This adds typed exact-identity overlays and two existing-module introductions.
It addresses eleven of the previously inventoried 23 generic callable deficits,
plus grapheme.length's thin existing summary. It does not count the modules as
missing routes. The previous 14 Text overlays and Text module introduction are
unchanged; they are rendered as controls where relevant.

Owned files:

- `apps/site/src/reference/editorial/unicode-reader/`: fourteen sets of paired
  `page/en/ja.ssrg` modules, their small typed model and exact-identity dispatcher
- `apps/site/examples/src/api-unicode/`: twelve standalone Seseragi programs and
  two TypeScript counterparts
- `apps/site/scripts/unicode-reader.ts`: exact IDs, paths and expected outputs
- `apps/site/tests/unicode-reader.test.ts`: native, WASM/runtime, TypeScript,
  source/URL identity, production rendering and dispatcher checks

The parent integrated the central editorial/build/check/example-import
registrations and owns locks and the next complete-site gate. No compiler,
runtime, normative specification or canonical reference metadata was changed.
The unposted work-item draft is
`/workspace/shared/seseragi-unicode-reader-work-item.md`.

## Reader-facing decisions

The grapheme module starts with a label containing A, an emoji with a skin-tone
modifier, and e with a combining accent. It explains the visible pieces before
naming scalars and bytes. The ordinary example reads **3 clusters, 5 scalars,
12 UTF-8 bytes**. Its TypeScript counterpart also shows **7 UTF-16 units**.

The operations explain `Maybe<String>` as Just/Nothing and a checked slice as
Right/Left locally. Ranges are end-exclusive and are not silently clamped.
The bad end position 4 on three clusters returns a reason with the original
bounds and length. Repairing it to 2 returns `A👍🏽`. These are typed result
values, not compiler rejection diagnostics.

Normalization, full case folding and display casing remain separate. NFC
combines e plus accent into é; NFD decomposes it; NFKC/NFKD can remove useful
compatibility distinctions. `Straße` and `STRASSE` both fold to `strasse`,
while lowercasing produces `straße`. The text does not promise reversibility,
locale collation, username validation, confusable detection or identical host
Unicode behavior. The alias `std/text.caseFold` links to fullCaseFold's owning
explanation instead of reproducing its tutorial.

Complete examples define every name and explain imports, calls, relevant
`fn`/`let`/`match` syntax and the output entry. The version example explicitly
passes `()` as Unit; it is not described as taking no argument. grapheme.at
defines scalar and Char locally rather than requiring the module article.

## Semantic authority and execution environment

Read against `docs/spec/10-library-surface.md` §§10.7 and the
`std/text/grapheme` / `std/text/unicode` sections, the standard registry in
`crates/seseragi-project/src/standard.rs`, the unchanged compiler reference
artifact, `runtime/unicode/manifest.json`, and the existing Unicode runtime /
conformance tests.

- Native CLI: published Seseragi 0.61.19, release commit `a8641b5a81a4`,
  x86_64-unknown-linux-gnu
- Native/WASM language Unicode data: **17.0.0**
- TypeScript execution: Bun **1.3.9**, reporting Unicode **15.1**, ICU **75.1**
- Target environment: Linux x64; neither another OS nor an actual browser
  session was executed for this batch

The actual difference between pinned and host Unicode versions is important:
the selected examples match where stated, but this does not establish identical
segmentation or conversion for every code point. All selected library
operations support process/browser and need no external service; the output
entry is still run by the corresponding test runtime.

## Commands and technical results

```sh
source /workspace/scratch/677b5e8ef737/seseragi-env.sh
CLI="$(cat /tmp/seseragi-first-run-699-path)/bin/seseragi"
SESERAGI_BIN="$CLI" \
  SESERAGI_UNICODE_READER_OUTPUT=target/site-unicode-reader-review \
  bun test apps/site/tests/unicode-reader.test.ts
```

Final complete result after the checkpoint-5 title repair: **18 tests passed /
941 assertions / 19.36 seconds**. This includes the Unit/scalar wording, wrapper
corrections, and exact page-name anchor regression checks. A
transient unrelated Regex module-catalog nominal-type mismatch held one
intermediate render; after its owner repaired that assembly issue, this full
focused rerun passed without changing Unicode behavior or excluding the
production catalog.

Coverage:

1. All twelve displayed `.ssrg` files pass native lint and exact stdout checks
   after being copied to isolated temporary directories. The expected final
   newline is tested explicitly.
2. The same source-seeded Playground bytes compile through the committed WASM
   adapter and execute through the existing browser-runtime harness, matching
   all twelve outputs including Nothing/Left/empty inputs. This is not visual
   browser testing.
3. Both displayed TypeScript files typecheck and run with exact documented
   output. Their simple inputs and differing full-fold/lowercase semantics are
   preserved rather than forcing false language equivalence.
4. The documented invalid-range repair executes successfully; the version
   output is checked against the canonical Unicode manifest.
5. Canonical source bytes, SHA-256, reconstructed highlighting and seeded URLs
   agree. TypeScript panels do not acquire Seseragi Playground links.
6. The production reference renderer emits 34 actual bodies: the 28 edited
   bodies plus unchanged Text-module, Text.concat and Unicode.isMark controls
   in both locales. Exact declaration/signature text, original function titles,
   locale identity, anchors, source panels and module links remain checked. All
   56 page-name links in the complete authored portions of the 28 edited bodies
   are matched against rendered destination headings, including inline links
   in both locales. Missing destination titles fail rather than being skipped;
   per-page counts and an intentionally decorated-anchor mutation protect the
   coverage. Section-fragment links and external actions are separately scoped.
7. The new dispatch requires exact module, identity, value namespace and
   function kind. Similar/unknown identities and wrong module/kind/namespace
   do not receive the new explanation.

Initial direct checks inside the examples package resolved the entire changing
package and encountered its stale lock and another writer's incomplete Regex
source. That invocation was stopped; all intended snippet tests use isolated
copies. Locks were not regenerated to hide that assembly state.

Scoped TypeScript/Biome and individual Seseragi format checks pass. No existing
Text module/overlay, spec, compiler, runtime or reference metadata diff was
introduced by this task.

## Reading and display evidence

Retained HTML is under `target/site-unicode-reader-review/`, following the real
routes above. The author read both complete localized bodies and their displayed
examples. An independent agent read all 28 bodies and requested the Unit
wording correction; the local scalar definition was also strengthened. The
shared example wrapper now names `let` and graphemes and avoids unrelated
backtick guidance on the version page. The final fresh version pair and changed paragraphs were also read by the
author. The independent reviewer then reread the complete version pair, the
local scalar/Char definitions and all 24 leaf wrapper instances: no remaining
body blocker. It also verified all 32 seeded links across the 28 edited bodies
against displayed source. That pre-checkpoint artifact digest was
`bc4d1bf6f03b7f97b162ac45615be1e95e8ed6349c8b2838c5b01151ffcfdd23`; it is
superseded by the title-link repair below, not claimed as the final artifact.
This is agent review, not first-time-human-reader acceptance.

Advisory Japanese prose checks are retained in
`target/unicode-reader-yomiyasu.json`. They exclude code blocks and preserve
inline-code notation. The first pass reported 26 repeated-ending warnings, ten emoji warnings and
six negative-contrast notes; the final second pass after reader corrections
reported 26, ten and five respectively. The emoji are the subject of
the Unicode explanation, and distinctions such as byte offsets versus cluster
positions are intentional technical contrasts. No language rule or example was
weakened to obtain a style score.

The focused input has three real modules but only selected symbol lists. It
proves the changed rendering and the stated controls, not the complete API
index, full navigation/route closure, link-click round trips, desktop/mobile
appearance, keyboard behavior or overflow. Browser access remains blocked.
Full-site generation and integration evidence must be recorded separately.

## Checkpoint-5 exact-title correction

Integration preflight found eight authored links per locale in the Unicode
module whose anchors included purpose text. The destination titles were
correctly enforced by the full-site gate; the original focused checks had not
covered this rule. On 2026-10-01 the eight anchors were changed to the exact
headings normalize, isNormalized, fullCaseFold, caseFold, toLower, toUpper,
version and std/text/grapheme. Their purpose text remains adjacent, separated
by a colon. The descriptive std/text#text-units section link is unchanged.

The only production file edited for this repair was
`src/reference/editorial/unicode-reader/unicode-module/page.ssrg`; the dedicated
`tests/unicode-reader.test.ts` now checks all 56 authored page-name links across
the 28 bodies, not only the 16 reported mismatches. The complete focused test
rerun passed 18/941 in 19.36 seconds and refreshed the retained HTML. The site
gate's scoped TypeScript flags, Biome, Seseragi formatting and diff whitespace
checks pass. No canonical source, output, metadata, registry or lock changed.

The initial test-only mutation targeted the first navigation anchor rather than
the authored article; it did not prove rejection. It was corrected to mutate
the authored anchor as well and both locale mutations now fail as intended.
This is recorded separately from the final passing run. The independent
reviewer reread both refreshed Unicode-module bodies: all 16 corrected labels
match destination H1s, adjacent EN/JA purposes remain specific, and URLs plus
code/output panels are unchanged. Only these two HTML files differ from the
earlier reviewed snapshot. The final selected 28-file artifact digest is
`1cf967a4023d7142ffdf722ab51c76c6d529be93423b15e5f2554dd5e5ca0bfd`.
The broader integration title audit and full generation remain parent-owned;
this bounded reread does not substitute for them.
