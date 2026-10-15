# Native Float core

Tracking: #832 under #811 / #809. This is a bounded part of the experimental
Rust target, rather than completion of the numeric library or the native target.

## Values and checked evidence

A Float is an immediate `f64` in the native value ABI. Literals are decoded once
and emitted as `f64::from_bits`, preserving subnormals and negative zero. Shared
containers, closure captures, Ref and synchronous Effect payloads retain those
bits without allocating a managed node for each Float.

The backend accepts the checked canonical Float Add, Sub, Mul, Div and Rem
instances. Generic and user instances call their selected methods; source
application stages retain their evaluation order. Standard Show, Debug, Zero
and One are explicit dictionaries. Closed Float partial application uses the
same source-arity and evidence boundary as the existing Char slice.

There is no Float Eq, Ord or Hash instance. `std/float.ieeeEq` and `totalCompare`
are explicit operations. Total comparison puts negative zero before positive
zero and NaN after infinity; all NaNs compare Equal through this explicit API.
The runtime also implements named NaN / infinities, format / fromInt,
classification, minimumNumber / maximumNumber / clampNumber, abs and sign.

Power (`**` and `std/float.power`), parse, roundIntegral, toInt and std/math remain
unsupported. Emission must return a positioned diagnostic without writing Rust
or metadata for those operations. They are subsequent slices, not host fallbacks.

## Canonical display

The existing TS runtime defines Float spelling through `canonicalFloat`.
Native spelling retains `-0.0`, `NaN` and signed infinities, fixed notation from
1e-6 up to 1e21, normalized exponents outside that interval, and `.0` for fixed
integral values.

Rust shortest formatting alone is insufficient. For example, binary64
`0x43040c2d2010d68a` is displayed as `705360935656145.3` by Rust and
`705360935656145.2` by TS. The native implementation starts with Rust's shortest
significand, then checks a possible decimal midpoint against the exact binary
value using odd integers and powers of two/five. It selects an even decimal
candidate only when that candidate still round-trips to the same bits.
The last condition matters at asymmetric power-of-two intervals, including
`5.960464477539063e-8`.

This preserves the existing TS spelling and the decimal tie policy described by
[ECMAScript Number::toString](https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-numeric-types-number-tostring).
Prelude Console's established primitive coercion is separate: direct Float
output uses `0`, `1` and `1e+21`, while Show uses `-0.0`, `1.0` and `1e21`.
Qualified Console's String argument contract is still checked by the frontend.

## Verification boundary

`float-values.ssrg`, `float-effects.ssrg` and `float-console.ssrg` cover values,
selected generic/user dictionaries, reusable partials, IEEE arithmetic, explicit
comparison, Unicode-independent decimal spelling, Ref, typed failure/recovery,
resource release, Show and Console.

The CLI test calls the unchanged generated native `display` body for 100,000
seeded bit patterns and each binary exponent with its adjacent bit patterns:
106,294 values in total. It compares the report with strict TypeScript output
in both profiles, executed in fresh Bun and Node processes. Special values and
the asymmetric midpoint regression also have focused runtime tests. This is
sampled binary64 coverage, not an exhaustive enumeration of all bit patterns.

Canonical source CI, Release, dogfood and distributed-program acceptance are
recorded in #832; local scoped checks alone do not complete that handoff.
