# Issue #333: comparisons through Ord

## Decision Evidence

The canonical Ord dictionaries and Ordering ADT already exist. Comparison syntax
previously used a primitive Binary fallback, while Eq and arithmetic binaries
carry selected evidence. Reuse that typed Binary/Core evidence path instead of
introducing a second dictionary resolver or synthesizing source-level match names
that could be shadowed. Eq and Ord share same-type operand evidence selection.

All four comparisons call the selected compare once, then inspect the canonical
Ordering tag. Strict operators test Less/Greater; inclusive operators exclude
Greater/Less. Operands are evaluated left-to-right exactly once per comparison.
Operator references and partial calls use the same dispatch and Bool result.
Comparison overload declarations remain unavailable: implement Ord.compare.
Float remains without standard Eq/Ord; no host comparison fallback is retained.

This changes previously primitive comparison behavior for types with Ord and
rejects missing evidence. Deriving (#332), iterator (#330), and generic inline
inference (#503) remain outside this leaf. The remaining pending section policy
is for List cons (:), owned by #298, not comparison syntax.

Operator sections passed to generic higher-order functions initially failed:
`apply (<) left right` tried selecting Ord before learning Score from later
arguments. Reuse lambda argument scheduling for operator sections (including
existing Eq/arithmetic/trait sections). Type ordinary arguments first, then
context-dependent callable expressions, while retaining source execution order.
This does not change the independent inline collection inference issue #503.

Strict TypeScript additionally exposed untyped eta-expanded comparison parameters
when returning a partial comparison from a function, and host generic inference
of operator arguments as unknown. Carry the source-checked function type through
the existing CheckedResult backend node for standard operator references and
partial comparisons. Eq and arithmetic references share the same missing-context root cause and are covered too.
The executable fixture exercises both returned references and returned partial
calls, plus generic higher-order application, under strict TypeScript and WASM.

The independent returned-callable saturation defect is tracked as #534. It also
reproduces on installed v0.54.0 without operator sections or generic declarations.
Returned functions in this fixture are bound before invocation, preserving the
strict-TypeScript test of returned operator references and partial applications.

## Validation

Focused driver coverage checks local/generic/standard evidence, function values,
and missing or mismatched evidence. Canonical executable fixtures cover reverse
user ordering and Unicode scalar ordering (different from host UTF-16 ordering).

The required final full gate is justified by compiler/lowering/WASM changes,
canonical artifact regeneration, and the explicit Issue acceptance criterion.
Final `bun run check` passed on 2026-09-05, including:

- Rust workspace tests; focused driver (5 tests) and LSP comparison hover also
  passed independently during development.
- Canonical conformance: 110 single-module executions, 30 project executions,
  36 project compile fixtures, all syntax/semantic/IR/ABI/standard catalogs.
- Native CLI: 24 executable samples, 3 Web packages, 146 Tour lessons,
  148 exercises and 148 diagnostic snapshots.
- Committed WASM freshness; 549 Playground tests and production bundle build.
- Release contract tests, host native archive extraction/smoke, 23 extension
  tests, official native-LSP VSIX extraction/smoke, and legacy migration VSIX.

The Tour comparison diagnostic now points at the mismatched String operand;
its snapshot was refreshed with `bun run tour:diagnostics:update`. The lesson
explains Int comparison without implying every numeric type has Ord. Tour and
Articles catalogs were regenerated with their canonical producers.

The local/imported Ord fixtures execute generated TypeScript under strict
checking. The WASM regression also executes the canonical local fixture. Returned
references and partial calls, higher-order Eq/arithmetic references, all four
comparison truth mappings, and Unicode scalar ordering are exercised.

Canonical v0.55.0 publication and local dogfood evidence will be attached to
Issue #333 and the #291 checkpoint after main integration.
