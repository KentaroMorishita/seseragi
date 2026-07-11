# Seseragi TypeScript Runtime

This package is the runtime ABI target for the Rust compiler implementation.

It is intentionally separate from the legacy TypeScript compiler under `src/`.
Do not add new Rust-compiler runtime helpers to `src/runtime`; add only the
helpers required by `examples/spec/artifacts/runtime-schema-1/**/abi.json` here.

Current scope:

- `./effect`: minimal `Effect<R, E, A>` representation and constructors.
- `./service`: explicit success / typed-failure results at host-service
  boundaries. Raw throws, rejected promises, and malformed results remain
  defects.
- `./console`: injected Console service plus cold `print` / `println` helpers.
  The live adapter maps stdout write errors into `ConsoleError`.
- `./stdin`: injected Stdin service plus cold `readLine`. The process adapter
  owns one lazy root-run-local cursor, rejects concurrent reads through the
  typed channel, and returns singleton `Nothing` at sticky EOF.
- `./show`: the runtime dictionary shape for the pure `Show<A>` trait and
  canonical dictionaries for String, ConsoleError, and StdinError. String is
  rendered without quotes; opaque host errors never expose a host object or
  stack trace.

Typed failureはruntime内部のprivate carrierでdefectと区別します。`fail`だけがcarrierを発生させ、`run`は
carrierだけを`EffectResult.failure`へ変換します。任意のJavaScript throw / rejected Promiseはdefectとして
再throwし、failure channelへ暗黙変換しません。`mapError`もcarrierだけを変換し、source Effectまたはmapperが
発生させたdefectを捕捉しません。

`./stdin` is deliberately still smaller than the full `std/stdin` contract. It
now converts host read failures to `StdinReadFailure`, owns an active-read
lease, keeps EOF sticky, and separates idempotent host cleanup from EOF. It does
not yet implement byte reads, configurable line limits, strict UTF-8 offsets,
cancellation pushback, or line streams. Those require a byte-buffer-owning
adapter rather than extending the bootstrap readline iterator indefinitely.
