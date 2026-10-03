# Bytes inspection11 reader batch

## Reader task

A TypeScript developer wants to inspect a small byte payload, validate one byte,
check a position or range, and join chunks. Introduce a byte as an integer from
0 through 255 and Uint8Array as a byte sequence. Author Japanese first and then
English, with every deep link teaching its own example's names and syntax.

## Exact scope

Eleven existing std/bytes identities: Byte and BytesSliceError (type/opaque-type),
plus byte, toInt, empty, singleton, isEmpty, get, slice, append and concat
(value/function). This is 22 EN/JA bodies and six complete Seseragi/TypeScript
pairs. No module overview or constructor page is added. The current 344 authored
identities are disjoint; this exact batch would bring that count to 355.

Use the existing fromInts/toInts idioms to make sample data visible. Byte is
opaque; it has no public Byte constructor or pattern. BytesSliceError retains
its canonical opaque-type classification, but its public InvalidByteRange
constructor and record pattern expose start/end/length. Only that exact
identity/owner/namespace/kind receives a new declaration-reading explanation.
All other readings, signatures, routes and kinds remain unchanged.

## Honest TypeScript comparisons

Both examples may be short. Native Uint8Array construction wraps/truncates input;
native slice normalizes negative bounds and clamps rather than rejecting them.
Explicit validation implements the desired rejection policy. Do not imply that
all TypeScript numbers are Seseragi Int values or that a checked ordinary number
acquires a nominal Byte type. Spread and flatMap examples are for the displayed
small payload, not a large-buffer performance recommendation.

Teach space-separated calls, imports, Either/Left/Right, Maybe/Just/Nothing,
record and tuple patterns, do, loops and pipe only where used. Define half-open
ranges, missing indices, suffix-first append arguments, and empty chunk behavior.
Showing unchanged input values does not establish storage or allocation identity.

## Delivery discipline

The bounded authoring candidate is checkpointed in final PR753 HEAD 85e5de3867a9
and integrated on the Mac. The transferred final evidence is recorded in
bytes-inspection-verification.md. Fresh Mac integration must use the current
compiler candidate. Authoring checks cover exact-source process checks, strict TypeScript and actual
Node/Bun execution, committed-WASM/current-browser-runtime host checks, rejection
and repair fixtures, rendered-body checks and maintained negative controls.

Read all 22 complete initial rendered bodies independently, apply corrections,
and independently reread all 22 final bodies. Record body/source/output hashes.
Model reading is not genuine first-time human acceptance. Actual browser,
whole-site integration and public deployment are separate gates. This article batch does not change compiler, runtime or dependencies; those
independent changes are handled by the unified Mac integration owner.
