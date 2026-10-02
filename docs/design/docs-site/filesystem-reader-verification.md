# Filesystem reader verification

Status: authored-source checks and bounded independent reading are complete.
This document separates executable behavior, rendered documentation, reading
review, browser checks and publication. It makes no whole-site or
first-time-reader acceptance claim.

## Scope and reproducible sources

Sixteen existing identities are listed in
`apps/site/scripts/filesystem-reader.ts`: two module entrances and fourteen
symbol leaves. Canonical source files live under
`apps/site/examples/src/filesystem-reader/`; the four TypeScript comparisons live
under `apps/site/examples/comparisons/filesystem-reader/`. No route, compiler
metadata, runtime behavior or normative specification is changed.

The complete examples cover path validation/display, a report roundtrip, three
write choices, an inspected filesystem error, strict UTF-8 and a temporary
workspace. Every filesystem program creates only its own temporary report files.
Its panel retains Seseragi highlighting but has no browser/Playground run link.

## Commands

The focused execution and rendering checks are:

```sh
SESERAGI_BIN=/path/to/released/seseragi bun test \
  apps/site/tests/filesystem-reader.test.ts \
  apps/site/tests/filesystem-reader-editorial.test.ts
```

`SESERAGI_BIN` selects the tested CLI. The execution PATH must also contain Bun.
The renderer uses temporary verification packages rather than changing the site
package lock. `FILESYSTEM_READER_RENDER_DIR` optionally retains complete rendered
bodies for reading review. The shared site integration lane remains separate.

## Behavior established before authoring

Release CLI 0.61.19 established that CreateNew rejects a second write without
changing existing contents; Replace truncates and can create a missing file;
Append can create a file and inserts only the bytes supplied. The UTF-8 examples
preserve non-ASCII text and an initial BOM, distinguish missing-file access from
malformed text, and report InvalidUtf8 at offset 1 for bytes [65,195,40,66].
A temporary directory is removed after ordinary success and a typed callback
failure when cleanup succeeds. Authored sources are smaller standalone versions
and require their own final execution checks below.

The TS counterparts use direct filesystem flags wx, w and a. Strict decoding
uses fatal:true and ignoreBOM:true. Default string decoding substitutes a
replacement character for malformed bytes and is a different contract. Native
TextDecoder does not supply the byte offset reported by Seseragi. Both TS
runtimes used for preliminary comparison were checked separately: Node 24.19.0
and Bun 1.3.9.

## Boundaries retained in the explanations

A successful web build does not establish a supplied FileSystem service. The
browser host tested during feasibility did not populate that service. This is
not presented as a compile-time unsupported-target diagnostic or as successful
browser execution. Filesystem examples use the process runtime.

A direct String argument to readTextUtf8 in the rejected do-binding fixture
reports SES-T0101, “This position requires an Effect value.” The typed binding
fixture reports “Binding annotation type mismatch,” with expected Path and actual
String. The explanation leads with obtaining a Path through parse and handling
its result; the observed diagnostic wording remains unchanged.

The normative combined callback/cleanup rule and the observed provider boundary
must not be conflated. In an owned-resource identity-replacement probe, the direct
Effect result preserved the typed callback failure inside Right; a later
provider-package shutdown raised a separate defect. No attached cleanup diagnostic
was observed on the tested result/error and log surfaces. That observation is not
a universal absence-of-diagnostics claim, a completed CLI-success result, or a
public defect diagnosis. The beginner programs verify ordinary success/failure
with successful cleanup. They do not prove cancellation, unexpected-defect,
forced-termination, atomic-write or power-loss guarantees.

## Final focused results

The final combined execution and production-render closure run passes **12 tests,
zero failures, 1,920 assertions**. It uses release CLI 0.61.19, Bun 1.3.9,
Node 24.19.0 and the repository's TypeScript 5.8.3. All ten exact programs run;
four comparisons pass strict checking and execute on both TS hosts; three pure
Path examples compile and execute through committed WASM. Every filesystem
example's owned temporary directory is checked absent after execution.

The supplemental fixture distinguishes five real-host provider scenarios from
three injected FileSystemHost write/flush/close cases. Both use the real provider
adapter/lifecycle. The injected cases establish typed write completion failures,
not real disk-fault frequency, browser behavior or durability. See the fixture's
README for the exact boundaries and separately observed shutdown outcomes.

All 32 locale bodies were rendered through the production catalog and semantic
Block renderer. Exact identity/owner/namespace/kind selection, negative controls,
canonical source/output/declaration panels, same-identity locales, related-title
links and section-local navigation are checked. The initial combined run passed
10 tests / 1,895 assertions; it remains historical evidence rather than an
additional count to add to the final run.

The independent initial reading found corrections in eight areas: a missing
parameter arrow in prose, the displayed declaration's actual argument name,
Path construction wording, two inaccurate generated reading tails, a directional
cross-reference, local flush/close definitions and a concrete offset explanation.
The corrected render changes fourteen bodies and preserves eighteen unchanged
bodies. All 140 preformatted panels and 382 article-link occurrences remain
byte-identical across the correction.

Two exact guarded site-reading overrides correct FileSystem and FileSystemError.
The first describes a runtime-supplied service instead of implying a public
constructor. The second names operation, path, otherPath and kind, and states
that this declaration has no type parameters. Field names are verified against
`crates/seseragi-project/src/standard.rs` and exercised in the create-new program;
the canonical declaration/type-parameter metadata is retained. Wrong-owner,
namespace, kind and nearby-type controls return the original reading unchanged.
No generic opaque/struct wording or compiler metadata is changed.

Independent automated reading covered all 32 initial complete bodies, then all
fourteen corrected complete bodies. A final Japanese clarification separated
whole rejected filenames from forbidden characters; that complete body and a
small English reading refinement were reread again. All nine reader findings
are resolved. Eighteen original bodies remain byte-identical. The final
format-only rerun preserved all 32 complete HTML/article bodies unchanged.
This is independent automated review, not first-time human feedback.

Final scoped TypeScript and Biome checks pass. All 63 owned Seseragi source,
editorial and diagnostic-fixture files pass canonical format checks. One
catalog brace indentation and one TypeScript import ordering were corrected
before the final exact-source test run; no displayed program changed.

## Separate acceptance categories

- Execution and comparison: bounded final authored-source checks pass
- Rendering: all 32 complete locale bodies and exact panel/link checks pass
- Independent full-body reading: all 32 initial bodies and every corrected body read; no unresolved substantive finding
- Actual browser layout and clicked navigation: not established here
- First-time everyday-TypeScript-reader feedback: outstanding
- Shared integration, publication and deployment: separate checks

Counts, paired translations, styles or successful compilation alone do not
establish that a reader can understand and use an API.

## Advisory Japanese prose review

The existing pinned yomiyasu linter, SHA-256
`047b38a29c22ec46ef7e22d65194a8beee088b44ca7ed712ec787f6e5c96216a`,
ran in the foreground on sixteen complete rendered Japanese pages, excluding
preformatted code and protecting inline code. The initial check reports 86 sentence-ending repetition warnings; the final
check reports 87 after splitting an ambiguous filename rule into two sentences.
Both have six negative-parallelism notes and two high-list-ratio notes. The list ratios come from the generated
module API directories. Necessary distinctions about Path versus String,
FileTextError versus JS exceptions, and Unit versus the enclosing function's
String result are retained. Polite explanatory endings were not varied merely
to lower a score. The report is advisory; no lint server was started.

## Remaining audit boundary

The generic generated reading for other zero-type-parameter structs still needs
its own audit. Preserving nearby types as negative controls does not mark those
pages reader-reviewed or accepted. Actual browser layout/click-through,
first-time-reader feedback, full-site gates and publication remain separate.

## Retained final artifact fingerprints

The focused renderer can retain the same named artifact kinds through
FILESYSTEM_READER_RENDER_DIR. The final evidence uses these SHA-256 values:

- `checks.log`: `d877cafbae1498a3ee44dc44935f6f18264b21184b8b4e65bfbc1b7b46426b28`
- `body-manifest.json`: `7126914be27762288eb179241fbe3afbaa808d679d313a18742a105c6d117c92`
- `source-manifest.json`: `e618b2a89a21cd5bf54200508e14555820429983b9c1dec19f44c61503ae6c92`
- `correction-integrity.json`: `34d6d613d3e5c8e02c02ec8e3ad4ecf7d3574f0d56aca8f234e4ce29c9f5960c`
- `format-integrity.json`: `85b54399d23c503062dbc0e5f3a9ca60f7b62ba00ced1a3e7c1a361c6f5b0d32`
- `yomiyasu.json`: `bb1ce2d1708f344cb04d58f397379f4ad98d10106ce755f185f7e396040b88d4`

These hashes identify exact evidence bytes. They are not extra accepted pages,
browser proof or substitutes for the reading and runtime boundaries above.
