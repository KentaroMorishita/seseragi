# Bytes20 reader verification

## Scope and current evidence

The batch authors twenty existing library identities and forty EN/JA bodies:
three module overviews, five opaque-type identities and twelve pure functions.
Fourteen complete source triples cover the Seseragi program, Bun-native
TypeScript and a separately checked Node alternative. This adds explanatory
content without changing routes, canonical signatures, targets or runtime code.

The initial full-family compile selected exactly twenty identities. The current
corrected focused run passes eight tests with 4,911 assertions. Its source,
commands, outputs and production-shaped bodies are retained separately from the
prior feasibility results. Independent readers reread all fifteen changed complete bodies per locale and
verified five unchanged body/text hash pairs per locale. No required correction
remains. This document does not claim human-reader or real-browser acceptance.

## Exact source and runtime lanes

The official release CLI used for these checks reports Seseragi 0.61.19, commit
a8641b5a81a4, clean release metadata. Each exact displayed `.ssrg` source is
copied outside the canonical example package before formatting, linting,
process compilation and execution. This prevents focused checks from updating
or depending on the site's shared package locks.

All fourteen exact programs also compile through the committed WASM adapter and
execute with the current browser runtime through the existing test harness.
Their entry requires Console only, with no additional provider. These checks
cover the exact Playground seed bytes and printed observations, not a deployed
host or an interactive browser session.

The primary TypeScript codec panels run on Bun 1.3.9. They are strictly checked
with TypeScript 5.8.3 and installed Bun types 1.2.17. The Node alternatives are
strictly checked with Node declarations and actually run on Node v24.19.0.
Node does not supply the newer Uint8Array hex/Base64 methods; its comparisons
use Buffer. No casts, injected native-method declarations, polyfills or
compiler/dependency upgrades were needed.

Eleven triples print identical output across the languages. Three error-focused
triples deliberately differ: Utf8DecodeError, HexDecodeError and
Base64DecodeError show the typed byte position, length or reason available from
Seseragi, while native TypeScript reports general failure. Each exact output is
checked separately; the comparisons do not fabricate native error payloads.

## Boundary and diagnostic proof

The Seseragi contract matrix contains 57 observations:

- Five numeric-byte construction cases, including both valid endpoints, empty
  input and the first invalid value
- Ten hex inputs, including uppercase, empty, parity-first non-ASCII failures,
  bad digits, prefixes and whitespace
- Sixteen standard Base64 inputs, including every reason category, whitespace,
  alphabet/padding policy and mixed-error ordering
- Thirteen URL-safe Base64 inputs, including unpadded success, invalid lengths,
  standard-alphabet symbols, padding and nonzero unused bits
- Thirteen UTF-8 cases, including valid scalars, empty, NUL, BOM, malformed or
  truncated sequences, overlong forms, surrogate encoding and above-U+10FFFF

The native TypeScript matrix separately checks the corresponding 52 codec/UTF-8
acceptance/data observations and its default-BOM behavior. Matching
ignoreBOM:true preserves leading U+FEFF; native defaults strip it. Documentation
keeps these options beside examples that actually use TextDecoder.

A current-runtime probe verifies input/output copies, round trips for all 256
byte values through hex, and both Base64 formats for deterministic payload
lengths 0 through 64. The toInts probe mutates its directly returned host array,
not an extra copy made by the test. This is verification of the runtime boundary,
not a proposed mutable Seseragi API.

Five focused invalid fixtures produce a single SES-T0101 with the relevant
message key and actual/expected types: String passed to Base64, String passed to
UTF-8 decode, an unhandled Either passed as Bytes, a Float inside Array<Int>, and
an Int passed as Byte. Each has a complete formatted repair that executes with a
checked result. Failed exploratory fixtures and the distinction between Show
output and JSON quoting remain recorded in the separate feasibility history.

## Rendering, guards and independent reading

Production modules render all forty selected bodies in both locales. Focused
checks preserve:

- Exact identity, owner, namespace and kind dispatch; three selected modules and
  unchanged existing std/text module ownership
- Exact source panels, output text, source hashes and single matching Playground
  seed for each article
- Runtime-labelled Bun panels and Node alternatives
- Localized article paragraphs, canonical declarations and the four exact error
  type reading overrides
- Exact linked titles, same-identity locale switch, same-module previous/next,
  topic overview round trips, and the generated module/API boundary
- Unselected byte operations and existing authored text operations

Independent English and Japanese readers each read all twenty complete initial
bodies and recorded their hashes. Corrections made from those reads distinguish
byte positions from character positions, Bytes inside Right from the full
Either return, and Node output from the intentionally different Seseragi error
output. They also define the actual loop binding, TextDecoder options and BOM
locally, explain Buffer in the Node section, and link standard Base64 encoding
to its matching decoder. Fifteen bodies per locale changed; five per locale are
hash-identical to the initial reviewed bodies.

The advisory Japanese prose lint has two recorded passes. Repeated polite
sentence endings, essential negative restrictions, and the supplementary-scalar
emoji used as literal sample data are not removed merely to improve its score.
The lint score is not reader acceptance. Optional remaining self-links and
shared-wrapper wording are separate usability suggestions, not semantic proof
or a reason to alter unrelated generic declarations.

## Final focused-snapshot integrity

An independent audit found that the initial focused all-module snapshot omitted
the existing Unicode example inputs for caseFold, toLower and toUpper. The
forty selected bodies were unaffected. The focused test input now includes
those existing examples. The full eight-test gate was rerun, and its final
render preserves all forty independently reviewed complete HTML/body hashes.
All 124 unselected article bodies match the retained checkpoint13 baseline
byte-for-byte; none of the 164 focused pages contains a missing-example marker.
This was a focused-fixture completeness correction, not a runtime or authored
behavior change.

## Boundaries

Whole-site integration, publication, mobile/desktop layout and real-host
Playground execution remain later gates. No source-level compiler/runtime repair
or public issue is part of this batch. Independent model reading is not genuine
first-time human feedback. The original 106 core checkbox rows are preserved;
this batch's authored-library count is distinct from those rows and from reader
acceptance.
