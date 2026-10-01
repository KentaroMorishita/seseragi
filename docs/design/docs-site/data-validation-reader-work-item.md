# Local work-item draft: missing values, failures, and validation readers

Status: local implementation and verification record, not a public issue or a
first-time-human acceptance decision. Base is checkpoint 6,
`bd5d39ebaf164c32cf562624d971e531309b68c7`.

## Problem and selected identities

Ordinary TypeScript developers need to see what the caller receives when a value
is missing, a check fails, or several independent form fields are wrong. The
existing Maybe/Either/Validation module summaries and callable definitions were
present, but lacked enough local examples, consumers, callback timing, and reasons
to choose their operations. Prelude flatMap additionally had a false
collection-only description.

This bounded batch authors fifteen existing identities, thirty locale bodies:

- `std/maybe` and `std/maybe::{withDefault,orElse,traverse}`
- `std/either` and `std/either::{fold,mapLeft,traverse}`
- `std/validation` and `std/validation::{valid,invalid,invalidMany,fromEither,toEither}`
- `std/prelude::Monad::flatMap`

The module pages explain their alternatives before use and provide purpose-labelled
choices. Every callable explains one task locally before its exact signature.
The Validation module first defines each check, explains a normal two-argument
booking function and call, then explains pure and each <*> application. No Tour,
ADT, Rust, Haskell, or functional-programming prerequisite is imposed.

## Delivered approach

- Typed per-identity EN/JA/page files in the existing Block model
- Exact module/identity/value-namespace/function-kind dispatcher and isolated helper
- Fifteen complete canonical Seseragi sources and fifteen fair runnable TS sources
- Both selected-output consumers and practical mistakes explained locally
- Native, strict TS, committed-WASM seed, timing/rejection, and real-renderer checks
- Exact paragraph/panel/source/declaration checks and complete authored-body H1
  title/purpose guards in both locales
- Japanese-first copy, two bounded advisory extracted-prose passes, and a separate
  independent agent reading of all thirty complete rendered bodies
- One reader-found parameter-arrow explanation corrected in both Validation module
  bodies, followed by a full rerender and independent complete-body reread

Eager fallback timing, non-short-circuiting traversal callbacks, selected-only
fold/mapLeft callback bodies, nonempty ordered Validation errors, explicit
conversion boundaries, and Validation's absence of a Monad instance remain
visible. The TS comparisons do not misuse truthiness, add an imitation FP library,
replace eager traversal with a short-circuiting loop, or claim TS cannot describe
nonempty errors. Small integer inputs do not imply equivalence of all Int/number
values.

## Evidence and acceptance boundaries

`data-validation-reader-verification.md` records the actual executed commands,
outputs, diagnostics, source and HTML hashes, failed assertion-authoring attempts,
final focused suite (6 tests / 1,524 assertions), corrected full-body rerender
(1 test / 1,235 assertions), and independent reader checks. All fifteen identities
have code and static rendered-body evidence. All thirty bodies cleared independent
agent review after the two-argument explanation correction.

Desktop/mobile UI, clicked round trips, visual overflow and genuine first-time
human feedback remain unverified. The parent owns shared registration, aggregate
main imports, locks, the complete-site integration checkpoint and any commit or
publication decision. No compiler/runtime edits, canonical metadata edits, new
routes, dependency install, public issue/comment, push, PR or deployment belong
to this batch. The earlier StateT.get correction is preserved.

## Explicit remaining backlog

- `std/maybe::sequence`
- `std/either::{sequence,mapRight,bimap,swap}`
- Supporting Maybe/Either/Validation type and constructor leaves and their instances
- Five transformer module/struct families and the nineteen still-generic callable
  descriptions, with the existing StateT.get overlay retained
- Canonical upstream flatMap description correction, separately from this exact
  site overlay

These are real outstanding pages and metadata debt. The authored module
introductions do not individually accept their generated type/constructor/instance
leaves, and passing tests does not bulk-accept the whole data/validation group.
