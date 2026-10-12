# JSON Schema second-source proof

This bounded frontend converts real JSON Schema 2020-12 input into the existing
source-neutral `seseragi-interop::SemanticGraph`. Its production dependencies
contain no TypeScript parser, DTS converter, filesystem, or runtime loader.

The subset covers string/boolean/null domains, exact integral scalar literals,
finite enums, closed object snapshots, optional properties, immutable data
arrays, local root `$defs` references (including recursion), and `anyOf`/`allOf`
domain relations. Shared core normalization preserves the same relations and
provenance contract used by other frontends. Definition names use deterministic
`SchemaDefN` public identities; the original JSON Pointer remains in provenance.

It deliberately preserves unknown validation, open objects, unbounded JSON
numeric domains, oneOf exclusivity, URI resource scopes, external references,
non-scalar/nonnative numeric literals as explicit policy-required meaning.
No constraint silently becomes `Unknown`, `Any`, a bounded integer, or a closed
record. Unsupported dialects and malformed/duplicate-key input fail. Source,
node, definition, and recursion budgets bound interpretation.

The meaning is a decoded JSON data snapshot, not a JavaScript object identity or
an assertion that arbitrary host objects are immutable. This is a type/schema
frontend proof; it does not implement JSON instance validation, decoding,
resource acquisition, or a JSON Schema runtime library. Data-only input needs no
runtime package linkage. JSON Schema syntax never enters the common model.

The integration example and tests call the existing public common-graph
projector, without invoking its TypeScript declaration frontend:

```sh
cargo test --locked -p seseragi-schema
cargo run --locked -p seseragi-schema --example schema-projection
```

The projector's current conservative type-identity policy produces opaque data
types in its existing foreign backend. Structural/ADT adapter decisions belong
to the same canonical projector (#482/#483), rather than a schema-specific
emitter. Tests check source-independent graph construction and actual generated
output, exact integer round trips, constraints, provenance, recursive relations,
malformed input, policy states, and stale projection settings.

Origins cover the full UTF-8 input document and identify the relevant JSON
Pointer in `rule`. They do not claim exact token ranges that a value-only JSON
parser cannot provide.

Semantic reference: [JSON Schema Core 2020-12](https://json-schema.org/draft/2020-12/json-schema-core)
and [Validation 2020-12](https://json-schema.org/draft/2020-12/json-schema-validation).
