# Remaining Effect, lifecycle, and Signal reader verification

Local work-item draft: `../seseragi-effects-11-issue-draft.md`, under #601/#630.
No public work item or progress comment was posted for this batch. Baseline is
local checkpoint `882a6814d7c965da375ba56c1bef084e773820eb`.

## Exact scope

The eleven existing paired routes under `/docs/language/effects/` are:
`effectful-for`, `task`, `sequential-and-parallel`,
`cancellation-and-resources`, `scheduler-fairness`, `fiber-supervision`,
`signals-and-transactions`, `derived-signals`, `signal-operators`,
`subscriptions-and-lifetime`, and `exceptions-and-algebraic-effects`.

The ten earlier #710 pages are outside this batch. Original page identities,
including shortened `cancellation-resources`, `signals-transactions`,
`subscriptions-lifetime`, and `exceptions-algebraic`, remain unchanged.

Each page now starts with a complete, bounded process program, explains its
names and results locally, and presents a rejected variation with a concrete
correction. English and Japanese prose remains in the page's own typed locale
files. Existing detailed rule/type/evaluation/diagnostic sections and legacy
fragments remain, with an explicit declaration-only bridge. Related links name
the purpose of the destination. No compiler, runtime or normative source changed.

## Sources and concrete checks

Normative sources: `docs/spec/05-effects.md` §§5.4, 5.7–5.16 and
`docs/spec/10-library-surface.md` §10.11. Implementation evidence includes
`runtime/ts/src/effect.ts`, `signal.ts`, `ref.ts`, `deferred.ts`, the existing
`effect-resource` and `effect-concurrency` probes, and current compiler-owned
reference signatures.

The eleven complete examples and eleven rejected variations are registered by
`scripts/lifecycle-readers.ts`. Each native execution has a 15-second process
limit. Examples use only local state and console output, with no network,
filesystem mutation, payment, external communication or subprocess spawning.

| Topic | Observed result and meaning |
| --- | --- |
| for | Aki, Mio, done: empty item skipped and empty array performs no body |
| Task | `[0, 1, 2]`: creating a Task does not run it; two runs are not cached |
| sequential/parallel | Both result arrays `[10, 20]`, empty input `[]`; no claim of an observed wall-clock order |
| resources | `42 / BA / expected / BAC`: same-scope LIFO release and cleanup before typed-failure recovery |
| fairness | `[3, 3]`: two finite procedures complete three explicit yielding steps each; not proof of time slicing or strict alternation |
| fibers | `42 / true / 21`: a readiness handshake precedes scope exit; a blocked child is cancelled and cleaned up before the parent result, followed by an explicit join |
| Signal transaction | `[2, 12] / 12`: subscriber sees initial and complete updated area, not a mixed intermediate area |
| derived Signal | `22 / 45`: explicit read-only sources combined through `<$>` and `<*>`, then read into Effect |
| Signal operators | `0 / 5 / 10`: captured integer stays unchanged; later read differs; infix multiplication remains distinct |
| subscriptions | `[1, 2, 2] / 3`: initial notification and duplicate writes are observed; scoped release stops later notification without freezing the source |
| expected failure | `ok: 2 / error: must be positive`: recover handles declared failure, not defects or cancellation |

The invalid cases use observed diagnostics. In particular, Signal `>>=` lacks a
Monad instance (`SES-T0201`), a fallible subscription observer conflicts with
`Never` (`SES-T0101`), and this unsupported try/catch spelling is reported as
unresolved names (`SES-N0001`). The inherited assertion that this exact example
must be a parser error was corrected in both locales.

A supplemental runtime test applies two plans to one source in order, checks
one stable notification, injects a harmless staging defect and verifies neither
source nor notifications changed, then checks repeated unsubscribe. Existing
resource/concurrency probes preserve cancellation, cleanup, child ownership and
failure-selection contracts. The native examples remain the primary evidence
for the language syntax and visible output.

## Verification and review status

Initial focused native/rejection/runtime selection passed three tests and
79 assertions. The first 22-locale production rendering completed but a test
correctly found misleading Playground links on legacy fragments without main.
The integration owner changed exactly eleven verified declaration-only legacy
descriptors to `standalone=false`; complete new examples remain runnable.

The final focused suite passed five tests with 604 assertions, including all
22 current-source locale pages. TypeScript checking, Biome and whitespace
checks passed. Current HTML is retained at `/tmp/lifecycle-effects-rendered`.
The Japanese prose advisory covered eleven bodies: 38 repeated-ending notices
and five technical distinctions using negative contrasts. The distinctions
explain actual differences (Fiber versus success value, Task versus Unit,
input versus completion order) and were retained. Independent reading covered
all 22 bodies, and the targeted reread cleared the final corrections. This
agent text review does not establish human acceptance. Its fixture uses real production page import
closures and example descriptors, with an empty navigation-area list. It does
not establish full-catalog navigation, browser layout, real-human acceptance or
publication. Parent integration owns full-site checks and final locks.

## Independent reading corrections

The reviewer read all 22 English/Japanese bodies and found no semantic or
example-output blocker. Two local explanation fixes followed: diagnostic
paragraphs after rejected examples now use direction-neutral wording, and
resource/fiber/subscription pages explain `$` locally. The resource page also
explains `|>`; subscriptions explicitly explains `:=` and `*source` as
Effect-producing operations, with `<-` receiving the current integer.
No executable example changed. The final focused rerender passed one test
with 326 assertions. The independent targeted reread confirmed both corrections
and found no remaining blocking explanation issue in the 22 reviewed bodies.

Final Japanese advisory (eleven bodies): {'sentence_end_repetition': 38, 'negative_parallelism': 5}.
The technical distinctions were retained; no further style pass was run.
Current report: `/tmp/lifecycle-effects-prose-final.json`.

## Full-catalog title correction

Checkpoint 3 identified 66 purpose-only related-link labels across these 22
locale pages. Each now links the exact localized destination title, followed
by its original purpose as ordinary text. Destinations are unchanged. The
focused production rerender passed one test with 612 assertions; its regression
checks all 66 links against current destination locale titles, including pages
outside the focused fixture. A separate comparison against the retained full
catalog's destination H1 headings passed all 66 links and confirmed every
original purpose and URL was preserved. Fresh HTML: `/tmp/lifecycle-effects-rendered`.
TypeScript, Biome and whitespace checks passed. This verifies link text and
rendered structure; browser and human-acceptance limits above still apply.
