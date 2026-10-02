# Effect sequencing reader batch

## Reader task

An everyday TypeScript developer prepares a release-notes preview: check a title,
require a selected template, display or recover an expected failure, and visit
labels in order until a deliberate stopping point. Every article starts with the
familiar TS task, then presents Seseragi, its benefit, and locally explained
syntax. Rust, Haskell, functional-programming terminology and advanced types are
not prerequisites.

## Exact existing scope

`apps/site/scripts/effect-sequencing-reader.ts` records all identities, namespace
and kind guards, existing routes, source IDs and expected output. The batch owns
one existing module entrance and twelve existing leaves:

- `std/effect`
- `std/effect::succeed`
- `std/effect::fail`
- `std/effect::fromEither`
- `std/effect::fromMaybe`
- `std/effect::attempt`
- `std/effect::mapError`
- `std/effect::recover`
- `std/effect::defer`
- `std/effect::forEachUntil`
- `std/effect::LoopControl`
- `std/effect::Continue`
- `std/effect::Break`

The 26 locale bodies use nine canonical Seseragi programs and nine TS counterparts.
No new route, runtime behavior, compiler-owned declaration or normative rule is
introduced. Temporal scheduling, HTTP and browser navigation remain outside this
batch. The existing core Effect pages are related material, not newly counted
coverage or prerequisites.

## Behavior to explain and preserve

- Values passed to `succeed`, `fromEither` and `fromMaybe` have already been
  evaluated. Their returned operations are still unexecuted
- `defer` calls its construction callback at each execution; it is not a cache
- `attempt` returns a known failure as Left; `mapError` changes a known reason;
  `recover` selects another operation. None catches programming defects or
  cancellation, and none automatically wraps later output operations
- `Break` and `Continue` are ordinary LoopControl values. A traversal callback
  returns them as an Effect success. Break stops successfully, not by failure
- Sequential traversal waits for each callback and does not pull another item
  after Break. Empty input still allows the outer program to print Completed
- The Console output wrapper has its own service and error requirements. These
  examples need no network, filesystem, Clock or browser-navigation provider

The old generic Break/Continue summaries incorrectly described them as Effects.
Exact editorial overrides correct those two identities. The Japanese-only
LoopControl reading names its public constructors and match usage, replacing the
conflicting function-only guidance. The adapter is limited by all four identity
fields, preserves English and the canonical signature, and rejects a changed
baseline instead of silently rewriting it.

## Checks and remaining acceptance

- [x] All nine native programs and nine strict TS comparisons agree on output
- [x] All nine exact source seeds compile and execute through committed WASM
- [x] Four genuine compile-rejection/repair pairs and four native defect probes
- [x] Current-runtime checks for callback selection, cancellation, saved values,
  delayed construction, sequential order and no pull after Break
- [x] Initial production rendering of all 26 bodies and exact-identity guards
- [x] Independent full English and Japanese reading, with concrete corrections
- [x] Corrected full-body render and paired rereading
- [ ] Whole-site integration and publication checks
- [ ] Published desktop/mobile and real Playground browser review
- [ ] Actual first-time TypeScript-reader feedback

Technical checks and independent agent reading are separate from actual
first-time-reader acceptance. See `effect-sequencing-reader-verification.md` for
commands, boundaries and the final focused result.
