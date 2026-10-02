# Collection transformations14 verification

This record starts again after the 2026-10-02 cloud workspace replacement. The
previous local source/evidence directories were lost. No prior Transform14 PASS,
source digest, output receipt or reader acceptance was inherited. The restored
public Stdin baseline is commit 14c324998c6963c2de02280b09672a02d43e96c6, tree
ca065c6f253852a6bdfed62f1aeb02e7e398c1d5.

The reconstructed source candidate received an immutable WIP backup in Draft
PR 751 before long verification. That backup was not final acceptance. Retained
scripts/snippets and displayed text remain labelled recovery inputs; missing
family structure and editorial tests were newly authored.

## Fresh source and runtime checks

The actual configured site TypeScript command passes. Its exact argument list
was extracted from the current site check script, including the test harness.
Earlier scoped checking had omitted the harness's literal-inferred matcher types;
the reconstructed test includes erased generic widenings while retaining expected
values and assertions. Scoped Biome checks pass.

All 28 displayed source files were strictly checked as applicable and executed as
14 native/TypeScript pairs on both real Node and Bun. Each mode has 56 exact panel
process runs, four boundary runs, twelve intended malformed-source rejections,
24 nearby repair runs, two callback-contract probes and fourteen exact decoded
WASM/current-host seed runs. Each mode retains 193 bounded command receipts and
stdout/stderr bytes. Source/output manifests are freshly generated.

| Fresh mode | Tests | Assertions | Duration | RSS observation |
| --- | ---: | ---: | ---: | --- |
| Portable default | 7 | 1,762 | 18.79s | null; not measured |
| Opt-in Linux monitor | 7 | 1,569 | 25.77s | peak 724,791,296 bytes |

Portable execution uses a 60-second child timeout and 16 MiB output bound without
requiring Python or /proc. The optional monitor samples the child process tree
every 20 ms against a 1 GiB RSS limit. Sampling is not an address-space reservation
guarantee. Different assertion totals reflect the null-RSS branch. These runs
used the current Linux host; no Mac/Windows run is claimed.

Fresh toolchain receipts pin actual executables and hashes: CLI 0.61.19 at the
restored official release path, Node 24.19.0 at the actual Node runtime path, and
Bun 1.3.9 at the restored bun-linux-x64 path. The CLI SHA-256 is
987b0834c1ac299d0e4922e33160a324fa1d85dde909867b15660bac6e9d53b7.

WASM checks compile the exact decoded native seeds and execute the current
browser-host implementation under Bun. Raw stdout checking preserves final
newlines. This is host-execution evidence, not browser UI, public deployment or
visual validation. Generated-TypeScript digest fields remain receipt-only when
the generated text is not retained; source, compiler/runtime and output bytes
are separately bound. No old digest is substituted for a fresh execution.

## Fresh rendering and full reads

An isolated render of the restored published baseline produced 184 complete
Array/List pages. The 156 unselected complete article hashes were generated from
that render and written into the new preservation fixture. No lost control hash
file was copied.

The initial new editorial gate passed two tests and failed one: both filterMap
pages linked to nonexistent function/map routes. All 28 initial complete articles
were independently read, Japanese first then English within each family. Twenty-
four were accepted; four localized filterMap bodies required the link correction.

The correction changes the link destination, canonical title and task-purpose
sentence to concat. The failed run and pre-correction bodies remain in the new
evidence. Exactly those four bodies changed; the other 24 did not. Displayed
programs and outputs did not change.

The corrected full editorial suite passes three tests and 2,465 assertions in
83.63s. It checks exact tuple guards, 28 bilingual bodies, source/output panels,
decoded seeds, canonical readings/declarations, curated links and 156 unchanged
prior complete articles. All 28 final bodies were fully reread independently,
including the 24 unchanged bodies. Every final body is accepted, with no remaining
reader issue. These are new reads of new rendered artifacts.

## Maintained negative controls

Five existing suites retain their positive matrices, control cardinalities and
identity-only negative dimensions. Because the 14 operations exhaust the prior
unauthored Array/List function set, same-owner no-overlay controls use canonical
Eq instances, plus Array Show where a third control is needed. Namespace/kind,
nonempty localized readings and exact signatures are explicit. Valid wrong-owner
and Text concat controls remain. The practical full-module fixture includes the
new examples and verifies the Eq lookup exists.

All five maintained suites pass freshly: 22 tests and 4,585 assertions. Per-suite
counts are Array 4/429, List 5/464, Sequence 3/448, API corrections 7/639, and practical
collections 3/2,605. This does not replace the parent-owned full integration gate.

## Evidence and remaining scope

New evidence is under the rebuilt Transform14 evidence root, separate from
retained-context recovery inputs. Key records are source-checks/summary.json,
source-checks/source-output-manifest.json, baseline-render-receipt.json,
editorial-initial.log, editorial-final.log, body-correction-diff.json,
reader-review/initial-full-body-review.json and final-full-body-review.json,
and maintained/receipts.json.

Whole-site integration, the fresh complete published-baseline build, two bounded
candidate builds, final ledger approval, commit and publication remain under the
parent task. Scoped success and the draft backup do not establish those results.
