# Seseragi TypeScript Runtime

This package is the runtime ABI target for the Rust compiler implementation.

It is intentionally separate from the legacy TypeScript compiler under `src/`.
Do not add new Rust-compiler runtime helpers to `src/runtime`; add only the
helpers required by `examples/spec/artifacts/runtime-schema-1/**/abi.json` here.

Current scope:

- `./effect`: minimal `Effect<R, E, A>` representation and constructors.
- `./console`: direct `println` helper required by the current generated
  TypeScript snapshots.
