# Set operation explanation evidence

Scope: std/set module and empty, singleton, fromIterable, contains, insert,
remove, size, map, union, intersection, difference and isSubsetOf. Thirteen
existing identities are authored in both locales. The earlier toArray/toList
corrections are preserved; filter/isEmpty remain unchanged no-overlay controls.
See [local work item](set-editorial-work-item.md).

## Behavioral evidence

Twelve independent native/TypeScript pairs have identical exact stdout. Each
native and TS process is bounded to fifteen seconds, and displayed examples are
registered from the executed canonical files. The native/comparison test passes
72 assertions. TypeScript counterpart files typecheck.

Tests demonstrate first-occurrence deduplication, present zero, empty and List
conversion inputs, unchanged originals, duplicate insertions retaining position,
remove/reinsert appending, explicit right-first union/intersection arguments,
directed difference, empty subset behavior and map's collapsed result order.
The TS code copies before mutation when retaining the original is part of the
task. Explicit spread/filter/every operations avoid depending on newer Set
algebra methods. Integer examples use small values; parity uses positive inputs.

Current contracts are grounded in docs/spec/10-library-surface.md §10.5 and
runtime/ts/src/set.ts. The existing Set runtime suite passes five tests / thirty-two
assertions, including callback counts, complete hash collisions and first
representatives. Its bounded wrapper adds two assertions. The exact dispatcher
test adds fifteen assertions for twelve intended identities and five invalid
symbol/module/namespace/kind probes. Compiler declarations, runtime and canonical
reference metadata are unchanged.

Production rendering passes 652 assertions across all sixteen Set functions and
the module, thirty-four locale pages. Twenty-six bodies are authored; eight are
preserved controls. All 56 authored page-name links (32 selector and 24 footer)
are checked against actual destination H1 titles. Signature text, canonical native
and TS panels, outputs and locale field parity are also checked. Preserved
conversion outputs and unchanged filter/isEmpty summaries are asserted.

The shared reader notes are scoped to the constructs used by each displayed
program. List syntax appears only for fromIterable; parameter/result syntax
appears only for map's parity helper. Member-to-JSON output is explained only
where it is printed. Array literals, empty-Set typing, Unit, TS spread, JSON and
copy-before-mutation each receive only the relevant notes. The render test adds
264 presence/absence assertions grounded in the canonical native and TS source.

The fresh author suite totals four tests / 741 assertions and completes in
113.56 seconds, plus the nested runtime suite recorded above. Targeted TypeScript,
Biome, Seseragi formatting and whitespace checks pass. Fresh artifact:
`/tmp/set13-reader-polished`; test log: `/tmp/set13-reader-polished-test.log`.
Compared with `/tmp/set13-final-rendered`, exactly three helper paragraphs change
on each of the twenty-four leaf pages. All other HTML bytes are identical. The
two module pages and eight control pages are wholly byte-identical.

Independent reading previously covered all twenty-six authored bodies, paired
panels, output/declarations, runnable-source links and local fragments. Both
module locales now name insert/remove specifically when pointing to TypeScript
comparisons, because filter/isEmpty remain controls. The new helper-paragraph
source delta and all 72 changed EN/JA rendered paragraphs have also received an
independent reread with no remaining correction. Fresh independent checks pass
56 title anchors, 26 source-seed/native panels, 24 TS panels and 162 local
fragments. The edited-page manifest is `/tmp/set13-independent-reviewed26.sha256`
(aggregate SHA-256:
`4017dd61381d3678cadb040365f887ecf76f007f302fa3f0e0091bb3f8bcabd6`).
The earlier Japanese advisory report `/tmp/set13-prose-edited.json` predates this
helper cleanup and has not been rerun.

The real production import closure uses a filtered Set module, so this evidence
does not prove full cross-module navigation. Full-catalog integration and shared
registries are outside this focused check. No browser, human-acceptance or
publication claim is made.
