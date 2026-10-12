# Semantic Interop corpus

Four small, legal TypeScript declarations cover primitive/null/undefined domains,
readonly/mutable collections, tuples, constraints/defaults, keyof/indexed access,
infer, conditional/mapped domains, unions/intersections/discriminants, literals and
templates, callables/callbacks/Promise, class constructor/receiver/method relations,
overloads/namespaces/merging, recursion and higher-rank policy boundaries.

`cargo test -p seseragi-dts --test semantic_corpus` compares the complete interpreted
IR, live public-surface compatibility classifications/reasons, and projection result
against `examples/spec/artifacts/semantic-interop-schema-1`. Unsupported relations
remain explicit and reason-bearing. An opaque generated declaration is not evidence
that its internal structure was projected. Snapshot refresh is explicit:

```
SESERAGI_UPDATE_SEMANTIC_CORPUS=1 cargo test -p seseragi-dts --test semantic_corpus
```

The conformance lane also runs `bun scripts/check-semantic-corpus-drift.ts`, which
uses the actual locked TypeScript checkers from the root and Playground dependencies
(5.8.3 and 5.9.3). It records version, compiler SHA-256, declaration digest, exported
checker types and diagnostics. Changes require inspection and an explicit `--write`
refresh; no compiler download or version substitution happens inside the check.

`apps/playground/e2e/interop-corpus.spec.ts` loads these same declarations into a
normal multi-file workspace and verifies the Semantic IR and Compatibility views,
including conversions whose source generation fails. The latest canonical WASM must
contain the report adapter before running the browser test.
