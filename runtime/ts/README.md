# Seseragi TypeScript Runtime

This package is the runtime ABI target for the Rust compiler implementation.

It is intentionally separate from the legacy TypeScript compiler under `src/`.
Do not add new Rust-compiler runtime helpers to `src/runtime`; add only the
helpers required by `examples/spec/artifacts/runtime-schema-1/**/abi.json` here.

Current scope:

- `./effect`: minimal `Effect<R, E, A>` representation and constructors.
- `./console`: direct `println` helper required by the current generated
  TypeScript snapshots.
- `./stdin`: the initial asynchronous `readLine` helper. It owns one lazy,
  process-wide input cursor; callers must invoke it sequentially and receive
  `undefined` at EOF.

Typed failureはruntime内部のprivate carrierでdefectと区別します。`fail`だけがcarrierを発生させ、`run`は
carrierだけを`EffectResult.failure`へ変換します。任意のJavaScript throw / rejected Promiseはdefectとして
再throwし、failure channelへ暗黙変換しません。

`./stdin` is deliberately only the first runtime slice, not the full
`std/stdin` contract. It does not yet implement byte reads, configurable line
limits, strict UTF-8 diagnostics, `StdinError` conversion, concurrent-read
leases, cancellation, line streams, or a deterministic injectable test host.
The Rust compiler's ABI records the language-level `StdinError` contract, while
this bootstrap adapter currently covers only the successful line/EOF path;
host read errors still reject the JavaScript promise and are not yet a runtime
conformance claim.
