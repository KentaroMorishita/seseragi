# #348 generic collection short-circuit

## Scope and lookahead

Base: `a27d470926960ca042268a28b68ff56848ea60c0` (v0.46.0).
The #291 queue, Epic #353, foundations #329/#331/#359, and subsequent
#303/#304/#306/#307/#330 were reviewed before implementation.

- Spec 10.6 names the result `ReduceStep<A>`, with `Next` and `Done`.
- The standard interface owns the public signature and Iterable requirement.
  Existing SizeError remains available; prelude map/reduce/aggregates/traverse
  are not copied into the module.
- Runtime traversal pulls the selected dictionary's persistent Iterator, with
  neither a Reducible requirement nor an intermediate Array. Done returns its
  payload before requesting the next element.
- Imported opaque ADT constructors preserve the same canonical type identity
  as annotations, including a curried lambda result. Previously this rejected
  two types with identical display names as incompatible.
- Saturated, constrained source calls with a callback after their initial
  non-function argument carry the source-checked callable type into TypeScript.
  TypeScript otherwise commits generics before seeing that callback (`unknown`
  element types and over-narrow tuple literals). This is erased type transport,
  not a new runtime call path; source argument/evidence order is unchanged.
- `Iterable<Iterator<A>, A>` supplies the existing iterator as-is. No Reducible
  instance is added; `std/iterator` module exports remain #330 work.
- The combined `compile/short-circuit-traversal.ssrg` fixture is not claimed to
  execute fully. Its pure behavior is exercised by `collection-reduce-until`;
  Effect `forEachUntil` / `LoopControl` remains #502.

## Evidence

- `runtime/ts/tests/collection.test.ts`: source-order callback/pull trace,
  infinite input, immediate Done, finite exhaustion, empty input, immutable
  constructors, persistent input reuse, and function-valued accumulator.
  Connected to the conformance/full gate in `scripts/check-scoped.sh`.
- `schema-1/collection-reduce-until` and matching execution artifact: Array,
  List, Range, custom Iterable/Reducible (including superclass evidence),
  infinite custom Iterable/Iterator, generic element/accumulator types,
  pattern matching, first-class/partial application, callable accumulator.
- Driver tests reject ordinary accumulator results, wrong accumulator types,
  missing Iterable, unrelated user ADTs, and Reducible on Iterator.
- WASM integration executes the same source through the Playground runtime
  registry, with stdout matched against the native result.
- Real Chromium Playground at `http://127.0.0.1:5190`: enter ordinary source,
  press Run, observe `Completed` and stdout `3` for `[1, 2, -1, 100]`;
  no Vite overlay; source editor has nonzero width. Reference exposes the
  canonical generic `reduceUntil` signature and Iterable constraint. Browser
  and dev-server sessions are closed after verification.

## Integration contract

`bun run check` is required by #348 and by the compiler/runtime/WASM cross-area
change. Focused runtime/driver/WASM checks precede it. An initial fixed-count
standard-instance assertion was updated for the added Iterator instance.
Analysis, standard registry, and affected IR artifacts use the existing Rust
writers; provider-backed fixture locks use `seseragi lock update`.

Final `bun run check`: passed, including 138 generated modules, 102 execution
fixtures, 26 project executions, 37 Analysis artifacts, strict generated
TypeScript checking, Playground/WASM, native archives, and the host VSIX
package/execution smoke. The generic callback/tuple case is part of the
strictly checked canonical execution artifact.

Canonical release: v0.47.0. Merge, release publication, and local dogfood
results are recorded on #348/#291 only after their live checks succeed.
