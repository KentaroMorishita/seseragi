# Shared integer literal decoding

Issue #826 closes a frontend gap exposed while testing the Rust target's Unicode
boundaries. The language specification already permits `0b`, `0o`, `0x`, their
uppercase forms, and separators. Previously `0xFF_00` became an integer `0` and an
identifier `xFF_00`, while some accepted decimal separators survived into CoreIr
and could fail native emission or conservative scalar-loop recognition.

The lossless lexer now retains complete numeric spellings, including malformed
digits, separators and suffixes. A shared decoder validates digit sequences and
accumulates an exact integer bounded by `9007199254740991`, without asking a host
number parser to interpret the source. Overflow and invalid spelling use
`SES-P0203` with the whole token's UTF-8 range. Invalid declaration names retain
their declaration diagnostic. Ranges, ordinary member access, unary minus and
decimal Float exponents remain separate language constructs; hexadecimal Float
spellings are rejected.

After syntax validation, Typed HIR stores valid integer expressions and signed
patterns as canonical decimal values. Pattern coverage therefore identifies
`42`, `0b101010`, `0o52` and `0x2A` as the same case, including normalized zero.
CoreIr and both backends receive those values with original source origins. The
formatter continues to render lossless tokens, retaining prefix case and
separators. This change adds no integer range, arithmetic, evidence, source arity,
evaluation-order or runtime ABI exception.

The executable contract is
`examples/spec/fixtures/rust-backend/integer-literals.ssrg` and its independent
expected stdout. It covers every base, safe-range endpoints, negative zero,
signed and equivalent patterns, and a scalar self-tail loop. Focused syntax and
driver tests cover malformed maximal tokens, overflow, nested templates, UTF-8
diagnostics, formatter stability and common loop recognition. CLI tests compare
native execution with both TS profiles and check strict TS output in fresh Bun
and Node processes. Playground tests compile, format and execute the same fixture
through the committed compiler WASM and reject malformed inputs before emission.

This is a common compiler correction, not a program-WASM implementation or a
general native performance claim. Release, CI and installed-tool acceptance are
tracked on #826 and the parent #809; pending gates remain pending there.

## Measured common improvement

A source that decrements an Int by `1_0` previously failed native emission and
missed shared scalar-loop recognition in TS. Canonical values unblock that same
analysis for both backends; no generated function body was rewritten for timing.
The same source emitted by canonical compiler WASM v0.61.32 and v0.61.33 used
the development profile, bundled by Bun 1.3.11 and measured on macOS arm64:

| Runtime | Previous TS | Canonicalized TS / native |
| --- | ---: | ---: |
| Bun 1.3.11 | 2.317806 ms/call | 0.172331 ms/call |
| Node 24.14.0 | 4.382902 ms/call | 0.558375 ms/call |
| Native, rustc 1.97.1, `-O` | valid separator rejected | 0.012058 ms/call |

Each call runs 20,000 or 20,001 scalar iterations. Each sample calls inputs
200000 through 200019, with checksum 40005800235. Five fresh processes per
variant use balanced rotating order, at least three warmups and one second of
warmup, then five samples each. Timing excludes compilation, bundling, module
loading, process startup and checksum validation. The native timing harness
replaces only the entrypoint and calls the unchanged generated public ABI; all
managed values are released afterward. TS shape checks confirm the old marker
path and the new scalar loop. This measures one supported kernel and does not
establish an application-wide or release-profile speedup.

Source SHA-256: `ed433d071cb4a85bc7fa4a83fe589c4509117693c53a0264395e1dd0dbccb8a8`.
Raw samples, generated sources and timing harnesses are retained under
`target/native-lifetimes/integer-spelling-performance`, outside Cargo profiles,
and the durable report is linked from #816 and #826.
