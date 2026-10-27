# Deprecation on API members

This extends the first #861 slice through trait, inherent, and foreign members.
The saved first slice remains a separate commit. API HTML/JSON generation,
actual DTS generated-binding metadata, and source-map metadata have separate
commits and scoped acceptance. Final integrated production acceptance remains
pending; #861 remains open.

```seseragi
pub struct Box { value: Int }
impl Box {
  pub fn read self: Box -> Int = self.value
  pub deprecated "Use read" since "0.61.0"
  fn old self: Box -> Int = self.value
}
pub fn answer box: Box -> Int = box.old
```

Previously the prefix could hide the inherent member. The member must remain
callable and produce one `SES-L0201` warning at `old`, with its original clause
and defining source. Its own body is exempt. An unrelated record field named
`old` does not receive a warning or tag. Generated program code and canonical
type, dictionary, and source-arity contracts remain unchanged.

A deprecated trait method warns on the selected method use, including when
other traits have a method with the same spelling. Merely implementing that
contract in an instance does not warn. An instance cannot introduce its own
deprecation clause. Importing a nominal receiver under an alias preserves its
inherent method owner, message, version, and original provider source.

Foreign function, value, and opaque-type notices use the same closed clause
grammar. Nested foreign namespace members keep their own identity and never
inherit a sibling or owner's notice. Host export names and package versions
are not a metadata source.

Member source provenance is an analysis table: exact SurfaceExpr ranges identify
the original callee token, and typed canonical callees/trait dispatch select
the owner. A generated call without an original callee receives no fabricated
source range. Typed HIR, Core IR, evidence frames, and their serialized schemas
do not gain a deprecation-dependent field. AnalysisSymbol retains member_owner
so inherent methods can be navigated and tagged without introducing an
unqualified lexical value or conflicts between different owners' names.

Focused acceptance covers metadata attachment and malformed-clause ranges,
selected-owner warnings, own-body and field exclusions, provider provenance,
and exact generated TypeScript bytes in both profiles. LSP member navigation
and tags reuse the shared analysis symbols.

Inherent completion retains an optional `nominalReceivers` table on the analysis
document. Its canonical owner comes from the actual final typing environment's
resolved nominal key, including inferred local aliases and imported type aliases.
A scoped type parameter with the same spelling has a distinct key and gains no
nominal methods. The table lives beside Typed HIR, is omitted when empty, and is
not recorded by ordinary compile paths. Recovery keeps the original diagnostics
and exposes only proved receiver facts. Namespace and expected-record completion
retain their existing paths; arbitrary chained-expression completion and new
trait-call resolution are outside this bounded change.
