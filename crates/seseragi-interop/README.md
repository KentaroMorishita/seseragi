# Interop Semantic Graph

`seseragi-interop` owns the source-independent graph described in specification
18. It does not parse TypeScript, load packages, emit Seseragi source or execute
foreign code. Source frontends and projection/runtime adapters consume its
public Rust API and strict schema-1 JSON.

Types are addressed by `TypeId`; symbols have stable frontend-supplied
identities. Generic parameters name an explicit lexical binder. A symbol
reference enters the target declaration's own scope. Recursion is legal;
missing references and escaped parameters are rejected. Validation walks the
graph iteratively, including cyclic and deeply nested graphs.

The type vocabulary keeps structural snapshots, mutable identity, opaque
handles, nullable/optional domains, callable receivers, generic constraints
and projection relations distinct. Uninterpreted meaning has an explicit
reason rather than an implicit Any/Unknown fallback. A readonly shape is not a
promise that a foreign mutable object can be used without a decoder/copy.

Runtime bindings separately record exact host identity, export path, targets,
calling convention, evaluation approval and failure boundaries. An unresolved
boundary stays unresolved; the graph does not infer purity or cancellation
from a source signature.

`Origin` uses source identity and UTF-8 byte offsets. Both symbols and type
nodes may retain several origins. Interpretation steps record their inputs,
rule, classification, reason and optional policy identity. `origins_for_type`
follows these steps without losing the original serialized provenance.

`to_json` and `from_json` validate the graph. Unknown fields and unsupported
schemas fail. Ordered maps make serialization independent of insertion order;
array order is explicit semantic/interpretation order. `digest` identifies the
complete serialized graph. `from_json_for` also requires exact producer,
source, settings and runtime-contract identities for cache reuse.

Native validation and portable compilation:

```sh
cargo test --locked -p seseragi-interop
cargo check --locked -p seseragi-interop --target wasm32-unknown-unknown
```

This initial foundation belongs to #475/#476. Converter migration (#477),
projection (#482), compatibility reports and Inspector wiring remain separate
integration work. Library tests alone do not complete those Issues or prove
the complete Semantic Interop product.
