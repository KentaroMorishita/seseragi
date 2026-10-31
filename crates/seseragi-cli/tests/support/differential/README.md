# TS/native differential tests

Run the fixed CI corpus (30 programs, seeds 1170–1199):

```sh
cargo test --locked -p seseragi-cli --test differential -- --nocapture
```

Bun, Node and rustc must be on PATH. Cargo uses the caller's `CARGO_TARGET_DIR`.
The test removes its temporary sources, executables and runtime bundles, including
on assertion failure. It does not rebuild the Playground or change the compiler.

For a longer run (600 programs by default):

```sh
SESERAGI_DIFF_LONG=1 cargo test --locked -p seseragi-cli --test differential -- --nocapture
```

Choose a consecutive seed range, or reproduce one seed:

```sh
SESERAGI_DIFF_SEED=1170 SESERAGI_DIFF_COUNT=2000 cargo test --locked -p seseragi-cli --test differential generated_targets_match -- --nocapture
SESERAGI_DIFF_SEED=1170 SESERAGI_DIFF_COUNT=1 cargo test --locked -p seseragi-cli --test differential generated_targets_match -- --nocapture
```

Each seed determines its area and all choices within that area. The generator
uses SplitMix64 with fixed constants, without system randomness. Float inputs
come directly from binary64 bit patterns. Seed modulo five
selects Int arithmetic, Float display, Unicode text/Char, Array/List ordering, or
Map/Set insertion order. The fixed corpus covers all five areas. Int cases use
exact i128 arithmetic as an additional oracle for safe-integer results, truncation
toward zero, remainder signs, overflow and zero divisors. The other areas compare
the targets without treating either target as a specification oracle.

The generator constructs expressions from known types and library signatures.
Every generated batch passes through the real type checker. A compilation or
native admission failure fails the test; the runner never filters those cases.
The current generator uses features supported by both targets.

To amortize native optimization, up to 30 cases share a build. Each case is a
separate function with local values. The entry reads a case selector from standard
input, then evaluates only that function. Each target runs each case in a fresh
process with a separate temporary working directory. A Defect cannot prevent
later cases from running or share their state.
The three target processes run concurrently. The TS build uses the same driver
and process adapter as `seseragi run`, with JSON diagnostics to retain the Defect
message. Native builds use
`seseragi experimental build-native main.ssrg --entry main --out-dir <dir>`.
Bun and Node execute the same emitted process artifact.

The runner compares stdout bytes, exit code and Defect kind. It recognizes Int
overflow, division by zero and remainder by zero from the runtime diagnostic;
other Defects retain their message. Paths and stack traces are not compared.
Unexpected stderr or termination also fails the test. A mismatch prints the seed,
area, observations, oracle (when present), and a standalone Seseragi program.

## Shrink a mismatch into a regression fixture

1. Reproduce the printed seed with `SESERAGI_DIFF_COUNT=1`. Copy the printed
   standalone program into a temporary `.ssrg` file.
2. Replay that file after every reduction:

   ```sh
   SESERAGI_DIFF_REPLAY=/absolute/path/repro.ssrg cargo test --locked -p seseragi-cli --test differential generated_targets_match -- --nocapture
   ```

3. Remove half the output expressions or collection operations. Retain a change
   only if the file still type-checks, both targets build, and the same stdout,
   exit-code or Defect-kind discrepancy remains. Reduce again one item at a time.
   Remove unused bindings and imports. For numbers, try zero, one, minus one and
   the nearest safe-integer or exponent-notation boundary. For text, remove one
   Unicode scalar at a time, preserving any combining sequence or supplementary
   scalar needed for the mismatch. Reduce sort/group keys and collection entries
   without removing the equal keys or repeated insertion that exposes the bug.
4. Confirm the reduced file independently with `seseragi run` and
   `seseragi experimental build-native`. Replay does not infer an expected result;
   for an oracle-only Int failure where both targets agree, also retain the
   original seed's expected result in the reduction predicate.
5. Save the smallest useful program as
   `examples/spec/fixtures/rust-backend/differential-<area>-<description>.ssrg`.
   Derive its `.stdout` and expected exit/Defect from `docs/spec/`, not from either
   target's observed output. An overflow fixture has empty stdout and exit 70.
   If the specification does not determine the answer, report the ambiguity.
6. Add the fixture to an explicit comparison test, checking its expected stdout,
   exit and Defect on all three engines. Leave a genuine regression failing until
   the compiler repair is reviewed separately; do not change the expectation to
   the observed incorrect behavior.
