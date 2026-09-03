# #303 Maybe / Either specific operations

Base: `557d81b0572d110e28e915980b64d6f6e8d98dc6` (v0.47.0).
#348 release, all 15 assets, and local CLI/LSP/extension dogfood were verified
before starting this leaf. The #291 queue, Epic #353, #329/#331/#359 foundations,
and subsequent #304/#306/#307/#305 were reviewed.

## Contract and scope

- Canonical project interfaces expose four Maybe and seven Either operations,
  including 9.2's mapRight. Types/constructors and common trait methods remain
  prelude-owned. Foreign nullable adapters, Validation, and `??` are excluded.
- 10.4 now states the signatures and strict evaluation contract. Sequence and
  traverse reuse the source Traversable and existing target Applicative; they
  do not duplicate traversal or introduce target-specific collection branches.
- First Left is retained. This does not add callback short-circuiting to strict
  Traversable; reduceUntil is the separate explicit short-circuit operation.
- 9.5's conditional Maybe Semigroup/Monoid was already tracked by #303 in the
  canonical prelude audit. Both require only Semigroup of the payload. Nothing
  is identity; two Just payloads append left to right.
- A shared standard-operation routing seam preserves source arity/evidence and
  callable results, reusing the established uncurried runtime call lowering.

## Evidence

- Runtime tests: selected callback exactly once, preserved inactive payload,
  callable values, strict callback order, first failure, empty and List shape,
  custom Traversable dictionary identity, and Maybe Monoid laws/order.
- `schema-1/maybe-either-apis`: native execution with every API, Array/List/
  NonEmptyList/custom Box Traversable, generic and partial traversal, callable
  fallback/fold results, and a payload having Semigroup but no Monoid.
- `project-schema-1/imported-maybe-either`: imported higher-kinded Traversable
  and conditional Monoid constraints across ordinary module boundaries.
- Driver negative tests reject wrong payload/results, missing Traversable, and
  Maybe<Int> Monoid (Int intentionally has no canonical Semigroup).
- The portable parity package exercises both modules through the existing
  CLI/LSP/WASM/Playground route. Dedicated WASM execution uses the canonical
  single-module source and expected stdout.
- Runtime ABI, standard registry/prelude/parity, and Analysis artifacts use
  their normal producer/verification paths. Locks use `seseragi lock update`.
- LSP completion distinguishes the canonical Prelude, Maybe, and Either
  traverse identities and checks each constraint/result signature.
- Headless Chromium at 1710x1112 against a fresh Vite server: importing
  std/maybe, sequencing Array<Maybe<Int>>, and applying withDefault produced
  Completed with `[1, 2]`. Reference searches rendered withDefault and bimap
  with their correct modules, categories, generic signatures, and descriptions.
  Screenshots were captured and visually inspected; the editor and output
  remained visible. The browser session and dev server were then closed.

## Known inference follow-up

#503 retains a pre-existing generic inference gap: inline Right with an
unconstrained error parameter can be rejected despite earlier callback types.
This is reproduced by a user-defined fold on the installed v0.47.0 CLI, without
the new modules. Polymorphic callback/result cases are also recorded there;
their shared root cause is not assumed. Explicitly typed inputs/callbacks and
results in this fixture do not claim those missing inference cases are fixed.

#508 separately tracks higher-kinded match-result annotations that are emitted
as invalid TypeScript `G<Box<B>>`. This was reproduced with the installed
v0.47.0 CLI and strict TypeScript, independently of the new APIs. The custom
Traversable fixture still dispatches generic evidence and preserves Box shape,
using `map Box (f (unbox value))`; its ordinary match is in unbox. It does not
claim higher-kinded match-body annotation erasure is fixed.

Strict generated-TypeScript checking also found two new runtime signature
issues, now repaired: Either helper overloads preserve already-narrowed branch
types, and Maybe Semigroup/Monoid factories consume the canonical erased
RuntimeDictionary before projecting the payload Semigroup internally.

## Integration gate

`bun run check` is required explicitly by #303 and by this compiler/runtime/WASM
cross-area change. Focused runtime/driver and TypeScript checks precede it.
Canonical conformance passed, including 139 generated modules, 33 project
compilations, 27 project executions, 103 single-module executions, 38 Analysis
artifacts, runtime ABI, and all three standard-library artifacts.
`bun run check` passed on 2026-09-03 (`/tmp/seseragi-303-full.log`): Rust
workspace, all conformance, native samples, committed WASM freshness, 542
Playground tests, TypeScript and isolated production build, native archive
packaging/re-extraction, and official/legacy VSIX verification all succeeded.
`bun run release:check` also passed. Merge, v0.48.0 publication, and local
dogfood results belong to the post-merge issue handoff; local package checks
are not publication proof.
