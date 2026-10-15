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
