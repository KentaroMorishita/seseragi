# Native persistent List core

Tracking: #834 under #811 / #809. This extends the experimental native target;
TypeScript remains the reference for source values, source application stages,
selected evidence and evaluation order.

## Representation and supported boundary

`ListEmpty` is immediate. Each cons owns one `head: V` and one `tail: V` in the
existing shared native heap. Cons does not inspect or copy its tail. A pattern
rest and `drop` return an alias to the existing suffix. `append suffix prefix`
copies only the prefix and retains the suffix. No Array/List coercion is added.
User nominal types named List never select the builtin literal/display path.

The target supports List literals, cons syntax and curried cons function values,
exact/minimum-length checked patterns and nested List element/rest projections.
The bounded named operations are `std/list.empty`, `singleton`, `length`,
`isEmpty`, `head`, `tail`, `get`, `toArray`, `reverse`, `take`, `drop`, `append`,
`filter`, `find`, and `std/array.toList`. Negative and out-of-range counts follow
the existing TS operations. Predicates run left to right; find stops on its first
match. Fully applied intrinsic calls and reusable partials keep source arity.

Show and Debug for List validate the canonical trait, container identity,
element type and selected child evidence. They invoke the selected child method
in source order, including user/generic instances. Primitive Debug preserves
String quoting/escaping and the existing Char/Float policies. Standard display
methods share the TS document-construction depth limit of 128, with restoration
after a defect. Configurable RenderDocument APIs remain unsupported.

Both List edges participate in Ref cycle tracing. The existing iterative final
owner drop queue releases long flat and deeply nested lists without recursively
dropping children. No borrow/ownership annotations are exposed to Seseragi users.

List Eq/Hash, collection typeclass dictionaries, comprehensions, sorting and the
remaining std/list API are outside this slice. They must produce a positioned
target diagnostic and no Rust/metadata output rather than a guessed dictionary.
General module/package linking, async/provider and program WASM remain tracked
separately. This is independent of the browser compiler WASM used to emit TS.

## Validation

Before the version-only release bump, local runtime23, generated-native26 and
CLI11 tests passed. The CLI compares pure17 and Effect19 sources against both TS
profiles. The two List pure fixtures additionally typecheck unchanged emitted TS
under strict mode and execute on fresh Bun and Node for both profiles. List
cases cover empty/safe Int count boundaries, reusable partials, nested patterns,
selected user Show, String escapes, Char and non-finite/signed-zero Float values.

The native tests independently check one-node cons allocation and tail identity,
100,000-element construction/traversal and flat/nested destruction on a 256 KiB
stack, predicate visitation/short circuit, display depth restoration, and traced
Ref cycles with duplicate edges, rooted callbacks and an outstanding Ref borrow.
The source Effect fixture checks retained snapshots, a List-contained callback
cycle, typed List failure and release closure lifetime.

Four additional positioned no-output rejection cases (List Eq/sort/
comprehension/Functor) passed locally after the whole CLI run; the final CI must
also run the committed expanded negative matrix. Canonical full integration is
required because this changes compiler/runtime foundations and release output.
Release, dogfood and actual distributed CLI acceptance remain pending until
recorded on #834; local tests or a merge alone do not close that gate.

## Actual normal-body measurement

The source is `list-tail.ssrg`: build a persistent List and sum it through a
checked head/rest pattern. Both targets use their existing general self-tail
loops. Only the native executable entry is replaced to call the unchanged public
curried `kernel`; no runtime or function body is optimized in a measurement copy.

On macOS arm64, Rust1.97.1 / Bun1.3.11 / Node24.14.0, median time per call is
native **2.182652 ms**, Bun **3.477750 ms**, Node **5.317754 ms**. Inputs are
10,000..10,009 nodes/call, 10 calls/sample, 6 fresh processes per variant with
5 samples each, balanced rotating order and at least 3 warmups and 1 second per
process. Checksum is 500500165 and native final managed-live is zero. Compilation,
bundling, loading, process startup and checksum checks are excluded. This is one
kernel and TS development output, not a whole-program or release-profile claim.
Previous native v0.61.34 rejects List and has no timing comparison.

Source SHA256: `6de3d3656dc5df21f76ea884c5b8b7240a583136b5e9b19ebeac5c3d9d4ab88a`.
Rust SHA256: `02ed885100c161a174591f88dd48d5e207ad061a61a74add94133b677998543f`.
TS SHA256: `3b93e328d71f9c9a5f2bf07a0ecf96b0760eb2194adbb1f7bbfe92f6811aa172`.
TS was emitted by the canonical compiler WASM v0.61.34. Verify byte identity
against the new canonical package and formal installed CLI/public compiler WASM
before accepting these as the distributed bodies. Raw samples, source and
complete harness are retained under `target/native-lifetimes/list-performance`
and linked from #816.

## Shared improvement candidates

The source's build and sum loops are outside the scalar-only CoreIr proof. TS
still emits tail markers and argument-array constructions; native uses generic V snapshots and
Rc handle operations. A common typed proof for immutable List cursors may remove
those costs while retaining source order, selected evidence, alias lifetimes and
closure snapshots. No such optimization ships in this slice.

For a wide fixed prefix pattern, each backend currently replays projection paths
from the root. Reusing proved prefix cursors is a separate shared CoreIr
opportunity; no claim of improved wide-prefix matching is made here. The tested
head/rest traversal advances one node per iteration. Physical heap counts on TS
and root CPU costs have not been measured, so generated shape alone is not an
allocation or timing claim. ADT representation/drop counterfactual #830 remains
research-only.
