# Shared scalar global reads

Issue #843 extends the scalar self-tail loop proof to closed monomorphic
`Int` and `Bool` literal bindings. The shared CoreIr analysis records the
canonical binding identity and primitive kind. It carries no constant value.
TypeScript output planning supplies checked predecessor summaries, and the
Rust project emitter derives the same facts from each original checked module.

The proof preserves declarations and initialization. Each global read remains
at its original expression position, including inside untaken branches.
Rust reads the existing global slot and unboxes its immediate value;
TypeScript retains its ordinary variable/import reference. Public currying,
source arity, selected arithmetic evidence and integer overflow checks stay
unchanged. Parameters take precedence over globals, while local binding shapes
outside the scalar slice retain the ordinary implementation.

Computed initializers, polymorphic values, Ref/Effect values, nominal types
and unknown canonical owners do not supply scalar global facts. This bounded
proof does not authorize constant propagation, read hoisting or moving an
initializer. General polymorphic native globals remain separate work.

`scalar-global.ssrg` compares the recognized loop with an ordinary loop,
including reusable partials and local/parameter shadowing. Native and
TypeScript probes compare primitive-operation and global-read traces across
69 integer boundary cases. Separate read traps check zero-step branches and
uninitialized reads. Forward-initializer diagnostics remain shared frontend
checks. The `scalar-globals` project fixture adds same-named globals in two
packages, namespace/reexport reads, cold Effect replay and a Ref finalizer
that reads a global after the scope body has returned.

Performance evidence belongs to #843/#816. Use the actual emitted bodies in
both TS profiles on Bun/Node and native Rust, retain raw fresh-process samples
and hashes, and compare those bodies with the canonical/public compiler and
distributed CLI. The earlier source copy replacing `increment` with `3`
remains a research counterfactual. Physical TS heap allocation requires its
own measurement; generated loop shape alone does not establish it.
