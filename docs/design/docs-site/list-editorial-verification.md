# List operation explanation evidence

Scope: std/list module selector and chunksOf, windows, takeWhile, dropWhile,
findIndex, init, last, reduceRight, sort, sortBy and groupBy. These are 12 existing
page identities in English and Japanese. Local work item:
[list-editorial-work-item.md](list-editorial-work-item.md).

The existing six List construction/pairing overlays are preserved. New prose is
owned by typed locale files; list-model.ssrg supplies presentation and local
syntax help. The dispatcher uses exact identities. Compiler declarations,
reference metadata and runtime are unchanged.

## Current behavioral evidence

Thirteen complete canonical programs cover positive, empty and boundary inputs,
including complete Maybe/Either consumers. They execute as isolated programs with
a 15-second bound. Expected results preserve List display markers, nested List
results, present-empty versus absent values, full windows versus short final
chunks, stable ties and first-seen group order.

Native examples, supplemental runtime callback traces and TypeScript counterpart
checks pass three tests with 76 assertions. Runtime checks cover prefix and
first-match short-circuiting, right-to-left element-first reduction, no empty
callback, stable sortBy ties, one key call per input in order, and group ordering.
The TypeScript code uses numeric comparison and copies before sorting; it does
not misrepresent default string sorting or mutation as a language limitation.

Normative source: docs/spec/10-library-surface.md §10.5. Runtime sources:
list.ts and sequence.ts. Exact signature source: the compiler-owned standard
reference artifact. Supplemental runtime instrumentation logs callbacks in the
test only; it does not suggest effects are legal inside pure Seseragi callbacks.

The production dispatcher renders 30 locale pages: 24 edited bodies and six
empty/zip/length controls. The focused render passes one test with 340 assertions,
including all code/output panels, declarations, locale field parity, module
selector titles against real headings and unchanged control summaries/examples.
The exact-identity guard test passes 14 assertions for the eleven new functions
and five wrong identity/module/namespace/kind probes. The existing six List
construction/pairing dispatcher entries remain present without duplication.

Fresh HTML is `/tmp/list-next12-rendered`. The fixture uses actual production
imports and one filtered library module; it does not establish full-catalog
navigation. TypeScript, Biome and whitespace checks pass. The Japanese advisory
on the twelve edited bodies found 39 sentence-ending notices, two technical
contrast notices and one selector-list ratio notice; results are recorded in `/tmp/list-next12-prose-edited.json`.
Independent reading covered all 24 English/Japanese bodies. It requested two
copy corrections: limit groupBy's odd/even remainder claim to the positive
integers shown, and describe println as displaying a value (some examples print
Int directly). Both locales were corrected without changing programs or output.
The final production rerender passed 340 assertions and the targeted bilingual
reread cleared both corrections and the updated shared immutable-List summary.
Paired code/output panels match. The reviewer found no remaining body blocker;
this simulated-reader evidence does not establish real-human acceptance.
Edited-artifact digest: f227584e4425f84a930cb09bc2d2277980f5d36d33deda367b8bca27c42f834e. No browser or
human-acceptance claim is made. Parent integration owns the full-site gate,
aggregate example imports and final locks.

## Checkpoint 5 exact module-title repair

The existing module's H1 is std/list. Its 22 authored leaf links previously used
List; the shared label now uses std/list in both locales, with the same URL and
adjacent purpose. The focused regression additionally checks every authored leaf
body against real destination titles, alongside the existing module selector
checks. The refreshed 30-page fixture passes 362 assertions. Across the five
author tests this brings the total to 452 assertions. TypeScript, Biome and
whitespace checks pass.

Independent targeted rereading verified that each of the 22 changed artifact
files differs only by the intended label, and preserved the earlier semantic
review of all 24 edited bodies. Current selected-artifact digest:
a800fdab37b8577a2b470a11caf61d340f99bae67121f81a9359871b57f3f047.
This replaces the earlier digest above. Browser/human/full-catalog limits remain.
