# #304 Validation error accumulation

Base: `0f488defb092f1fda468b98b7b97d080ef9c0cbd` (v0.48.0).
#303 canonical release (18 jobs / 15 assets) and local CLI/LSP/official extension
dogfood completed before starting. #291, Epic #353, #301/#359 foundations and
the next #306/#307/#305/#308 leaves were reviewed. Only #304 is implemented.

## Contract

- The canonical standard-module interface owns `Validation<E, A>`, `Valid A`,
  `Invalid (NonEmptyList<E>)` and five construction/conversion helpers.
- Runtime metadata reuses standard-operation source arity and evidence routing.
  Both native embedded runtime and the manifest-generated browser registry
  include the new module; no synthetic fixture module or duplicate registry.
- Functor maps only Valid. Applicative accumulates both Invalid payloads with
  existing NonEmptyList append, left to right, without requiring Semigroup<E>.
- Conditional Eq, Show and Debug delegate both payload dictionaries. Display
  uses the shared render-document engine and NonEmptyList's document shape.
- No Monad, implicit Either conversion, empty Invalid, form framework or
  additional standard trait instances are introduced.
- A Valid function has no error payload: overloads defer inference of E until
  application to a Validation argument. Strict generated TypeScript exposed
  and now covers this generic curried-application boundary.
- Runtime application joins TypeScript-narrowed error variants and preserves
  a success-only result when both inputs are Valid. Source typing still
  enforces the same Validation error type. Explicit conversions capture the
  complete tagged union before projecting its independent payload types.

## Evidence

- Runtime tests cover all branches, callback counts, error order/duplicates,
  payload identity, callable values, conversions, Eq and rendering layouts,
  Functor identity/composition and Applicative identity/homomorphism/
  interchange/composition laws over valid and invalid values.
- `schema-1/validation-apis` executes Lesson 09 equivalent independent input
  validation, named and namespace constructors, every helper, callable values,
  ordinary pattern matching and generic Eq/Show/Debug with user dictionaries.
- `project-schema-1/imported-validation` preserves generic application and
  conditional dictionaries across module interfaces, including a first-class
  imported conversion. Both fixtures assert stdout and Console operation trace.
- `schema-1/validation-no-monad` fixes the actual missing-instance diagnostic.
  Driver tests also reject List payloads, wrong Valid payloads, missing Show,
  non-exhaustive matching and accidental standard evidence for a local
  same-named Validation type.
- Analysis/Reference and LSP completion assert canonical identities/signatures.
  The existing portable parity package adds accumulated Validation errors for
  CLI/LSP/WASM/Playground; dedicated WASM tests execute the main fixture and
  reject Monad. Package exports remain the browser resolver source of truth.
- Headless Chromium at 1710x1112: entering two invalid values and running
  `pure (+) <*> first <*> second` displayed Completed and
  ``Invalid `[first, second]``. The editor and output were both visible.
  Reference search for toEither displayed its Validation category,
  std/validation ownership, `Validation<E, A> -> Either<NonEmptyList<E>, A>`
  signature and lossless-conversion description. Both screenshots were
  visually inspected. Browser and Vite processes were closed after QA.

## Explicit remaining boundaries

The original Lesson 09 depends on text.trim/isEmpty (#306), Eq deriving (#332),
and an Effect-do match payload issue independently reproduced on released
v0.48.0 with Maybe<Int> (#510). The executable equivalent uses an explicit
empty-name predicate, handwritten Eq instances, and pure match followed by
the existing effectful loop. It does not claim those future slices are fixed.

Explicitly typed callbacks, inputs and intermediate results avoid known #503
generic inference gaps; no inference special-case was added here. A polymorphic
top-level let alias also failed during exploration; exported generic functions
plus first-class imported function use are covered, not general let-polymorphism.

Strict TypeScript additionally exposed a pre-existing curried generic emission
gap: a generic E appearing only in the second argument is fixed to unknown by
TypeScript at the first call. The released v0.48.0 compiler reproduces this with
an ordinary `Either` transform helper; evidence is recorded on #503 without
assuming the source inference and TypeScript emission causes are identical.
The imported application helper takes `(function, left, right)` together so
every generic is available at the first call. Generic application, conditional
evidence, runtime execution and strict generated-TypeScript checking remain
covered; general curried generic emission is not claimed fixed.

## Verification gate

`bun run check` is explicitly required by #304 and needed for this cross-area
compiler/runtime/WASM and release change. Focused tests and generated TypeScript
checks precede it. Final gate, browser, release and dogfood results are recorded
after verification rather than inferred from source tests.

Canonical conformance passed: 140 generated modules, 34 project compilations,
28 project executions, 104 single-module executions, 40 Analysis documents,
runtime ABI and all three standard-library surfaces. Runtime tests passed
7 tests / 61 assertions; dedicated WASM tests passed all 3 cases. Committed
WASM freshness and release-version synchronization passed.

`bun run check` passed on 2026-09-03 (`/tmp/seseragi-304-full-final.log`): all
Rust workspace tests and canonical conformance, native samples, committed WASM
freshness, runtime and release contracts, native archive extraction/smoke checks,
544 Playground tests, TypeScript and isolated production build, and official /
legacy migration VSIX verification. Earlier failing runs found the generic ABI
issues and an embedded-source/formatting race; they are not counted as passing
evidence. The final run used the stable, formatted implementation and included
the Validation runtime suite in the normal gate.
