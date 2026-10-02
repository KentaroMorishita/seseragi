# Filesystem reader execution boundaries

The article sources and their four TypeScript comparisons live in
`apps/site/examples/{src,comparisons}/filesystem-reader`. The canonical mapping
in `apps/site/scripts/filesystem-reader.ts` records the exact expected outputs,
including normal TypeScript/Seseragi differences. All filesystem article
programs own managed temporary directories and need a process FileSystem
provider. A successful web build does not establish that service.

`apps/site/tests/filesystem-reader.test.ts` runs those exact sources with
`SESERAGI_BIN` (or the repository CLI by default), checks temporary directory
removal independently, runs strict TypeScript comparisons on Bun and Node, and
executes the three pure Path seeds with the committed WASM artifact. Set
`SESERAGI_BUN` if the Bun executable needs an explicit path.

The two `string-as-path` fixtures preserve genuine compiler rejections. The
unannotated call currently reports the indirect `This position requires an
Effect value`; the annotated binding reports `Binding annotation type
mismatch`. The tests execute the mapped valid repair after each rejection.
`path-boundaries.ssrg` supplements the smaller article examples with portable
root spellings, preserved dot segments, NUL, and rejected filename segments.

## Supplemental runtime/provider probe

`provider-boundaries.ts` imports the current, unmodified provider and runtime
sources directly. It is not generated Seseragi and is not a browser test. Bun
executes it directly; a Bun-built ESM bundle executes on Node.

- Five real-host cases verify actual bytes, cold construction and repeat
  execution, mode behavior, empty text, malformed UTF-8 and BOM preservation,
  ordinary success/callback-failure cleanup, acquisition failure, and deliberate
  resource-identity replacement during cleanup
- Three separate injected-host cases exercise successful write/flush/close,
  flush failure and close failure through `FileSystemHost`. These use the real
  adapter and lifecycle, but are deterministic injected faults, not disk-fault
  observations

The real-host cleanup-failure cases refuse to remove a replacement resource.
Acquisition/cleanup failures occupy the Left side of the returned Effect
failure; an ordinary callback failure occupies Right. In the simultaneous
callback/cleanup case, the direct run preserves the original callback error,
while later provider shutdown throws a separate `ProviderPackageDefect`. The
callback object has no added diagnostic property in this probe. This is not a
claim about every possible diagnostic channel, nor proof that the final CLI
process would exit successfully. Do not turn the specification's stronger
attached-diagnostic statement into an established behavior claim.

The injected close failure also reaches the caller as a typed failure and later
surfaces separately at provider shutdown. The current runtime attributes that
close error to ReadFile, even after a write; the probe retains that observation
without changing the runtime or inventing a different operation name.

Cancellation, unexpected defects, forced termination, atomic writes, concurrent
writers, power-loss durability and general sandbox/symlink guarantees are not
proved by these tests. No compiler/runtime repairs or snapshots are included.
