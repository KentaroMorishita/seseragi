# Local work-item draft: result type, constructor and remaining operation readers

Status: local implementation record, not a public issue or first-time-human
acceptance decision. Base: checkpoint 7, 15bd8858becffa32d19f55f66d630593ac90c603.

## Selected problem and pages

An ordinary TypeScript reader can reach a type, constructor or operation directly.
Specific one-sentence summaries existed, but the fourteen selected pages did not
explain a local task, complete value/consumer, branch meaning, timing, or limits.
The selected set contains three types, six constructors and five functions:

- Prelude Maybe, Just, Nothing, Either, Right and Left
- std/validation Validation, Valid and Invalid
- std/maybe sequence
- std/either mapRight, bimap, sequence and swap

Canonical ownership, kind and routes are preserved. In particular, Maybe/Either
constructors are Prelude leaves; Validation remains an opaque-type with exported,
matchable Valid/Invalid alternatives. The previously completed fifteen module/
operation pages and StateT.get correction remain unchanged.

## Delivered approach

- Fourteen typed EN/JA/page sets, exact four-field dispatch and a kind-aware helper
- Fourteen complete native/TS pairs with matched tasks and concrete consumers
- Fifteen native boundary/rejection fixtures and selected-callback instrumentation
- Exact localized copy, canonical panels/seeds, declaration and full-body H1/link
  guards in real renderer closures
- Approved exact-identity swap and Validation-JA reading corrections, preserving
  all other generic declaration-reading fields and unrelated identities
- Preserved-count replacement of newly authored negative controls with untouched
  Show instance leaves, explicitly not marking those instances reader-reviewed

Construction versus checking, no-value versus empty-string/zero, nonempty ordered
errors, already-evaluated sequence inputs and swap's non-recovery meaning lead the
examples. TypeScript remains direct and idiomatic. The work does not require a
Tour or prior knowledge of ADTs, pattern matching, Rust or Haskell.

## Verification and separate acceptance decisions

See result-foundation-reader-verification.md for source paths, executed checks,
failed attempts, final hashes, independent reading and remaining limits. Code,
static rendering, agent reading, actual first-time-human feedback and browser UI
are separate categories. Counts and advisory style scores do not establish reader
acceptance. Parent owns checkpoint integration, locks and publication decisions.

## Remaining work

The twenty-three supporting instance leaves and five transformer families remain
outside this batch. Canonical flatMap description debt is unchanged; its existing
site overlay remains. No compiler/runtime/canonical metadata change, new route,
public issue/comment, push/PR, deployment or browser access workaround was made.


## Final local status

All fourteen identities have native/strict-TS/WASM evidence and both rendered
locale bodies. Final focused tests pass 8 / 1,652 assertions; affected prior tests
pass 6 / 1,555 assertions. Independent agent reading cleared all twenty-eight
bodies after ten complete revised-body rereads; the remaining eighteen were
independently byte-identical. Four reader-found categories were corrected locally:
unused wrapper syntax, undefined helper aliases, a temporal absence implication,
and exact Japanese constructor guidance. Full failed-timeout/guard-attempt evidence
is retained. The parent repaired the shared dispatch chain without changing its
lazy precedence. No authored edits are pending; parent integration and real
browser/first-time-human acceptance remain separate.
