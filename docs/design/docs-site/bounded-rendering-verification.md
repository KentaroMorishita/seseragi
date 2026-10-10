# Bounded complete-site generation (#706)

## 2026-10-10 canonical-toolchain timeout investigation

The current candidate keeps fresh processes and raises the finite page cap from
32 to 64. It does not reduce the catalog, metadata, examples, routes, locales,
validation or full artifact comparison. The per-build deadline remains 420s.
Integration acceptance is pending complete two-build and browser PR CI results;
the historical passes below do not establish current release readiness.

Baseline: #771 head `0601ba3ccbbed07ee9be34db019320356590eb5c`, Bun 1.3.11
(`af24e281`), optimized CLI 0.61.23, Linux x86_64. Compiler Rust source matches
main's canonical compiler; the locally built binary records the Docs checkout
commit `0601ba3ccbbe`, rather than claiming to be a downloaded release archive.
The complete input has 965 examples, 63 modules and 1,812 symbols. Its exact
size/digest and measurements are in
[the probe report](../../reviews/issue-706/2026-10-10-probes.json).

Compile took 40.049s. Fresh baseline 32-page probes took 3.4–4.4s, repeating
the same full-input JSON parse and derived decode. CPU sampling identified that
decode as a substantial fixed cost. Diagnostic timing of pure expressions in
a separate compiled artifact also measured decode, catalog, render and encode;
its stdout matched the original exactly. Instrumented timing is diagnostic,
not a benchmark of the untouched entry.

Resolving each symbol's editorial entry once preserves matching, precedence,
fallback and output. It reduces redundant catalog work, but representative
wall times did not prove an end-to-end improvement from that change alone.
The 64-page candidate amortizes full-input decoding across 63 rather than 125
fresh render processes. Three full-input probes (including Japanese pages)
took 4.577/3.674/3.952s and peaked at 1,321,104/1,603,412/1,420,864 KiB.
All 192 selected page records, route order and HTML matched the original
32-page pairs exactly. Sampling at 50ms does not establish an instantaneous
maximum; full host-plus-descendant measurements remain required.

Opt-in `SESERAGI_SITE_PROFILE` writes phase JSONL outside published artifacts:
compile, input, serialization, plan, every batch, generation, validation,
links, publication, hashes and cleanup. Use an absolute path when subprocesses
change working directory. Linux `profile-process.py` samples the process tree,
enforces a deadline and 4 GiB ceiling, and cleans up its descendants. Its
child-capture, deadline and memory-stop smoke checks passed.

The new `Site verification` PR lane runs on relevant draft and stacked PRs,
checks the exact head with Bun 1.3.11 and optimized current-source CLI, and runs
complete `check:site`, including its browser stage. Logs, phase data, memory
report, toolchain/lock provenance and screenshots are retained as Actions
artifacts. Both complete `site-manifest.json` files (including every artifact
hash and route) are copied to the artifact before temporary outputs are removed,
through the absolute `SESERAGI_SITE_TEST_EVIDENCE` directory. A failed build
without a published manifest does not produce a claimed complete manifest.
Branch protection and Vercel production settings are unchanged.

Independent review of head `78a5f60512f5fcb77ca14e95231c9235be8c1d51`
found that the original monitor could miss detached, reparented descendants.
A safe reproduction allocated only 64 MiB with a 48 MiB sampled-tree limit:
the old monitor exited successfully and reported only 8,412 KiB while a detached
orphan remained alive. The corrected monitor seeds every sample with matching
PID/start-time identities and adopted children, then expands their descendants.
Cleanup pins each process with a pidfd, validates its start time before signaling,
and reaps adopted children with a finite cleanup budget. Five regression tests
cover normal children, deadline stop, detached-orphan memory stop, PID reuse,
descendant expansion and complete cleanup. The before/after reports are retained
in [the monitor regression evidence](../../reviews/issue-706/2026-10-10-monitor.json).
The original-head CI remains historical evidence; final-head CI is required.

The protocol oracle's four CLI steps took 80.825/34.025/48.695/22.572s on this
host. Its assertions completed but the aggregate 180s test deadline failed
at 187.792s. Its total budget now covers four existing 90s command caps plus
30s for transport/assertions; those caps and the full-build 420s deadline are
unchanged. This measured fixture-budget correction is separate from the
full-input batching improvement.

Reproduction (after locked dependencies and browser bootstrap):

```sh
mkdir -p target/site-verification
export SESERAGI_SITE_PROFILE="$PWD/target/site-verification/phases.jsonl"
export SESERAGI_SITE_TEST_EVIDENCE="$PWD/target/site-verification/manifests"
python3 apps/site/scripts/profile-process.py \
  --report target/site-verification/process-tree.json \
  --timeout 3300 --limit-mib 4096 -- bun run check:site
```

## 2026-10-02 release-profile regression check

The complete build regression now selects `NODE_ENV=production` explicitly,
and the browser regression requests `profile: "release"`. The 420-second
per-build budget was measured for release generation, but both test entry points
previously inherited the development default when no environment was set.
The public development-build default and all route assertions remain unchanged.

On macOS arm64 with Bun 1.3.9 and the canonical released CLI 0.61.19,
`bun test apps/site/tests/build.test.ts` passed: 1 test, 8,877 assertions.
Both complete 3,976-route builds passed validation and produced identical
manifests and file hashes. Generation took 255.320 and 279.804 seconds;
the entire test, including artifact assertions and cleanup, took 586.704 seconds.
It checked 662 bilingual Language page-name links and 1,411 authored Library
page-name links across 578 localized bodies.

A separate monitor sampled RSS every 250 ms for the test process and all
descendants found recursively from macOS `ps` parent IDs. The observed process-tree
peak was 2,917,488 KiB (about 2.78 GiB), below the configured 4 GiB stop limit.
The monitor also enforced a 1,200-second overall deadline. Neither limit fired.
Sampling does not establish an unsampled instantaneous maximum or guarantee a
fixed memory bound for future metadata or larger pages.

The first complete output was retained for browser checks through the test's
existing `SESERAGI_SITE_TEST_RETAIN_OUTPUT` option. Local evidence is in
`/tmp/seseragi-706-release-build-check.json` and
`/tmp/seseragi-706-release-build-check.log`; the retained site is
`/tmp/seseragi-706-complete-site`. Browser checks and simulated reader review
remain separate results. The full generation success does not mark them passed.

Status: the bounded protocol and two complete static builds were verified on a
fixed source snapshot on 2026-10-01. This removes the observed all-at-once
rendering blocker for that snapshot. It does not accept later prose revisions,
browser behavior, root-domain cutover, or first-time-reader understanding.

## Change and ownership

[Issue #706](https://github.com/KentaroMorishita/seseragi/issues/706) adds a
separate, versioned request envelope around unchanged `BuildInput`. Seseragi
returns the complete route plan and renders contiguous batches of 1–64 pages.
Every document still receives the full catalog and compiler metadata. A fresh
process releases each batch's HTML and typed JSON allocations before the next
batch starts.

The host checks the response schema, mode, offset, total, route safety,
uniqueness, exact requested order and complete inventory. It retains all
existing global links/fragments, duplicate HTML IDs, reference coverage,
Japanese placeholder checks, missing-example checks, publication and manifest
hashing. Each invocation creates a uniquely named transport directory, so concurrent
callers sharing a generator cannot overwrite one another's input or output.
An outer cleanup removes only that invocation's directory on success or
failure; preexisting files beside the generator are untouched. A failed build
cannot publish its partial staging directory.

Implementation:

- `apps/site/src/model/render-request.ssrg`: transport types
- `apps/site/src/render/batch.ssrg`: selection with the full catalog
- `apps/site/src/main.ssrg`: typed request decoding and execution
- `apps/site/scripts/render-generator.ts`: strict protocol and process loop
- `apps/site/scripts/build.ts`: existing input/compile helpers and publication
- `apps/site/tests/render-generator.test.ts`: protocol and equivalence tests

No compiler, runtime JSON implementation, `BuildInput`, page renderer, catalog
ordering, browser permissions or deployment configuration was changed for this
work. The generator's internal stdin protocol changed together with its host;
direct `renderDocument` tests keep the same input contract.

## Baseline and snapshot

The pre-change diagnostic observed full-metadata rendering exceed 4 GiB RSS
before emitting output, in both normal Bun and `--smol`. Its measurement
harness intentionally stopped those runs at that ceiling. This does not prove
that an earlier external SIGKILL was an OOM kill. Redirecting stdout was not a
fix: stdout was already directed to a file descriptor.

The verified snapshot was taken from the uncommitted Docs work based on
`90cd575c1dc42566bb622b8e9baf794f5c71cf3b`. Concurrent article writers prevented a
stable shared lock, so the measurement copied the complete site source into a
private workspace and generated its own lock. It used 187 canonical examples,
63 compiler Reference modules and 1,812 symbols/instances. No library metadata,
page or locale was removed. Unchanged shared dependencies were resolved from
the repository; source files under review were copied.

Snapshot workspace content digest:
`sha256:4dfa21b12a0f9cce8ecd7de9b1d9f84f334b7e3c44ae940b916387177d64bd8d`.
The saved source-file checksum inventory has SHA-256
`7e3df88899b1fa9f0b4cb7984b277c3e37ab91be36c976c34f37623c8f7bb7a7`.

The snapshot predates the final small #705 reading corrections and subsequent
#708 editorial additions. Their integration must run against the later source
and final lock; this report does not claim they were included.

Toolchain: development CLI 0.61.19 at compiler commit `ab42da841d76`, Linux x64,
Unicode 17.0.0, and Bun 1.3.9 (`cf6cdbbb`). The generator itself was built with
`--profile release`. No compiler source changed between #696 and this test.

## Measured results

| Work | Result | Elapsed | Observed memory |
| --- | --- | --- | --- |
| Full-metadata route plan | 3,976 unique routes | 1.628 s | Child peak 564,700 KiB, about 551 MiB |
| First 32-page batch, same full input | 32 pages, no missing-example sentinel | 1.578 s | Child peak 662,040 KiB, about 647 MiB |
| First complete default build | 3,976 pages, 3,981 listed files | 247.535 s | Sampled host peak 1,430,616 KiB, about 1.36 GiB |
| Second identical complete build | Same inventory and file hashes | 246.517 s | Sampled host peak 1,144,836 KiB, about 1.09 GiB |

The plan and batch probes read `/proc` for their direct child and also recorded
Linux `getrusage` peak RSS. Full-build host sampling succeeded, but descendant
PID enumeration returned no child processes even after checking every Bun
thread. Therefore **the maximum renderer-child RSS across all 125 batches was
not measured**. The intended process-tree ceiling only covered visible
processes and must not be presented as a proved whole-tree maximum.

A quick separate Bun `resourceUsage` probe also completed one 32-page batch.
It reported raw maxRSS 707,320. Installed type documentation describes bytes,
while observed raw Linux values resemble KiB; this report does not turn that
inconsistent unit into a new memory claim. No third full build was run merely
to obtain a nicer measurement.

The 32-page cap bounds output selection, not arbitrary future page size or
metadata growth. The complete builds succeeded in this environment; that is
not a universal fixed-memory guarantee.

## Complete output and determinism

The ordinary CLI build was run twice in the fixed complete snapshot:

```sh
NODE_ENV=production SESERAGI_BIN=/path/to/seseragi \
  bun apps/site/scripts/build.ts OUTPUT https://seseragi.example
```

Both builds passed the existing global validation before publishing their
local output directories. The 3,981 hashed files consist of 3,976 HTML pages
and five assets; the manifest is an additional file. Those listed artifacts
total 161,646,691 bytes.

The two manifests are byte-identical and every listed file hash matches.
Manifest SHA-256:
`f8c6ae1028661e2fe7eda85f34f214274b51d270cd46950df4959516b8fc1956`.

All current `build.test.ts` assertions were then applied to those already-built
artifacts with a temporary harness substituting only the two build calls and
output paths. No assertion was removed. This passed 6,138 assertions,
including 650 exact bilingual destination-title links and 300 section-local
previous/next links across 214 Language Reference locale pages. The committed
default regression still performs both complete builds itself.

The regression deadline is now 420 seconds per complete build and 900 seconds
for its two-build test. This is based on measured full-metadata decoding of
about 1.6 seconds per batch across 125 batches, plus compilation. The previous
180-second budget could not accommodate that work. Each render request also
has its own 90-second timeout; missing output or timeout causes failure.

## Focused regressions

Final command `bun test apps/site/tests/render-generator.test.ts` passed all
five tests and 217 assertions in 79.65 seconds after the transport isolation
review fix and import cleanup.

`render-generator.test.ts` verifies:

- Unique complete planning and exact ordered collection, including a short
  final batch
- Invalid schemas, modes, offsets/counts, unsafe routes, changed totals,
  duplicate/missing/extra/reordered pages, and malformed records
- Byte-identical HTML from direct rendering and batches of one or four pages
  for the same complete small catalog, including Unicode, quotes and newlines
- Full cross-page navigation even when its destination lies outside the batch
- Cleanup after success, malformed JSON, a failed child, or an incompatible
  response; preservation of preexisting input/output files; and two concurrent
  callers using the same generator entry without mixing their payloads

The production protocol uses the same `renderDocument` as the direct oracle.
The test never compares two reduced catalogs or silently removes library
metadata to pass a full-site check. After the complete snapshot runs, independent
code review requested isolated per-invocation transport directories. That
transport-location-only fix was applied and covered by cleanup/concurrency
regressions and the direct-render oracle; the full-site builds were not repeated
for it. Unneeded BuildInput/RenderResponse imports were removed from main;
RenderRequest remains necessary for its field access.

`reference-titles.test.ts` now recognizes title-first purpose links followed by
an English period plus whitespace or a Japanese full stop, preserving the
existing colon/em-dash forms. Regressions reject wrong titles and avoid
classifying arbitrary sentence links as title links. All six tests and 13
assertions passed. The complete HTML check retained its original thresholds.

TypeScript checking, Biome, scoped Seseragi formatting and `git diff --check`
passed. The source/metadata/coverage tests remain separate from rendered
browser checks.

## Retained evidence and remaining gates

Local raw evidence is under `../seseragi-render-diagnostics/bounded/` beside
this checkout: exact full inputs, source checksum inventory, private snapshot,
plan/batch measurements, both complete builds, measurements, and the assertion
reuse harness. These generated artifacts are not added to Git.

The full outputs make text, links, fragments and real catalog navigation
available for review. Browser access is independently blocked: no browser was
started or retried, and no desktop/mobile layout, overflow, interaction or
console-error gate is claimed. `bun run check:site` was not invoked because it
would automatically start that browser stage. No publication, deployment,
merge or issue closure was performed.

At the snapshot checkpoint, the final current-source build and lock refresh
remained with the integration owner. The next section records their later
completion. Actual first-time-reader acceptance remains separate.

## Current-source integration checkpoint — 2026-10-01

After the content batches through #710 froze, the integration check used the
actual worktree on `docs/697-reader-contract`, based on
`90cd575c1dc42566bb622b8e9baf794f5c71cf3b`. It did not substitute the earlier
snapshot. All 898 site source/input/test files in the before/after inventory
remained byte-identical throughout the test run. The SHA-256 of that sorted
path-to-file-hash inventory is
`269edf994bc84e4fe5c23a128a6a80e174e1eca6c84616cf20f5f5c2b8ae46af`.

The current Rust CLI passed `cargo build -p seseragi-cli`. Lock updates,
examples and site tests consistently used the independently verified published
Linux x64 CLI `0.61.19` (release commit `a8641b5a81a4`), with Bun 1.3.9. The
compiler/runtime/Cargo/toolchain source diff between its full release commit
and current HEAD was empty, as was the corresponding worktree diff. Download
provenance is in [first-run verification](first-run-verification.md). This is
published-CLI execution evidence plus a current-source native build, not a
claim that the test executions used the development binary.

The three package locks were explicitly updated with this compiler:

| Lock | SHA-256 |
| --- | --- |
| `apps/site/seseragi.lock` | `8b1dc206fb6cea5524684ffba89499b18f78d7ea47241e77e6598f83a5344aa5` |
| `apps/site/examples/seseragi.lock` | `f7805fc4f402bb67a10791c9c861c1f2a3eaffaeb89e2429bfc4cf2b5015d5ef` |
| `apps/site/examples/invalid/seseragi.lock` | `57b3af0de117d5a6b9a7ac61d6260f318f1b702ecbad2110379b4c4750098936` |

Passed gates:

- Biome over `apps/site` and the complete TSC argument list from `check.ts`.
- Both foreground Python prose-extractor tests, the 351-route / four-source
  content-map check, and `git diff --check`.
- `bun apps/site/scripts/check-examples.ts`, including required diagnostics
  from all 77 language-invalid source files and separate import/visibility
  rejected projects. Runtime-failure examples remained outside the
  compile-invalid aggregate and were checked by their own tests.
- `cargo test -p seseragi-conformance stdlib_surface::tests::canonical_`:
  both canonical metadata tests passed.
- Every one of the 16 non-browser test files listed in `check.ts`, run
  serially with the published CLI and `NODE_ENV=production`: 59 tests,
  23,833 assertions, zero failures. This includes native examples,
  Playground WASM execution, all bounded page renders, complete navigation
  and protocol validation, concurrent transport cleanup, and `build.test.ts`.

The complete build test generated all 3,976 routes and 3,981 files twice, in
213.813 and 214.148 seconds. It checked 650 bilingual page-name links and all
other existing complete-site assertions, then confirmed equal manifests.
The retained first manifest's SHA-256 is
`6565bb2d12cdf8aa7adfb0c428a3bc8680fcb0e215b5c2a6672c842050410a82`.
Its full input contained 246 canonical example records, 63 reference modules
and 1,812 reference items. No routes, metadata or navigation were trimmed.
The finite 420-second per-build and 900-second test budgets were retained.
`build.test.ts` now honors `SESERAGI_BIN`, retaining the development CLI as its
default. Its one stale optional-field prose assertion was replaced by the
current explicit sentence distinguishing a required `Maybe` field from an
optional field; semantic coverage was preserved.

During preparation, an aggregate-import update accidentally wrote through a
reused path variable into one syntax-invalid fixture. The exact original
fixture was restored, the imports were written to the explicit aggregate main
path, and all 77 paths and aliases were checked. The final native syntax,
repair, aggregate-diagnostic and full-site tests above ran after that repair.
The restored fixture still produces its intended `SES-P0101` diagnostic.

Raw command logs, source inventories, lock hashes, worktree diff and the
retained manifest are in the local integration evidence directory
`/workspace/shared/seseragi-integration-20261001/`. Generated HTML and runtime
artifacts were not added to Git.

Browser execution was explicitly excluded because the environment action is
blocked. Consequently this is a successful current-source non-browser
integration checkpoint, not a `check:site` pass, browser/layout acceptance,
first-time-reader approval, publication or deployment. The earlier memory
measurement caveats remain unchanged; the two new runs did not measure every
renderer child's peak memory.

## Second current-source checkpoint — #711–#714

The next integration run used the frozen worktree based on local commit
`0ee111373e56f0316a5fc26b9f893d2cef4c7a0b`, including the module, type-reuse,
data-operation and Array/List batches. The same verified published Linux CLI
0.61.19 and Bun 1.3.9 were used for execution; a fresh current-source
`cargo build -p seseragi-cli` passed independently. Compiler/runtime/Cargo
source comparisons remained empty.

One real integration error was found: the data/newtypes article labelled its
link to types/newtypes as `Newtypes`, although the destination is
`Newtype identity and representation`. The owner corrected that one English
label, checked all 36 labels in the six-page batch and rerendered it. No title
validation was weakened. Before retrying, a temporary title-only program used
the actual typed catalog and complete BuildInput to emit 3,976 unique localized
titles. All 404 purpose-labelled links in 132 current focused locale artifacts
matched those titles. The diagnostic emitted titles rather than HTML, retained
the full catalog and metadata, and did not change the production render protocol.

After the correction, all configured non-browser checks passed:

- Three explicit lock updates; full site Biome and the configured TSC list;
  two Python prose tests; the 351-route/four-source content map; diff checking.
- The complete example checker, with all 93 language-invalid files imported
  once and required to produce diagnostics. Their bytes were checked unchanged
  during aggregate reconciliation. Whole module-project rejection cases remain
  in their separate complete-project test, not this aggregate.
- Both canonical Reference/Prelude conformance tests.
- All **22 non-browser test files: 112 tests, 27,933 assertions**, zero failures.
  The entire test list was rerun after the link correction.

The final complete builds took 232.295 and 226.137 seconds. Both generated all
3,976 routes and 3,981 listed files, passed 650 exact bilingual page-name link
checks and the remaining full-site assertions, and produced equal manifests.
The 420-second per-build and 900-second test budgets remain in place.

A test-only opt-in, `SESERAGI_SITE_TEST_RETAIN_OUTPUT`, now accepts an explicit
fresh output path. Inside `build.test.ts`, it copies only the complete generated
HTML/assets tree after generation and the route-count check, verifies the
copied manifest bytes, and then continues every normal assertion. An existing
destination is rejected; copy options also prohibit overwriting. Default CI
behavior and the unconditional temporary-directory cleanup are unchanged.
The branch was exercised in this final run. A retained generation is not marked
accepted merely because copying succeeded: the later full test result decides
that. The earlier external-copy attempt raced cleanup after the failing title
assertion; its partial directory is explicitly separate and is not a preview.

The complete accepted tree is retained locally at
`/workspace/shared/seseragi-integration-checkpoint2-20261001/full-site-attempt2/`.
All 3,981 listed file hashes were independently checked against its manifest;
all 3,976 route files exist, and the only additional file is the manifest itself.
Its manifest SHA-256 is
`965929b23aebba22ff00749ff16de8b60952dbeb8cae05870dcb37de371f04eb`.

All **1,089 site files** in the final run's before/after inventory remained
byte-identical while testing. The sorted path-to-file-hash inventory SHA-256 is
`53b120be1e478e4b096c187526c41412a7adfc06ce950bb4c179f62aade00777`.
The full input contained 386 canonical example records, 63 reference modules
and 1,812 reference items. Final lock hashes:

| Lock | SHA-256 |
| --- | --- |
| `apps/site/seseragi.lock` | `2b4aeffe73ba4af714bbbf13e076a97f7f954e80fb70674a185b724df9bf51f8` |
| `apps/site/examples/seseragi.lock` | `5f59e32e8bf4ecba99a6b22b3e44f9cd112a21ccc8dece5210ce4ad323001202` |
| `apps/site/examples/invalid/seseragi.lock` | `69ff8d1288f4e68bf92cdcfe856a6ed23ddf9eddc46071f4304333fbdc636ee9` |

Raw logs, both-attempt evidence, actual-catalog title data, source inventories,
status/diff snapshots and retained-file verification are under
`/workspace/shared/seseragi-integration-checkpoint2-20261001/`. Generated files
remain outside Git. Browser execution was excluded because the action is
blocked; this does not claim a full `check:site` pass, visual/browser acceptance,
first-time-reader approval, push, publication, deployment or issue closure.

## Third current-source checkpoint — remaining core articles

This run covers the remaining seven Types, ten Model, thirteen Traits and eleven
Effect-lifecycle articles, based on local commit
`882a6814d7c965da375ba56c1bef084e773820eb`. It uses the same verified published Linux
CLI 0.61.19 and Bun 1.3.9. A fresh `cargo build -p seseragi-cli` also passed.
Compiler/runtime/Cargo/toolchain sources remain identical to the release source;
no compiler or runtime change was part of this work.

The first complete generation succeeded in 260.318 seconds but the subsequent
Language title-link assertion failed. All three related links on eleven Effects
pages used explanatory questions as their link text instead of the destination
page's title. A complete retained-output audit collected all **66** EN/JA cases
before any retry. The owner restored the exact titles and kept each original
purpose beside its link. The first title-only preflight had covered 192 retained
focused bodies, but its directory discovery omitted the Effects artifact under
`/tmp`; it was not evidence that all core links had passed.

The title helper was strengthened for the corrected list-item form. It now checks
an exact title followed by an explicitly separated purpose in either a paragraph
or list item, including ASCII colon plus space. Anchor matching cannot consume a
closing anchor and spill into a later item. Eight tests/24 assertions preserve
standalone links, existing paragraph forms, mixed-list coverage, and the exclusions
for fragments, external destinations and ordinary sentence links. The original
full-gate coverage threshold remains unchanged. Before retry, the complete retained
Language output with the fresh 22 Effects bodies overlaid passed all **666**
matched references. Four of those point outside the Language tree; the final
Language-only test below checks 662.

After this single complete repair batch, every configured non-browser gate was
rerun against the frozen current source and passed:

- Fresh current-source cargo build and all three lock updates
- Site Biome: 87 files; the complete TSC argument list from `check.ts`
- Two Python prose tests and the 351-route/four-source content map
- Canonical examples: all 187 valid language files and all 130 invalid language
  files are imported exactly once in their proper aggregate mains; all required
  invalid diagnostics are checked. Reconciliation preserved every fixture byte,
  including the previously repaired source-text fixture. Complete module-project
  rejection cases remain in their separate project tests
- Both canonical Reference/Prelude conformance tests
- All **27 configured non-browser test files: 158 tests, 31,244 assertions**,
  zero failures

The final complete generations took **255.829 and 254.805 seconds**. Both produced
all **3,976 routes and 3,981 listed files**, passed all remaining global assertions,
and produced equal complete manifests. The final test checked **662** exact
bilingual Language page-name links. Neither generation was reduced to a subset;
the 420-second per-build and 900-second aggregate test budgets remain intact.

The accepted retained HTML/assets tree is:

```text
/workspace/shared/seseragi-integration-checkpoint3-20261001/full-site-attempt2/
```

Every listed file's SHA-256 was independently verified. All route files exist;
there are exactly 3,982 files including the manifest and no unlisted extra files.
Manifest SHA-256:

```text
9fa545add311657c4faa44d0f4ccf0c1f9a182e81cd815fce8baba1ad881751c
```

All **1,181 site files** in the successful run's inventory remained byte-identical
throughout testing. The sorted path-to-file-hash inventory SHA-256 is:

```text
4512c9d8c7cb4d7f90f3403180beac8842a6563ec3cd805a680c408026cb27cb
```

The complete input contains 466 example records, 63 Reference modules and 1,812
Reference items. The example corpus and metadata were not cut to lower memory.
Final lock hashes:

| Lock | SHA-256 |
| --- | --- |
| `apps/site/seseragi.lock` | `bfaa23507e3a945e06836a88ad9c0fe575e1802af2c2325ff6c7583709b03e4c` |
| `apps/site/examples/seseragi.lock` | `3de5ae13f000e4e334ddacb7d7815a911914226ea3b7929bcb0f6e147d35882d` |
| `apps/site/examples/invalid/seseragi.lock` | `644816d7a9cec470af8163c4c8dd4373897aa04f270d1997e83d1ed5e5d58003` |

Raw logs, before/after source inventories, the failed first attempt, complete
retained output, title audits and hash verification are under
`/workspace/shared/seseragi-integration-checkpoint3-20261001/`. The first retained
output is explicitly unaccepted; the successful second attempt is the artifact
identified above. Generated files are outside Git. Browser execution remains
excluded by the access decision. This is not a full `check:site` pass, visual
acceptance, first-time-human-reader sign-off, publication, deployment or issue
closure. Memory caveats from the earlier measurement remain unchanged.

## Fourth current-source checkpoint — Library directory and API corrections

The current source based on `6bd210c7903b9c98b1de9f924daa750c191640dc` includes the
Library landing/directory, twelve List pages, and nine exact API corrections.
The independent reader record covers 22 page identities and 44 EN/JA bodies.
All application sources and the reader ledger were frozen before final acceptance.
The compiler/runtime/Cargo/toolchain diff from official release commit
`a8641b5a81a4063054e4db4061aedaff86c55a7a` to the base is empty. The official Linux
CLI 0.61.19 executed the tests, and a separate current-source cargo build passed.

Every configured non-browser gate passed without an integration repair:

- Fresh native compiler build and all three lock updates
- Full site Biome over 101 files and the complete configured TypeScript argument
  list, including static checking of the browser test source
- Both Python prose tests, the 351-route/four-source content map, and diff checks
- Canonical examples and both canonical Reference/Prelude conformance tests
- Exact aggregate audit: all 187 valid language sources, all 130 invalid language
  sources, 13 new List fixtures and nine process-compatible correction fixtures
  are imported once with unique aliases. Storage's web-only integration module
  is excluded from the process aggregate and separately web-compiled/mock-tested.
  The original invalid source-text fixture retains its exact bytes
- All **30 configured non-browser test files: 171 tests, 33,384 assertions**,
  zero failures. The 29 non-build files account for 170 tests and 25,997 assertions

A bounded title-only run constructed the complete current catalog: 3,976 unique
localized titles. It checked 668 page-name references across 232 article bodies
with no mismatch. The Language sources were unchanged from the previously
accepted complete artifact; all 18 new correction bodies came from their fresh
successful render. Generated module-directory labels were not misclassified as
prose page-name references. The full build then independently checked all 662
bilingual Language page-name links with the existing coverage threshold.

The two complete builds took **264.493 and 271.254 seconds** and produced equal
complete manifests. Each includes **3,976 routes and 3,981 listed files**.
Batch size 32, the 420-second per-build limit, the 900-second aggregate limit,
and all global inventory/determinism checks remain unchanged.

The accepted complete HTML/assets tree is retained at:

```text
/workspace/shared/seseragi-integration-checkpoint4-20261001/full-site-attempt1/
```

All 3,981 listed hashes and every route file were independently verified. There
are exactly 3,982 files including the manifest, with no unlisted extra files.
Manifest SHA-256:

```text
43a21799cc2286584e21be1b4a807897c1378335f2474ba82138e75eaa90ab99
```

All **1,282 captured site source files** remained byte-identical throughout the
successful test run. Source path-to-hash inventory SHA-256:

```text
7494fceb064600599b6d494c9bec0b060b718a4fd7aa5f3ee225f5c7224d11bb
```

The retained full artifact contains all 18 corrected API bodies, 22 List API
bodies and both Library landing bodies byte-identically to their independently
reviewed focused renders. Both List module editorial introductions also match;
the subsequent generated public API directory naturally includes the full
module inventory rather than the focused test's subset.

The full build input has 497 canonical example records, 63 Reference modules
and 1,812 items. Input SHA-256 is
`abd5839b37669ed13735f6f0d74ce27d04c7fd7f95d035c2bae370b3fa231969`.
Final lock hashes:

| Lock | SHA-256 |
| --- | --- |
| `apps/site/seseragi.lock` | `e814dfde27d6b33f6fcba69f8fcbfdcec8608646eb7c5a4ca5e30d43630888ea` |
| `apps/site/examples/seseragi.lock` | `236d529011db37be2fb2b0f2702ff14831e1768f77f04dd35d2d6806da81647c` |
| `apps/site/examples/invalid/seseragi.lock` | `644816d7a9cec470af8163c4c8dd4373897aa04f270d1997e83d1ed5e5d58003` |

Commands, raw logs, unchanged-source inventories, title audit, artifact parity
checks and `summary.json` are under
`/workspace/shared/seseragi-integration-checkpoint4-20261001/`. Generated artifacts
remain outside Git. Browser execution is excluded by the existing access
decision; this is not a full `check:site` pass, visual browser acceptance,
publication, deployment or issue closure. No compiler/runtime change or new
memory-peak measurement is claimed.

## Fifth checkpoint — Map, Unicode, Regex and complete editorial title coverage

This checkpoint is based on `78c3ff955b8194e0efa9f1eb04b4ca12a28f9533`. The new
scope is 40 existing identities / 80 EN/JA bodies: Map fifteen pages,
Unicode/grapheme fourteen pages, and Regex eleven pages. Exact canonical
signatures and compiler metadata remain unchanged. The official Linux CLI
0.61.19 / `a8641b5a81a4` executes examples and rendering; a current-source cargo
build passes separately. Compiler/runtime/Cargo/toolchain sources still match
the release, with no changes in this docs batch.

### Preflight findings and repairs

The first title-only preflight found 44 current-batch mismatches before a full
build: 28 Map links used `Map` rather than the destination's `std/map` title,
and 16 Unicode selector anchors included their purpose text. Exact title labels
were restored with purposes kept adjacent and targets unchanged.

The audit then expanded to **every actual authored library overlay**, using the
typed `editorialFor` / `moduleEditorial` dispatchers rather than a guessed list.
It selects 100 identities: 93 API overlays and seven module introductions, or
200 localized bodies. This exposed 22 older List footer labels, plus 112 older
Array/Text labels: 22 Array footers, 34 Array selectors, 28 Text footers and
28 Text selectors. The repairs preserve action/purpose text in adjacent `Words`.
Generated public API directories with deliberate kind labels were excluded at
the established `using-this-module` boundary. Existing standalone descriptive
CTAs, section anchors and external links remain separate from page-name checks.

The clean expanded preflight checks **283 references across all 200 bodies**;
coverage did not drop when purposes moved out of the anchors. Fresh bounded
reading verified the repaired labels/purposes and unchanged code panels. Four
previously missing English Array bodies and the three earlier English entrance
bodies were also read. One Array sort sentence was corrected to say that the
second `println` call prints; no program or output changed. The final ledger is
frozen at SHA-256
`1b46f0854ad67d97ac0de4e361bec9175516d711ffc0c65eb12d7984d006bb0d`.

### Persistent regression coverage

The full build now retains this library check through test-only changes:

- `tests/library-titles.ts` executes the real typed dispatchers to derive the
  authored identities and checks their rendered bodies against all output H1s
- Module introductions stop before the generated metadata/API index; API bodies
  retain their complete article content
- `tests/library-titles.test.ts` covers exact titles, mutation rejection,
  generated-index exclusion, legitimate CTA/fragment/external-link exclusions,
  and missing article/boundary/title failures. With the existing title tests,
  this focused regression passes 13 tests / 37 assertions
- `tests/build.test.ts` requires at least 200 localized authored bodies and
  283 matched library references. Its previous Language checks, route count,
  complete second build, determinism and time limits are unchanged
- `tests/render-page-closure.ts` gains optional stdin for the bounded typed
  metadata query. Omitted stdin preserves the previous invocation behavior
- Both configured check lists include the new test

The actual helper was exercised before full generation: 100 identities,
200 bodies, 283 references, no mismatch. No production renderer or protocol
behavior changed for this regression.

### Exact staged validation

All static/native preparation passed after the content repairs: fresh cargo
build; three lock updates; site Biome over 136 files; the complete configured
TypeScript list; both Python prose tests; the 351-route/four-source content map;
canonical examples; both canonical Reference/Prelude conformance tests; and
diff whitespace. Biome's three informational template-literal suggestions in
parallel Map TS examples were left intact.

The aggregate audit confirms all 187 valid language sources and all 130 invalid
language sources remain imported once with unique aliases. It also confirms
37 new process imports: 14 Map, 12 Unicode and 11 Regex. Earlier 13 List and
nine correction imports remain present. The web-only Storage integration module
stays outside the process aggregate and has separate web-compile/mock tests.
The original invalid source-text fixture retains its exact known bytes/hash.

The sequential non-browser run exposed one **test-only stale control**: the
older nine-API test treated `std/map::containsKey` as untouched, but that function
is now intentionally authored. It was replaced by the still-unmodified
`std/map::filter`, preserving the same no-overlay and exact declaration/summary
assertions. The affected complete file was rerun successfully; full configured
TypeScript checking was repeated. Hashes prove that the prior 29 passing test
files and every production input were unchanged. Their results were retained,
and the affected file plus the remaining four files were executed on the final
source. This is an auditable staged run, not a claim of an uninterrupted run
without any test edit. The failed control/log and its only-file hash delta are
preserved under `test-control-repair/`.

Final result: **34 configured non-browser test files, 204 tests, 36,685
`expect()` assertions, zero remaining failures**. The two full generations took
**265.165 and 263.495 seconds** and produced equal complete manifests. Each
contains **3,976 routes and 3,981 listed files**. The full-output checks verify
662 Language page-name links and all 283 authored Library references across
200 localized bodies. Batch size 32, 420 seconds per full generation and
900 seconds for the build test remain unchanged.

### Accepted artifact and final hashes

```text
/workspace/shared/seseragi-integration-checkpoint5-20261001/full-site-attempt2/
```

Every listed file hash and route file was independently verified. The tree has
exactly 3,982 files including its manifest, with no unlisted extras. Manifest
SHA-256:

```text
81fc55c07f5ee2295eca39d5b095abff3b35e50a1a8e545196d84d5163063a9d
```

The final 1,480-file site inventory is captured after the test-control correction
and remains unchanged through completion. Its SHA-256 is:

```text
0101d029b2fb3eb41189b34dbd344435fe28fdadfae62bcc4399daf0b0bc3ab5
```

Every production input was unchanged throughout the successful staged tests;
the sole mid-run test-file correction is recorded above. The retained full
artifact matches all 200 accepted/fresh editorial sections byte-for-byte:
186 complete API article bodies and 14 module introductions before their
intentionally broader generated directories. This includes all 80 newly read
bodies. It does not silently expand earlier explanation-reading scope merely
because an unchanged body was compared.

The complete input includes 561 canonical example records, 63 Reference modules
and 1,812 items. Input SHA-256 is
`f1aafb185a2d0dda43078ee13ae77ce96ed31c96bf2e5041a4af7b3008291d76`.
Final locks:

| Lock | SHA-256 |
| --- | --- |
| `apps/site/seseragi.lock` | `660d5963c73e85498f51fa1f316dfe6157dc886ca3e582e7907f91dded17cc7f` |
| `apps/site/examples/seseragi.lock` | `8ecf5158a52eb91b08264cb54c07b8c049fc8c4dc9dd593bf96893ffe3b34639` |
| `apps/site/examples/invalid/seseragi.lock` | `644816d7a9cec470af8163c4c8dd4373897aa04f270d1997e83d1ed5e5d58003` |

Full logs, failed preflight/control evidence, the typed inventory, all-overlay
audit, per-file hash verification, staged test-source hashes and `summary.json`
are in `/workspace/shared/seseragi-integration-checkpoint5-20261001/`. No full
build ran against the failed preflight. Generated artifacts remain outside Git.
Browser execution is excluded by the existing access decision; this is not a
full `check:site` pass, visual acceptance, publication, deployment or issue
closure. No new memory-peak measurement is claimed.

## Sixth checkpoint — Set, NonEmptyList/Iterator and Char/Text

This checkpoint is based on `afe906eee87dd0b588730a578554851f0e2404ac` on
`docs/697-reader-contract` and the independently reviewed, frozen Set thirteen,
NonEmptyList/Iterator eight, and Char/Text thirteen identity batches. These
are 34 existing identities and 68 newly authored English/Japanese bodies.
The typed production dispatchers now expose 134 authored Library identities:
123 API articles and 11 module introductions, or 268 localized bodies. The
reader ledger remains frozen at SHA-256
`1eaef9b439a964cc06cdcafb415537d6e9d7d55d4ead000d8a0a60271512c43c`.

### Toolchain, unchanged inputs and non-browser checks

The exact official CLI is `seseragi 0.61.19 (release, commit a8641b5a81a4, target x86_64-unknown-linux-gnu)` at
`/tmp/seseragi-first-run-699.oOVHG8/bin/seseragi`, SHA-256
`987b0834c1ac299d0e4922e33160a324fa1d85dde909867b15660bac6e9d53b7`. A fresh debug CLI Cargo build also passed;
the compiler/runtime/Cargo diff to the release revision is empty.
The existing process SSG protocol, 32-page batches, 420-second per-build
limit, 900-second test limit and normal temporary-output cleanup are unchanged.

The recorded static gates pass: fresh CLI build, three lock updates, full
configured TypeScript checking, Biome, the two prose checks, content-map
coverage (351 unique routes and four specification sources), comparison
checks, and two canonical metadata conformance tests. All 142 selected
Seseragi formatter checks pass. Biome reports three informational template
literal preferences in unchanged Map counterparts. The canonical valid
aggregate owns 187 language sources, 279 unique imports/aliases including
`std/effect`, and all 32 new runnable source modules. The invalid aggregate
owns 130 sources and 131 unique imports/aliases; every configured invalid
fixture check passes. The browser-only storage module retains its separate
web-compile/mock coverage outside the process aggregate.

The stale no-overlay controls were corrected before the frozen run:
Set `size` uses unchanged `filter`; NonEmptyList `fromList` uses its unchanged
opaque `NonEmptyList` type because all seven callables now have authored
copy; Unicode `isMark` uses unchanged Text `isEmpty`. These remain control
checks, not added reader-quality acceptance. No production source was edited
during recovery.

The initial run completed 36 configured files: 231 tests and 31,921
`expect()` calls. Its first build attempt has only an opening log and no
completion record or retained artifact. Its original execution session is
unavailable, so that attempt is recorded as incomplete/unconfirmed, neither
passing nor failing. Its logs and temporary files remain untouched.
All 7,141 frozen inputs and 1,658 site files were
hash-verified unchanged before retaining those successful results and running
only the unchanged complete two-build test in fresh attempt2 paths.

Final result: **37 configured non-browser test files, 232 tests,
39,822 `expect()` calls, zero failures in the accepted run**. This is an
auditable recovered run, not an uninterrupted-run claim. The two complete
builds took **269.051 and 265.419 seconds**,
and the existing test's full manifest equality assertion passed. The first
tree is retained. The second tree is normally cleaned; its evidence is the
second completed-build log and passing complete-manifest equality assertion,
not a retained second tree.

### Accepted artifact and independent verification

```text
/workspace/shared/seseragi-integration-checkpoint6-20261001/full-site-attempt2/
```

The artifact has exactly 3,976 routes and 3,981 manifest-listed files, or 3,982
files including the manifest. Every route file and every listed SHA-256 was
independently verified, with no unlisted extras. Manifest SHA-256:

```text
7e96613f34471f4827cbebbf49d8d8bbc64f22cbd95af4eaae57b497c3cebbdd
```

The built-in complete-output title guards and independent audit verify all
662 Language-to-Language page-name references and 413 authored Library
page-name references across 268 localized bodies. All 268 authored sections
are byte-identical to reviewed/prior-accepted HTML: 246 complete API articles
and 22 module introductions before the generated public API directory. This
includes all 68 new bodies. Review originals, retained review copies and all
three review manifests also match their recorded hashes. This parity does
not expand prior explanation-reading scope.

The independent audit's initial recovery run used all-site target titles for
the Language guard, counting 666 rather than the existing Language-only
contract's 662. Its failed assertion/log is preserved. The corrected audit
uses the same Language-only target map as `build.test.ts` and passes; all four
additional all-site references also pass title validation. Only the external
audit harness changed, with no repository source or test adjustment.

The 1,658-file site inventory is unchanged throughout all accepted tests;
its manifest SHA-256 is
`59b458ce86a29dce0b405f36bcd1267745ed07a07ef54f243a4bf40adfc5f0af`.
The broader 7,141-file frozen-input inventory also remains unchanged
through the completed test/audit run; its manifest SHA-256 is
`995a6e8fc77e3d21a922d0b6409afb0f7cb0e9ef1d3a4a4e63ce35579b329691`.
The only later change is this evidence appendix, recorded separately in the
final freeze evidence. BuildInput contains 625 canonical example records,
63 Reference modules and 1,812 items; its SHA-256 is
`c9a4ed22dd770fe298410e1d4deb7223391afbc144dad577d289c6edad039482`.

Final locks:

| Lock | SHA-256 |
| --- | --- |
| `apps/site/seseragi.lock` | `c23c8c3971a4b6fb81384a2fdb942dbbb94e16b2ad6353a4f3c76be728ef898a` |
| `apps/site/examples/seseragi.lock` | `89943910b8c48cceebabf3f6ef55a5a91a71da91f4d089a17d133e0fdf488faf` |
| `apps/site/examples/invalid/seseragi.lock` | `644816d7a9cec470af8163c4c8dd4373897aa04f270d1997e83d1ed5e5d58003` |

Full logs, both attempt records, source inventories, title-scope audit,
reviewed-artifact parity, per-file hash verification and `summary.json` are
retained in `/workspace/shared/seseragi-integration-checkpoint6-20261001/`.
Generated artifacts remain outside Git. Browser execution is excluded by
the existing access decision. This is not a full `check:site` pass, visual
acceptance, deployment, publication or issue closure. No repository-wide
compiler/runtime gate or new peak-memory measurement is claimed.

## Seventh checkpoint — missing values, validation and collection types

This checkpoint starts from `bd5d39ebaf164c32cf562624d971e531309b68c7` on
`docs/697-reader-contract`. It integrates the independently read fifteen
missing-value/failure/validation identities and nine collection type/constructor
identities: 24 existing identities, 48 new English/Japanese bodies. The exact
reviewed artifacts and manifests are retained separately from generated output.
The typed production dispatchers identify 158 authored Library identities:
144 API/type/constructor articles and 14 module introductions, or 316 bodies.
The prior 134 identities remain represented. The reader ledger is frozen at
SHA-256 `3a3d5bbebbf074150770635e8d82c5b2c3c597e5bd2f46dd559ab1fd31db4729`.

### Frozen inputs and scoped gates

The compiler is `seseragi 0.61.19 (release, commit a8641b5a81a4, target x86_64-unknown-linux-gnu)`, at
`/tmp/seseragi-first-run-699.oOVHG8/bin/seseragi`, SHA-256
`987b0834c1ac299d0e4922e33160a324fa1d85dde909867b15660bac6e9d53b7`. The fresh Cargo debug build also passes and
its actual version/hash is retained in provenance. Compiler/runtime/Cargo sources
remain byte-identical to the release revision. The process SSG protocol,
32-page batching, 420-second full-build deadline, 900-second build-test deadline
and normal temporary-tree cleanup are unchanged.

All eleven preparatory gates pass on the final source: fresh Cargo CLI build,
three lock refreshes, Biome, the configured TypeScript check, two prose tests,
content-map checks, diff whitespace, comparison checks and two canonical metadata
conformance tests. Biome checked 199 files with only three informational template
literal preferences in unchanged Map comparisons. The content map retains 351
unique routes and four named specification sources. All 107 new/changed Seseragi
files pass canonical formatter checking.

The valid aggregate retains every one of 187 Language sources and has 300 unique
imports/aliases including `std/effect`: all fifteen validation programs and six
collection-type programs are registered. The invalid Language aggregate retains
130 sources and 131 unique imports/aliases. Eight new standalone collection-type
rejection fixtures are registered and exercised separately. Browser-only storage
keeps its separate web-compile/mock coverage outside the process aggregate.
BuildInput has 675 canonical example records, 63 modules and
1,812 Reference items; its SHA-256 is
`2803902354b27281ab51b7201bc64cac5f9182e7f867afe02d23432a41e8ec5a`.

The NonEmptyList opaque type and Iterator opaque type are now intentionally
authored. Their earlier no-overlay test controls were replaced before this
checkpoint by unchanged `Hash<NonEmptyList<A>>` and `Iterable<Iterator<A>, A>`
instance pages. Fixture counts, real signatures/readings and absent-editorial
assertions are preserved. These controls do not accept instance prose as read.

### Preserved preflight failures and bounded repair

A supplementary formatter pass found only
`apps/site/src/reference/editorial/data-validation-catalog.ssrg` noncanonical.
With parent authorization, the canonical formatter changed that dispatcher
alone. Its bytes are identical after removing whitespace; no displayed native
example, TS example, prose, output or Playground seed changed. The pre-fix file,
source inventory and failed logs are preserved. All preparatory checks and locks
were rerun, all 107 format checks passed, and the final source inventory was
frozen before the first integration test. Real render tests and all-section
byte parity below verify that the correction did not alter authored output.

The first standalone typed-title preflight inherited the helper's default
fresh debug CLI and exceeded its existing 90-second limit. The failure log is
preserved. The retry explicitly selected the verified official release CLI and
passed without any source, timeout or infrastructure change. Regenerating
BuildInput with that CLI produced byte-identical input. That failed preflight
is not counted as passing. All accepted test commands explicitly use the
verified release CLI.

### Complete tests and deterministic artifact

All **39 configured non-browser test files, 248 tests and
42,953 `expect()` assertions pass**, with zero failures in the accepted
run. The configured files ran serially to bound memory, with the complete
build test last. Both full generations completed in **278.074
and 279.177 seconds**. The unchanged test's full manifest
comparison passes. The first generated tree is retained; the second temporary
tree was cleaned normally. Two completed-build log entries and the passing
complete-manifest equality assertion are the second tree's evidence, not a
claim that it remains retained.

Accepted artifact:

```text
/workspace/shared/seseragi-integration-checkpoint7-20261001/full-site-attempt1/
```

Every one of 3,976 route files and every SHA-256 for the 3,981 listed files was
independently verified. The retained tree contains exactly 3,982 files including
its manifest, with no unlisted extras. Manifest SHA-256:

```text
405a414d2c6c5425e187b1512f7d79b4625b9c3ea2bf30a8dcee144130d4e3be
```

The built-in full-output guards and independent audit pass all 662
Language-to-Language page-name references and all 541 authored Library references
across 316 localized bodies. The independent audit also checks the four existing
Language links to other site targets. All 316 authored sections are byte-identical
to reviewed or previously accepted HTML: 288 complete API/type/constructor
articles and 28 module introductions before their generated API directory.
All 48 new bodies are included. The two new reviewed-HTML manifests and their
retained HTML copies match their recorded digests. Byte parity does not expand
prior explanation-reading acceptance.

All 1,788 site files are unchanged through the accepted tests and audits;
the site inventory manifest SHA-256 is
`98028c57fa017b6b5f94d1f5e50c23e26997275dc4835654ed385ff3ca990ef5`.
The broader 7,278-file frozen-input inventory also remains unchanged
through that run; its manifest SHA-256 is
`639512ca51270197523c69aa330f2559befbf63bf5a56c66157d4156b7fc34f8`.
This evidence appendix is the only subsequent source-inventory change and is
recorded separately in the final freeze evidence.

Final locks:

| Lock | SHA-256 |
| --- | --- |
| `apps/site/seseragi.lock` | `1fadecd439d064cc9d3e8d07f2b450e397a0ef862667fc63b9e7d4e8daef0fb5` |
| `apps/site/examples/seseragi.lock` | `ee735730fed80cd4baa467973195938f54a62c9e395234f50f4f4db1c703168c` |
| `apps/site/examples/invalid/seseragi.lock` | `3febd48bf57cb50a1123b72bfe672c6de67f5cbbe4b6eb8ccb5d1a2b9a7e3843` |

Complete logs, source inventories, pre-fix and failed-preflight evidence, typed
inventory, title audit, reviewed-section parity, per-file hash verification and
`summary.json` are retained in
`/workspace/shared/seseragi-integration-checkpoint7-20261001/`. `sessions.json`
records exact long-running commands, logs and session IDs; persistent result
records, not process-list inference, establish completion. Generated artifacts
remain outside Git. Browser execution remains excluded by the existing access
decision. This checkpoint is not a full `check:site` pass, visual or real novice
acceptance, publication, deployment or issue closure. No repository-wide
compiler/runtime gate or new peak-memory measurement is claimed.

## Eighth checkpoint — numeric readers and result foundations

This checkpoint starts from `15bd8858becffa32d19f55f66d630593ac90c603` on
`docs/697-reader-contract`. It integrates thirteen independently read numeric
identities and fourteen result-foundation identities: 27 existing identities,
54 new English/Japanese bodies. The typed production dispatchers identify
185 authored Library identities: 169 API/type/constructor articles and sixteen
module introductions, or 370 localized bodies. The prior 158 identities remain
represented. The reader ledger is unchanged at SHA-256
`ba16eaa5980706408f99979fadc3955a4ee64f06598c3024073e6f7a5d1f0039`.

### Parent source changes and preservation boundaries

The shared editorial catalog now uses shallow named lazy fallbacks in the exact
eleven-dispatcher order, retaining first-Just selection, the function-only
collection fallback and the existing module-block tail. Parent structural proof
and measured formatter results are preserved under `dispatcher-profile/`.
The nested formatter check took 15.095 seconds; the canonical shallow check took
0.0105 seconds. These are formatter measurements, not a controlled full-build
speed comparison. The profile's original shallow source precedes three canonical
line wraps; its non-whitespace bytes match the current source. An initial external
preparation assertion incorrectly required raw-byte equality. That failed setup
and both snapshots are preserved; correcting the external comparison needed no
repository source edit. Only confirmed completed gates are counted below.

The result reading adapter changes only two exact site identities:
`std/either::swap / std/either / value / function` gets explicit original-to-swapped
branch mapping in EN/JA; `std/validation::Validation / std/validation / type /
opaque-type` gets Japanese guidance permitting its exported constructors and
match. Other reading fields, signatures and canonical compiler metadata remain
unchanged. Exact-key negative controls and baseline-change rejection tests protect
these boundaries. Former no-overlay controls in the prior data/validation suite
now use the unchanged Maybe, Either and Validation Show instances, retaining
fixture counts, canonical signatures/readings and absent-editorial assertions.
These controls do not expand reader acceptance to instance pages.

All 67 numeric and 93 result-foundation author-inventoried source records match
their final reviewed hashes, including the shared dispatcher snapshot. The full
prior/new authored-section parity below is required for the shared refactor.
The documented Float HalfUp defect remains a visible limitation, and its two
rounding pages remain outside acceptance. No arithmetic runtime fix is claimed.

### Toolchain, packaging and scoped gates

The exact compiler is `seseragi 0.61.19 (release, commit a8641b5a81a4, target x86_64-unknown-linux-gnu)` at
`/tmp/seseragi-first-run-699.oOVHG8/bin/seseragi`, SHA-256
`987b0834c1ac299d0e4922e33160a324fa1d85dde909867b15660bac6e9d53b7`. A fresh Cargo debug CLI build also passed;
its actual version/hash is retained in provenance. Compiler/runtime/Cargo sources
remain byte-identical to the release revision. Full generations retain 32-page
batches, 420-second build limits and the existing 900-second test limit; normal
temporary-output cleanup is unchanged.

The tracked Python cache `apps/site/scripts/__pycache__/check-prose.cpython-312.pyc`
is removed, with only `/apps/site/scripts/__pycache__/` added as the narrow root
ignore rule. Its absence and ignore match are recorded. Check processes inherit
`PYTHONDONTWRITEBYTECODE=1`; the final audit confirms no `.pyc` exists under
`apps/site`. Generated bytecode is not reintroduced into the accepted source.

All eleven preparatory gates pass: fresh CLI Cargo build, three lock refreshes,
Biome, full configured docs/site TypeScript checking, two prose tests, content-map
coverage, diff whitespace, comparison checks and two canonical metadata conformance
tests. Biome checked 231 files with three informational template-literal preferences
in unchanged Map comparisons. All 130 new/changed Seseragi files pass canonical
format checking. The content map retains 351 routes and four specification sources.

The valid aggregate retains all 187 Language sources and has 327 unique imports /
aliases including `std/effect`, registering all thirteen numeric programs and
fourteen result-foundation programs. The invalid Language aggregate retains 130
sources and 131 unique imports/aliases. Existing standalone collection rejection
fixtures and new bounded numeric/result probes are exercised in their respective
reader tests. Browser-only storage remains separately web-compiled/mock-tested.

The numeric author's initial expanded runtime TypeScript import check reported
existing diagnostics in `foreign.ts`, `http-client.ts` and `web-file.ts`. Their
exact baseline record and blob/hash parity are preserved; all three files remain
unchanged. The accepted scoped TypeScript check and dynamic runtime execution do
not constitute a full runtime TypeScript pass. No runtime source was changed to
hide or resolve those diagnostics.

### Complete tests and deterministic artifact

All **41 configured non-browser test files, 262 tests and
46,593 `expect()` assertions pass**, with zero failures in the accepted
run. The complete configured file list ran serially, with the two-build test last.
Both full generations completed in **274.289 and
279.730 seconds**. The unchanged full-manifest equality assertion
passes. The first complete tree is retained; the second temporary tree was normally
cleaned. Two logged completed generations and the passing complete-manifest
comparison are the second tree's evidence, not a retained second artifact.

Accepted artifact:

```text
/workspace/shared/seseragi-integration-checkpoint8-20261001/full-site-attempt1/
```

All 3,976 route files and all 3,981 listed file SHA-256 values were independently
verified. The artifact contains exactly 3,982 files including its manifest, with
no unlisted extras. Manifest SHA-256:

```text
d40d9582d87a84ac209a990cb5280e7e39476cb0536df9d7bb2ddf39595b1bbf
```

Built-in full-output guards and the independent audit pass all 662
Language-to-Language page-name references and all 691 authored Library references
across 370 localized bodies. The four existing Language references to other site
targets also pass. All 370 authored sections match reviewed or previously accepted
HTML byte-for-byte: 338 complete API/type/constructor articles and 32 module
introductions before generated API directories. This includes all 54 new bodies
and proves preservation of every prior authored section under the shallow dispatch.
Both new reviewed-HTML manifests and their retained copies match recorded hashes.
This parity does not expand explanation-reading or browser acceptance.

All 1,948 site files remain unchanged through accepted tests and audits;
the site inventory manifest SHA-256 is
`c9cc17aa94774bb3ca40ac38ba966a44f2d781808549f7c021cbd59da50371a7`.
The broader 7,442-file frozen-input inventory is also unchanged through
that run; its manifest SHA-256 is
`89efee88b961fee73c9af9bc2dc2e83828a3a263cdd5b3ac37e5d12438570a92`.
This evidence appendix is the only subsequent frozen-inventory change and is
recorded separately in final freeze evidence. BuildInput includes
729 canonical example records, 63 modules and 1,812 Reference items;
its SHA-256 is `f1112550210a108f5f6cd902c89689c076587508f0ef27de2d580135aa5016ba`.

Final locks:

| Lock | SHA-256 |
| --- | --- |
| `apps/site/seseragi.lock` | `0768dcdf3e83fd27bea9d78f87c2901996093ad5a73e1b79864bbbecaccf675e` |
| `apps/site/examples/seseragi.lock` | `64316816b4e7e27ad657d42dad993de7cd8d0a38f0e3e3af1c2d5d878cf59570` |
| `apps/site/examples/invalid/seseragi.lock` | `3febd48bf57cb50a1123b72bfe672c6de67f5cbbe4b6eb8ccb5d1a2b9a7e3843` |

Logs, preflight/profile evidence, exact source inventories, packaging audit,
reviewed-section parity, file hash verification, session commands/IDs and
`summary.json` are retained in
`/workspace/shared/seseragi-integration-checkpoint8-20261001/`. Generated trees
remain outside Git. Browser execution remains excluded by the existing access
decision. This is not a full `check:site` pass, visual or real novice acceptance,
public issue closure, ref publication or deployment. This integration worker made
no public writes, compiler/runtime changes, broader runtime typecheck claim or
new peak-memory measurement.
