# Checked native project linking

Tracking: #837 under #812 / #809.

The experimental Rust commands accept a package root as well as one source file:

```sh
seseragi experimental build-native \
  examples/spec/fixtures/rust-backend-projects/linked \
  --entry main --out-dir target/native-project
env PATH="" target/native-project/program
```

`seseragi.lock` must already exist and pass the ordinary strict project lock
validation. Update it explicitly with `seseragi lock update <package-root>`.
The entry module comes from `[run].entry`; `--entry` selects an unconstrained
Unit function in that exact module. A function with the same short name in a
dependency cannot become the entry. Pure entries print the native inspection
result; synchronous Effect entries execute the ordinary native plan.

## Shared frontend, separate target

`compile_core_project` and `compile_core_local_project` reuse source discovery,
package scope, export visibility, the checked module graph and dependency-ordered
semantic analysis. They return a `CoreProject` of `CoreCompilation` values before
TypeScript output planning or emission. The test compares each module's CoreIr
and typed interface with the ordinary TS pipeline and proves the CoreIr path
also accepts paths that the TS output planner rejects.

The Rust linker retains canonical module / function / constructor symbols. It
indexes user instance identities by their defining checked module; identities
for standard traits are not parsed as module names. Imported evidence must name
that exact provider and identity, with nested evidence passed to the same
factory. No short-name search or standard dictionary fallback is used. Imported
factories are not newly cached or moved. Existing source application stages,
returned-function arity, evaluation order and closure captures remain intact.
The metadata records the original modules in dependency order.

## Bounded acceptance

The fixture contains five modules in two path packages, including two files
named `lib.ssrg`, a reexport facade, nested generic dictionaries, an HKT
constraint, user Add / Show instances and a returned function. It also uses a
Ref created in another module and a retained callback that captures it. Typed
failure recovery crosses the module boundary and two scoped finalizers run in
LIFO order before the caller resumes.

Tests compare native with both TS profiles, require final managed-live=0 for
the Effect entry, and check an explicit pure entry. Counterfeit imported owner /
local evidence, missing or reordered graph nodes, wrong entry-module selection,
private and opaque access, missing / stale locks and unsupported imported source
are rejected. Target diagnostics carry the logical owner separately from the
portable CoreIr source name; the CLI maps it to the loaded physical source and
keeps the original byte range, even when basenames coincide.

Top-level bindings and their initialization / global-root semantics remain
unsupported, including unused bindings. Derived dictionaries, supertraits,
unimplemented standard operations, async / providers, foreign interop and
general program WASM retain their existing boundaries. Unsupported code in any
linked source fails before Rust, metadata or executables are written. This slice
does not complete #812 or imply TS backend parity.

No new performance optimization is claimed by linking. The earlier TS/native
measurements and shared CoreIr candidates in #816 remain separate evidence.

## Validation before canonical distribution

Local scoped checks passed: driver 140, native runtime 23, generated native 26,
project ownership 2, and CLI 13 tests. The CLI lane includes the existing 36
single-source comparisons and the five-module path-package case in both TS
profiles. Canonical artifact freshness, exact-head integration, official Release,
dogfood and installed CLI acceptance are tracked on #837; these checks alone do
not claim that handoff is complete.
