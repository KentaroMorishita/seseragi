# Result foundations: fourteen existing identities

This batch follows checkpoint 7 (`15bd8858becffa32d19f55f66d630593ac90c603`).
It authors fourteen existing identities, twenty-eight EN/JA bodies, for ordinary
TypeScript developers who do not already know match, ADTs or FP terminology.
It preserves the fifteen prior data/validation pages and the StateT.get overlay.
Compiler/runtime code and canonical reference metadata are unchanged.

## Exact scope and ownership

| Identity | Owning module | Namespace / kind | Program |
| --- | --- | --- | --- |
| std/prelude::Maybe | std/prelude | type / type | maybe |
| std/prelude::Just | std/prelude | value / constructor | just |
| std/prelude::Nothing | std/prelude | value / constructor | nothing |
| std/prelude::Either | std/prelude | type / type | either |
| std/prelude::Right | std/prelude | value / constructor | right |
| std/prelude::Left | std/prelude | value / constructor | left |
| std/validation::Validation | std/validation | type / opaque-type | validation |
| std/validation::Valid | std/validation | value / constructor | valid |
| std/validation::Invalid | std/validation | value / constructor | invalid |
| std/maybe::sequence | std/maybe | value / function | maybe-sequence |
| std/either::mapRight | std/either | value / function | mapright |
| std/either::bimap | std/either | value / function | bimap |
| std/either::sequence | std/either | value / function | either-sequence |
| std/either::swap | std/either | value / function | swap |

All these routes already existed. Their prior summaries were operation/type
specific, not universal placeholders. This is explanation work, not absent-route
repair. Maybe/Either and their constructors remain Prelude-owned. Validation
keeps its opaque-type classification and route while local copy explains the
public Valid/Invalid constructors and matching alternatives.

The typed tree is `apps/site/src/reference/editorial/result-foundations`.
`catalog.ssrg` requires the full identity, actual owning module, namespace and kind.
`model.ssrg` takes namespace and kind explicitly rather than using a function-only
helper. It reuses the existing Block renderer and typed CorrectionCopy pairs.
Parent-owned shared registrations add the callable/type/constructor dispatcher,
canonical example registry and tests. No module introduction was rewritten.

## Examples and reader contract

`apps/site/scripts/result-foundation-readers.ts` registers fourteen complete
Seseragi files and fourteen direct TypeScript counterparts under
`apps/site/examples/{src,comparisons}/result-foundations`. The displayed native
source is exactly the source in its Playground seed. TS panels have no Seseragi
launch link. Exact outputs are compared and shown on every page.

The first tasks use missing nicknames, checked seat counts, accepted names and
ordered form errors. Local prose explains annotations, fn, arguments, the final
return-type arrow, expression results, match alternatives and bound names, imports,
and the execution/output wrapper. Generic declaration slots are explained after
concrete values. Type and constructor leaves have their own task and consumer;
they do not require visiting a Tour or a module first.

TypeScript examples use optional values, ordinary result objects with a locally
explained ok field, ordinary conditions and arrays. Empty strings and zero remain
present. The nonempty error tuple is explained, not presented as a limitation TS
cannot express. mapRight/bimap comparisons directly produce the same display;
copy explicitly distinguishes that choice from Seseragi's intermediate Either.
No imitation generic FP library pads the TS side. The swap example declares a
reversed consumer contract and acknowledges that normal branching is usually
simpler when no such contract exists. Numeric examples use small integers only.

Important rules remain explicit:

- Just, Right and Valid construct alternatives; they do not validate contents or
  delay evaluation. Right 0 and Valid "" construct successful-side data
- Nothing is a value with no payload, not a Nothing () call
- Left/Invalid are returned data, not exceptions, automatic logging or cancellation
- Invalid requires NonEmptyList, preserving every error in order; a bare String or
  ordinary List is rejected even if the List happens to contain elements
- sequence receives already-computed result values. It preserves success order,
  returns Nothing or the first input-order Left, and succeeds with an empty
  collection. It cannot prevent work performed while constructing its input
- mapRight preserves Left without calling its transformation; bimap calls only
  the selected body once and retains the branch. Returning Either from a mapRight
  callback creates nesting; flatMap is the dependent-check operation
- swap exchanges branch and type positions without changing the payload. It is not
  recovery and does not make an earlier rejection valid

## Approved exact swap reading correction

The generic declaration-reading paragraph previously asserted Left=failure and
Right=success for every Either result, contradicting the explicitly reversed swap
consumer. Parent approval authorized the new
`scripts/result-foundation-reading.ts` and a small `scripts/reference.ts` wrapper.
The helper guards **std/either::swap / std/either / value / function** together.
It replaces only that sentence in EN/JA with the original-to-swapped branch
mapping. The argument count/order, result type, type-parameter guidance and all
other signature-reading fields are unchanged. Canonical metadata is untouched.

Tests check both localized replacements, reconstruct the original reading by
reversing the one sentence replacement, reject wrong identity/module/namespace/
kind matches, and preserve an unchanged non-swap mapRight reading. The helper
fails explicitly if the expected generic baseline sentence later changes rather
than silently applying a stale correction. No global Either semantic rewrite was
made.

## Former negative controls

Three controls in `data-validation-readers.test.ts` became intentionally authored:
Maybe.sequence, Either.mapRight and Validation's type leaf. With parent approval,
they were replaced by the unchanged Show instance for Maybe, Either and Validation.
The three replacements preserve the fixture's five modules and sixteen leaves;
reduceUntil remains its fourth control. Tests assert instance namespace/kind,
exact canonical signatures/readings, rendered declarations/readings and absence
of authored copy. The new exact-dispatch test also asks both the dedicated and
shared dispatchers to return Nothing for all three instance controls. These
checks do not constitute independent reader acceptance of the instance pages.

## Execution and rendering checks

The final suite is `apps/site/tests/result-foundation-readers.test.ts`:

1. Fourteen native lint/run programs, fourteen strict-checked TS programs, exact
   stdout/stderr and source-seed equality
2. Fourteen exact Playground seeds compiled through committed WASM and executed
   through the existing browser-execution runtime harness
3. Fifteen native boundary/rejection programs in the batch-owned fixture directory
4. Runtime instrumentation for exact callback selection/counts
5. Exact swap-reading override and unchanged-control regressions
6. Exact Validation type JA reading override and unchanged-opaque-type controls
7. Fourteen positive dispatch keys, four negative-key variants each, thirteen
   preserved earlier callables/StateT.get controls, and three unreviewed instances
8. All twenty-eight real production bodies, exact localized scalar/paragraph copy,
   panels, outputs, source links, declarations/readings, actual H1 titles,
   purpose-labelled links and same-identity locale targets

The real-renderer title guard reads complete authored bodies. Metadata inventory
remains 63 modules / 1,812 leaves. Related destination closures include the already
accepted data/validation and NonEmptyList explanations; this does not rereview all
those pages or broaden this batch's content acceptance.

The CLI is official Linux 0.61.19, release commit a8641b5a81a4, binary SHA-256
`987b0834c1ac299d0e4922e33160a324fa1d85dde909867b15660bac6e9d53b7`.
Bun is 1.3.9 and TypeScript 5.8.3. All owned .ssrg files were formatted before
rendering. No dependency install, Cargo build, WASM regeneration or complete-site
build was run by this worker.

The first complete run passed the first five tests, but both real catalog closures
reached the helper's existing 90-second bound during **lock update**, before run.
They used the verified release CLI, not the debug default. The failed log remains
`/tmp/seseragi-result-foundations-authored/test-initial.log`. Parent authorized a
local 180-second bound for these two closures, matching the numeric batch's scoped
checks. No shared helper timeout or guard was changed; the complete file is rerun
with those options. Test totals and independent reading are recorded below once
those runs finish. A failed attempt is not counted as proof.

Native fixture expectations include exit 70 for eager input/payload defects,
SES-T0101 for wrong constructor/contained-value arguments and Nothing (), and
SES-T0301 for a non-exhaustive Maybe match. Additional successful fixtures cover
unused defective callbacks, mapRight nesting, swap twice, List shape and first
Left. Planning-only earlier results are not counted as execution of the final
article sources.

## Remaining evidence and scope

The browser-execution harness is not a browser UI review. Desktop/mobile overflow,
clicked locale/related-link/previous-next behavior, and actual first-time-human
feedback remain unverified. Advisory Japanese lint, author checks and independent
agent reading are separate from those evidence categories. Parent owns the full
site integration checkpoint, shared locks and any commit/publication decision.
No issue/comment, push, PR, deployment or browser bypass was performed.

The twenty-three supporting Maybe/Either/Validation instance leaves remain
unreviewed. Transformer modules/structs/callables remain deferred, with StateT.get
preserved. The old canonical flatMap summary is separate upstream debt, covered
on the site by its prior exact overlay. This fourteen-page batch does not close
those remaining explanations or imply general language superiority.

## Bounded dispatcher diagnosis

The 180-second retry also stopped in lock update for both closures; it is preserved
in `test-pass2.log` and is not accepted evidence. No further bound increase was
made. The parent isolated a deep shared-dispatch match-chain parsing cost and
replaced it with shallow named fallback helpers while preserving the exact lazy
first-Just order, function-only collection fallback and module blocks. Parent
structural/profile evidence is under `/tmp/seseragi-dispatch-profile/`.

An isolated 2,306-byte stdin check measured the dedicated fourteen-page dispatcher:
lock update 5.463 seconds, execution 2.967 seconds, output `[true]`. The shared
snapshot already contained the parent's shallow repair (matching file hash), so
its lock 15.571 seconds/run 8.290 seconds measures the repaired path rather than
being a controlled comparison to the former nested source. Both outcomes and
phase timings are recorded in `diagnostic-dedicated.json` and
`diagnostic-shared.json`. The full focused file was then rerun without dropping
guards or changing the shared render helper. Earlier failed logs remain intact.


## Independent-reader revisions

The independent agent read all twenty-eight complete initial bodies and reported
five paired-page corrections, now consolidated in one revision:

- Maybe, Nothing, Validation, Valid and Invalid did not use show, so their wrapper
  paragraphs no longer explain that unused function. A new test compares wrapper
  prose with actual canonical source usage
- Nothing's Japanese wording no longer suggests that the bound Nothing later
  changes into a string
- Valid and Invalid identify valid, invalid and invalidMany as functions from
  std/validation rather than using an unintroduced validation alias
- Validation's Japanese generated opaque-type reading now explicitly permits its
  exported Valid/Invalid constructors and match, consistent with the page and the
  unchanged English guidance

The parent explicitly approved the last adapter change, extending the existing
resultFoundationReading helper only for **std/validation::Validation /
std/validation / type / opaque-type**. Only the Japanese representation sentence
changes; English and the type-parameter tail remain byte-for-byte unchanged.
Positive tests, four wrong-key controls, an unchanged Map opaque-type reading and
a changed-baseline rejection protect this boundary. No other opaque-type reading
or canonical signature was rewritten.

The reader found no blocker in the other eighteen initial bodies. Swap's complete
rendered declaration was independently confirmed to describe the reversed branch
mapping without calling it recovery. Final changed bodies are sent for complete
rereading after the full focused and affected prior suites finish. The canonical
native/TS programs are unchanged by these prose revisions.

The first corrected-body run passed the seven execution/identity tests but stopped
on an over-broad new prose assertion: rejecting `validation.` also matched the
legitimate module name `std/validation.` at the end of a sentence. The test now
rejects only the unintroduced qualified calls `validation.valid`,
`validation.invalid` and `validation.invalidMany`. Article prose and executable
sources did not change for that assertion repair. The failed attempt remains in
`test-revision2.log`; the complete corrected suite is rerun as `test-final.log`.
The final-adapter affected prior suite separately passed 6 tests / 1,555 assertions
in 56.58 seconds (`prior-suite-revision2.log`).

## Final frozen result — 2026-10-01 UTC

The complete final focused suite passed **8 tests / 1,652 assertions** in 58.95
seconds (`test-final.log`). The exact-dispatch closure took 25.582 seconds and the
complete twenty-eight-body closure 28.643 seconds; these are whole-closure times,
not separate lock/run measurements. The separate diagnostic phase timings above
remain explicitly labelled as the small-input diagnostic. The final-adapter prior
suite passed **6 tests / 1,555 assertions** in 56.58 seconds. No full-site pass is
claimed by this worker.

All fourteen paired programs have 36 matching stdout lines. All fourteen exact
committed-WASM seeds pass. Final source records and seeds are in
`canonical-source-records.json`; passing tests bind them to the displayed sources.
All 73 owned .ssrg files pass format checks; Biome checks 19 relevant TS files
without fixes; the scoped configured TypeScript check and git diff whitespace
check pass. The 47 pre-existing typed data/validation files remain byte-identical
to the planning preservation inventory.

Two foreground advisory yomiyasu passes inspected the entire rendered Japanese
bodies, excluding source/output blocks and marking inline syntax. The second pass
retains 45 repeated-ending warnings and four technical negative-comparison notes,
with scores 78–90. The contrasts explain already-computed values, grouped errors,
NonEmptyList versus a bare reason, and selected-only bimap callbacks. These are
real semantic distinctions. Active polite endings were not distorted to optimize
a style score. No further stylistic pass, lint process or browser process remains
from this work. Japanese-first source and the pinned tech guidance were used;
style scores are not reader acceptance.

Final artifacts under `/tmp/seseragi-result-foundations-authored/`:

- `rendered-revision2/`: final real production tree, seventy files including
  supporting older pages and controls; exactly twenty-eight bodies belong here
- `body-manifest-final.json`: selected full HTML hashes, SHA-256
  `96cd106eb95877fe23269466cd6a61cf48bacfc79e4775ac34e0e338cf425854`
- `source-manifest-final.json`: 93 owned/relevant source records, including the
  parent-owned shared catalog snapshot, SHA-256
  `62707a5e92e3e6b096d61b3683fde3ca4850e401bbbcc2a431ca319a27e6034a`
- `revision2-diff.json`: exactly ten corrected bodies, eighteen byte-identical
- `ja-prose-revision2/`, `yomiyasu-pass2.json`: final extracted prose and advice
- `test-final.log`, `prior-suite-revision2.log`: final complete passing runs

The independent reader completely reread all ten corrected bodies, separately
verified the other eighteen were byte-identical, and cleared the bounded agent
reading. It independently checked 76 purpose-labelled authored H1-title links,
28 native panels/source seeds, 28 TS panels, 56 outputs, 56 locale links and
28 canonical declarations, with zero errors. Its report and manifest are:

- `/tmp/result-foundation-independent-reader-review.md`, SHA-256
  `ff967e1a9692a57de351702d2ebae2a449b52b35a53af87266a2b0044ac7c3e2`
- `/tmp/result-foundation-independent-reviewed28-final.sha256`, aggregate SHA-256
  `30dc290299152b0b58b5d34a409143aa9c692cff57d27af048fd9b5df9ed1c6d`

The independent manifest sorts artifact-relative paths and writes each HTML-byte
SHA256, two ASCII spaces, the path and LF; its aggregate is the hash of manifest
bytes. This convention is different from the author's JSON manifest.

Per-identity evidence is recorded below. “Code” includes native lint/run, strict
TS parity and the exact committed-WASM seed. “Render” includes both locales and
all source/copy/metadata/title assertions. Browser UI and real first-time-human
acceptance remain unverified for every row.

| Identity | Code | Render EN/JA | Independent agent reading |
| --- | --- | --- | --- |
| std/prelude::Maybe | Pass | Pass | Corrected pair reread; clear |
| std/prelude::Just | Pass | Pass | Complete pair read; clear |
| std/prelude::Nothing | Pass | Pass | Corrected pair reread; clear |
| std/prelude::Either | Pass | Pass | Complete pair read; clear |
| std/prelude::Right | Pass | Pass | Complete pair read; clear |
| std/prelude::Left | Pass | Pass | Complete pair read; clear |
| std/validation::Validation | Pass | Pass | Corrected pair reread; clear |
| std/validation::Valid | Pass | Pass | Corrected pair reread; clear |
| std/validation::Invalid | Pass | Pass | Corrected pair reread; clear |
| std/maybe::sequence | Pass | Pass | Complete pair read; clear |
| std/either::mapRight | Pass | Pass | Complete pair read; clear |
| std/either::bimap | Pass | Pass | Complete pair read; clear |
| std/either::sequence | Pass | Pass | Complete pair read; clear |
| std/either::swap | Pass | Pass | Complete pair/declaration read; clear |

This does not accept the forty-two supporting/older/control bodies in the fixture,
any of the twenty-three supporting instance leaves, transformers, browser layout,
actual novice feedback or a complete-site checkpoint. Those boundaries remain
with their respective work and the parent integration task.
