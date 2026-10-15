# Native Char support

Tracking: #825 (basic language umbrella #811, practical Rust target #809).

## Semantic boundary

The checked frontend and CoreIr define Char as one Unicode scalar. The native
runtime stores it as an immediate Rust `char` in `V::Char`. String keeps its
shared UTF-8 storage. Captured Chars, ADT/record/tuple fields, Ref values and
typed Effect failures use the same value representation and existing lifetime
machinery.

The native projection covers:

| Surface | Behavior |
| --- | --- |
| Char literal | One decoded scalar, including NUL, controls and astral values |
| Eq / Ord | Selected standard or generic dictionary; scalar ordering |
| Show | The scalar itself |
| Debug | Quoted literal with the existing TS dictionary's deterministic escapes |
| `std/char::codePoint` | Scalar ordinal as a safe Int |
| `std/char::fromCodePoint` | Just Char or Nothing, rejecting surrogates and out-of-range Ints |
| `std/char::toString` | One-scalar String |
| Char / String pattern | Existing checked decision tests and guards |

CoreIr uses the same String decision test for Char and String literals. The
native matcher supports both ABI variants without converting stored Char values
into Strings. Canonical Maybe and Ordering constructor identities are retained.

## Calling convention and evidence

Primitive operations use their exact canonical symbols and unary source arity.
Saturated calls invoke the corresponding native helper directly. Function values
and aliases retain the boxed curried ABI. A roundtrip through fromCodePoint and
codePoint needs at most the result Maybe node, without a temporary callable node
for each primitive call.

Standard Char dictionaries require the exact builtin argument type and selected
identity. A nominal type named Char is not accepted as a builtin argument to a
standard Char dictionary. Generic Eq / Ord operators invoke the checked
dictionary's method, including a user-defined instance's behavior.

A constrained partial such as `same '🙂'` is supported when a known pure
declaration still has source parameters and every remaining parameter and every
selected closed dictionary is Char. The native factory binds these dictionaries
before source arguments; the saved partial retains those explicit captures.
This does not infer arity from a returned function type. Other deferred
dictionary/type-constructor shapes and constrained method aliases retain
positioned unsupported diagnostics.

## Verification

- Native conversion checks the entire range 0 through 1114112, including every
  surrogate, the first out-of-range value, and safe Int extremes.
- The generated native roundtrip checks all 1114113 inputs, an independent
  interval checksum (620506872831), a per-call allocation bound and final live 0.
- The same source's generated TypeScript is checked in strict mode and executed
  in development/release on fresh Bun and Node processes. Each checks all
  1114113 inputs, 1112064 valid scalars, 324 comparison pairs, Debug boundaries
  and the reusable partial/capture fixture.
- The pure fixture roundtrips Char through a record, tuple and user ADT.
- CLI fixtures compare pure Char values plus Ref, typed failure/recovery,
  resource release and qualified Console behavior against TS. Existing scalar
  loop, closure, dictionary and Effect cases remain in the same suite.
- Malformed CoreIr Char values, nominal standard-evidence lookalikes, unsupported
  Float / Hash Char and asynchronous operations are rejected before output.

Run with the repository-pinned Bun and Rust:

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p seseragi-backend-rust
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test -p seseragi-cli --test rust_backend
```

Release acceptance also requires canonical Linux WASM freshness, the
repository-wide source gate, main integration, the canonical Release, local
dogfood and installed CLI execution. Live completion evidence belongs to #825;
a pending gate is not a completed distribution.

## Remaining scope

Float, List, general project/stdlib linking, async providers and program WASM
remain tracked under #809. Hash Char, derived container dictionaries and
unrecognized staged calls remain unsupported.

The Char source uses accepted decimal Int inputs. Base-prefixed Int spelling has
a pre-existing shared frontend/spec discrepancy tracked in #826. Strict checking
also exposed the existing web-file pending-buffer type inference problem; #827
tracks its type-only repair while preserving the strict check.
