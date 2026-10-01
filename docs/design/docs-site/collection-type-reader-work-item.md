# Local work-item draft: collection type and constructor readers

Status: local draft, not a public issue or acceptance decision. Base is checkpoint 6,
`bd5d39ebaf164c32cf562624d971e531309b68c7`.

## Problem and selected batch

Everyday TypeScript readers can encounter a type or constructor leaf directly,
without knowing algebraic data types, pattern matching or functional-programming
terms. Six existing leaves had only the generic compiler-owned-symbol description;
ReduceStep, Next and Done had brief descriptions without an owning example.

The batch authors nine existing identities, eighteen EN/JA bodies:

- `std/map::Map`, `std/set::Set`, `std/non-empty-list::NonEmptyList`
- `std/prelude::Iterator`, with route and operations owned by `std/iterator`
- `std/collection::{SizeError,NonPositiveSize,ReduceStep,Next,Done}`

Canonical identity, namespace, kind, exact declaration and target metadata remain
compiler-owned and unchanged. Array/List retain the existing
`/docs/language/data/tuples-arrays-and-lists/` type/literal owner and their current
module pages. No standalone Array/List type leaf is invented. Existing module
introductions and callable overlays are preserved.

## Delivered approach

- Exact four-field dispatch: identity, owning module, namespace, item kind
- Typed per-identity `page.ssrg`, `en.ssrg`, `ja.ssrg` and the existing Block renderer
- Six canonical native tasks, six fair TS comparisons and eight rejected fixtures
- Local explanations of annotations, type parameters, construction, named result
  alternatives, match, and function/call notation where needed
- Distinct constructor questions, not function-only helper text or identical titles
- Concrete normal/empty/missing/stop cases; TS types and values compared honestly
- Exact-title related links with adjacent purposes, including the retained
  Array/List language owner

Critical restrictions remain explicit: NonPositiveSize wraps any Int rather than
validating its sign; Done is interpreted by reduceUntil rather than terminating
main; Iterator retains positions and is not a consuming JavaScript cursor;
NonEmptyList's tail may be empty; Map/Set updates preserve earlier values and their
value equality differs from JavaScript object identity.

## Verification and separate decisions

See `collection-type-reader-verification.md` for executed checks and review
boundaries. Code execution, display/navigation integrity, simulated reader review,
actual first-time human review and whole-site integration are separate evidence.
No counts, lint score, route presence or generated metadata closes reader review.
The parent integration task owns shared registries, example aggregation, locks,
complete-site checks and any decision to commit or publish.

## Adjacent instance backlog: 57 leaves, outside this acceptance scope

The existing generated instance inventory was rechecked against
`compilerReferenceModules()`. It remains a semantic-context backlog:

| Owner | Count | Existing instance names |
| --- | ---: | --- |
| Array | 13 | Applicative, Debug, Eq, Functor, Iterable, JsonDecode, JsonEncode, Monad, Monoid, Reducible, Semigroup, Show, Traversable |
| List | 13 | Applicative, Debug, Eq, Functor, Iterable, JsonDecode, JsonEncode, Monad, Monoid, Reducible, Semigroup, Show, Traversable |
| collection | 3 | Debug<SizeError>, Eq<SizeError>, Show<SizeError> |
| Iterator | 1 | Iterable |
| Map | 8 | Debug, Eq, Functor, Iterable, JsonDecode, JsonEncode, Reducible, Show |
| NonEmptyList | 12 | Applicative, Debug, Eq, Functor, Hash, Iterable, Monad, Ord, Reducible, Semigroup, Show, Traversable |
| Set | 7 | Debug, Eq, Iterable, JsonDecode, JsonEncode, Reducible, Show |

Their generic instance/signature explanation does not establish task-level
understanding. The new dispatcher explicitly rejects all 57. Future work should
select bounded tasks such as comparing/displaying values, iteration/reduction,
JSON conversion, and container-preserving transformations, then explain actual
operation constraints and order/finiteness consequences. Do not infer missing
instances, add them, or bulk-accept these leaves from this type batch.
