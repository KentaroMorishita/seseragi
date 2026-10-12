# Semantic projection and explicit data adapters

`seseragi-interop::projection_plan` classifies common graph meaning before any
source/frontend-specific configuration or host execution. It preserves opaque
identity and generic parameter/constraint/default/application relations, exposes
readonly-record candidates and proven tagged-domain candidates, and keeps
nullable/optional/async/resource boundaries visible. A candidate is not an
implicit native cast.

Record projection requires a proven readonly snapshot and codecs for every
field. Mutable identity and nominal/class/opaque handles remain on their raw
boundary even when their properties look like a record. Recursive, open,
unknown, async, generic host/resource values require a separately proven codec.
Opaque generic arguments and input/output parameter relations remain in common
IR and raw projection; adapter policy cannot replace them with `Any`.

A tagged ADT candidate requires at least two readonly snapshot variants with a
required literal-string field in every variant and pairwise distinct tags. An
explicit policy selects one proven discriminator. Ambiguous/overlapping tags,
optional tags, identity objects, and unproven fields do not become an ADT.

## Policy contract

The strict `ProjectionPolicySet` has schema 1, kind
`seseragi-semantic-projection-policy`, a producer name/version, the complete
semantic graph digest, exact `ArtifactIdentity`, and per-type or per-public-symbol
selectors. Source/frontend/settings/runtime changes invalidate the policy.
Unknown policy fields, duplicate selectors/names, private symbol export, unsafe
code identifiers, and identity-to-data mappings fail closed. A policy never
rewrites or weakens the common graph. Policy JSON is a separate artifact from raw
binding settings, avoiding a settings/graph digest self-reference.

`seseragi-projection::generate_adapters` returns a separate `semanticSource`
artifact plus `appliedPolicies`, source origins, producer, target type IDs,
`graphDigest`, and the input boundary. The caller combines this result with the
canonical raw binding artifact and exposes these metadata in report/Inspector;
this crate does not parse TypeScript or generate a competing raw emitter.

## Checked JSON data route

The supported explicit mappings are `json-snapshot`, `json-nullable`, and
`json-tagged-union`. Every generated function consumes **`std/json::Json`** and
returns `Either<json.DecodeError, A>`. It does not consume arbitrary host opaque
objects, invoke raw getters, serialize JS objects implicitly, acquire a resource,
or change pure/effectful/failure semantics of the original foreign function.
Applications must first choose an explicit serialization or validated snapshot
ABI if they want to connect raw host values to this JSON data boundary.

The bounded codecs validate String/Bool/native Int, exact scalar literals,
finite string domains, readonly snapshot fields, immutable arrays, and nullable
data. Missing optional fields become `Nothing`; present null becomes
`Just Nothing` for an optional nullable field. Explicit JS undefined cannot be
represented as JSON and is rejected, rather than conflated with missing/null.
A tagged decoder checks the tag, validates the chosen payload, and constructs a
native ADT. Unmatched tags and invalid fields return typed decode failures.
Foreign class/mutable/nominal identities never use these data mappings.

Empty native records, recursive/generic/async/resource codecs, mixed scalar
unions, and unsupported native field names remain reason-bearing policy errors.
Policies may not bypass these checks. The generator has bounded depth/work,
field/variant counts, and emitted source bytes. It does not invent unchecked
casts or helper functions absent from the official standard library.

Handwritten application adapters may add domain behavior, explicit host
serialization, or additional validated codecs. They must preserve raw identity,
failure/resource contracts and the same validation/freshness rules. They are not
a package-specific catalog inside common IR. Generated adapters describe only
the explicit mapping above; no automatic inverse encoder is implied.

The integration tests compile generated ordinary Seseragi code, build it with
the native runtime boundary, then execute with Bun. They cover readonly
snapshots, absent/null/present optional values, nullable `Maybe`, finite tagged
ADT success, bad fields/tags, and stale/unsafe policy rejection.

## Declaration host integration

A binding entry may explicitly name a separate policy JSON artifact:

```toml
[entries.api]
declaration = "host/api.d.ts"
specifier = "./api.js"
output = "api"
projection_policy = "host/api.policy.json"
```

The policy uses `ProjectionPolicySet` schema 1 and the exact declaration graph
and artifact identity, including these settings. `apply_projection_policy`
validates it after interpretation. CLI and virtual workspace adapters read the
same logical artifact. Invalid/stale policies produce `SES-F0105` at that JSON
source and publish no partial entry output. Changing the JSON policy invalidates
a previously generated semantic API even when the settings file did not change.

The raw `gen/api` binding remains separate from the checked `gen/api/semantic`
API. The latter consumes `std/json::Json`, validates through decoder failures,
and never casts a raw host object or installs an implicit foreign-call adapter.
Applied selector/function/mapping/producer/source provenance is included in its
semantic artifact and the shared compatibility report. The Playground Inspector
shows these policies and generated source; CLI text renders the same reasons.
Removing a policy removes only that entry's owned semantic source/provenance.
