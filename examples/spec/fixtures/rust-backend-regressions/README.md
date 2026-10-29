`json-decoder-inferred-partial-main.ssrg` preserves the original implicit
imported partial that compiles in the language and native backend but currently
emits `Rules<unknown>` under strict TypeScript (TS2322). It belongs in place of
`src/main.ssrg` in `rust-backend-projects/basic-json-decoders`, with the same
vendor module. The working acceptance package supplies the type argument
explicitly and still tests the saved partial and generic opaque boundary.

The inferred variant is an unresolved shared frontend case #1003 under #887, reproduced with official v0.61.115. Retaining
it here does not claim it is repaired by the native decoder implementation.
