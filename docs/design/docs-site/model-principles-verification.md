# Six semantic principles: reader verification

Verified 2026-10-01 from checkpoint
`882a6814d7c965da375ba56c1bef084e773820eb`. Together with
[the model concepts and grammar pass](model-concepts-verification.md), this
covers the approved ten existing pages. What-is-Seseragi and immutable-by-default
remain unchanged. This is scoped technical and agent reading evidence, not a
browser or actual first-time-reader acceptance claim.

## Six paired page identities

All routes retain `/docs/language/model/<slug>/`, their IDs and Japanese mirrors.
The purpose explanation precedes the first code. A narrow new typed
`reader-principle.ssrg` component composes existing semantic Blocks; it does not
replace the renderer or edit the original shared principle/explanation helpers.
The page-owned `explainPage` / guide / `ExplanationCopy` contract and existing
`understand-this`, `meaning`, `why`, `limits`, `related-rules` anchors remain.
Related-link labels use exact destination titles, with their purpose beside them.

| Slug | Primary canonical ID | Exact native output | Observed rejected code |
| --- | --- | --- | --- |
| expression-oriented | principle-expression-oriented (reused) | `pass\n` | SES-T0101 |
| no-hidden-danger | principle-no-hidden-danger (reused) | `not found\n` | SES-T0301 in the typed helper |
| backend-independent-semantics | model-reader-backend | `-3, -3.5\n` | SES-T0201 |
| diagnosable-behavior | model-reader-diagnostics | `3\n` | SES-T0101 |
| visible-costs | principle-visible-costs (reused) | `[2, 4, 6]\n` | No artificial invalid program |
| readable-density | model-reader-density | `7\n` | No artificial invalid program |

The later generic empty-list example reuses `principle-diagnosable-behavior`
and prints `` `[]\n ``. The metadata owner is `scripts/model-reader.ts`.
`newModelReaderExamples` registers only seven new sources, avoiding duplicate
registration of reused sources. Three are changed primary programs; four are
non-runnable rejection panels. Valid source bytes, SHA-256, highlights and
source-seeded Playground URLs are checked together.

Each primary example passes independent native lint/run. Four genuine rejected
programs and their repairs are checked; the mixed-numeric division repair is
verified with both Float and Int alternatives. Additional checks show that the
source array is unchanged, reversing pipeline stages gives 8, naming intermediate
values gives 7, and the later empty-list program runs. Seven runnable seeds also
compile through the committed WASM compiler and existing Playground runtime
with matching output, without opening a browser.

## What changed for the reader

- Expression-oriented uses the selected String result of an ordinary if, then
  explains blocks, Unit, Never and the execution boundary.
- Absence begins with one absent Array position and locally explains the import,
  call, Maybe, Just/Nothing, match branches, template and entry wrapper. It keeps
  recoverable failure, unexpected defects and foreign values distinct.
- Backend-independent uses the same signed inputs in Int and Float division;
  it does not imply a second implemented backend or equal runtime performance.
- Diagnosable behavior begins with a simple concrete type error and repair.
  Generic empty collections, constraints and instance selection are later details.
- Costs distinguishes transformation count, returned storage and a callback's
  own work from unmeasured time or allocation claims. Optimizations may preserve
  results and observable behavior; necessary observable operations, sequencing,
  failures, cancellation and resource lifetime cannot be silently changed.
- Density contains a genuine multiline pipeline. Stage comments explain the
  transformations and preserve multiline teaching layout under the formatter.
  Naming intermediate values remains a deliberate alternative.

Integer division by zero was also probed: the direct operation exits 70 as a
runtime defect. Handling `std/int.checkedDivide 0 7` through Left/Right prints
`division failed` and exits 0. The article does not equate absence, a recoverable
result and a defect, or promise that every execution is safe.

## Precisely recorded implementation discrepancy

The normative exhaustive-match rule is preserved. The current compiler misses
this particular check at a top-level let:

```seseragi
import * as arrays from "std/array"
let values = [10, 20]
let message = match arrays.get 5 values {
  Just value -> `found: ${value}`
}
pub effect fn main = println message
```

Both tested binaries allow lint/build (exit 0), then run exits 1 with
`non-exhaustive Seseragi match`. Moving the same match into an ordinary typed
helper rejects lint/build/run with exit 2 and SES-T0301, identifying Nothing:

```seseragi
import * as arrays from "std/array"
fn describe values: Array<Int> -> String = match arrays.get 5 values {
  Just value -> `found: ${value}`
}
pub effect fn main = println (describe [10, 20])
```

The complete successful program comes first. The page teaches the actual
function-level rejection and repair, then gives a brief current-version caveat.
It never presents the accepted invalid top-level form as a language feature.
Exact sources, commands, status, stdout/stderr and versions are retained in
`target/site-model-reader-gap-evidence.json`. Probe provenance is published
Linux x64 release 0.61.19 / `a8641b5a81a4` and integration development 0.61.19 /
`0ee111373e56`, dirty, with Bun 1.3.9. No compiler or normative specification
was changed and no issue/comment was posted for this finding.

## Verification and remaining limits

- `tests/model-reader.test.ts`: **11 tests / 477 assertions passing**, covering
  native behavior, repairs, seven WASM/runtime seeds and twelve real EN/JA renders.
- Scoped TypeScript/Biome, all six page/locale/guide modules, new helper and
  canonical example formatting, and diff checks pass.
- `target/site-model-reader-review` contains the twelve production-module HTML
  bodies. Tests include exact panels/output, unique IDs, same-page locale links
  and 36 bilingual related-link titles. Its tiny catalog is not a whole-sidebar,
  route-closure, CSS/mobile, keyboard or browser test.
- All twelve rendered bodies were read in both languages. An independent reader
  agent found no first-example blocker. Its optional Japanese cost clarification
  was applied: inlining expands a function body at the call site, and the
  optimization restriction concerns required observable behavior, not a blanket
  ban on inlining pure calls. The changed paragraph is re-rendered for rereading.
- The pinned Japanese prose guidance informed one bounded author advisory pass,
  excluding code/output; retained flags concern polite endings and intentional
  technical distinctions. These scores are not evidence of human acceptance.
- The global explanation preflight passes these six pages; other concurrent
  writers and full-site/lock integration are managed separately. No full-site
  generation, browser, commit, push, PR, deploy or external mutation was performed
  by this scoped pass.
