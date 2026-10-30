`json-decoder-inferred-partial-main.ssrg` preserves the original implicit
imported partial that compiled in the language and native backend but emitted
`Rules<unknown>` under strict TypeScript (TS2322) in official115 and119. It belongs in place of
`src/main.ssrg` in `rust-backend-projects/basic-json-decoders`, with the same
vendor module. The working acceptance package supplies the type argument
explicitly and still tests the saved partial and generic opaque boundary.

The inferred variant is #1003 under #887. The120 candidate keeps the registered
Prelude sum owner while importing callable aliases, so the original inferred
type reaches the first curried stage. Its driver and CLI tests use this exact
input, without adding explicit type arguments or changing the expected output.
The explicit119 package remains a comparison. Broader partial-application
contracts and the unrecovered historical native inputs remain separate work.
