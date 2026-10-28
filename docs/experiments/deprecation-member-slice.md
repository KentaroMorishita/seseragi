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

An explicitly deprecated foreign namespace retains a separate optional
`namespaceDeprecations` interface record. Its source-produced child identities
prove the owner relationship. An original lexer token then positions a warning
at `legacy` in `legacy.read`, independently of the child's own metadata.
Nested owners, qualified type uses, module aliases, public reexports, and
private output retain their original source ranges. The same owner record
reaches editor tags, generated metadata/source maps, and actual API HTML/JSON.
Empty metadata is omitted from the previous wire format. A member notice does
not change generated program code or introduce a new resolver entrance.

The separate impl-operator slice records a final instance identity, provider,
canonical trait/method, and the original source clause. Warnings follow selected
Local/Imported evidence at an original operator or callee token. A scoped
Parameter witness does not prove a concrete implementation. The global Prelude
Add/Eq method is never marked, so other nominal or primitive instances remain
ordinary. The optional record travels through public instance contracts into
generated metadata/source maps and API HTML/JSON. Operator implementations retain
definition navigation but are not offered a textual rename that would silently
change their built-in trait contract. This branch's scoped acceptance and final
integrated release are recorded separately from the already verified namespace
checkpoint.

The `rust-backend-projects/deprecation-operators` acceptance imports an aliased
nominal through a public facade and a path dependency. Its binary operation,
saved/reused partial and standard `add` call retain their selected dictionary;
Int, another nominal, import-only sites and the implementation's own body do
not inherit a notice. The separate CLI target compares the original generated
TypeScript in both strict profiles and the original emitted Rust with/without
the clause. It then runs Bun, Node and native. Its failing variant saves an
arity-two operator before printing `saved`, executes the body only when the
second argument arrives, and requires the scope finalizer before Defect exit.
These new checks remain pending until their exact-head scoped run is recorded.

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

Inherent completion retains an optional `nominalBindings` table on the analysis
document. It records resolved value binding ids and their canonical owner from
the actual final typing environment, including inferred local aliases and imported type aliases.
A scoped type parameter with the same spelling has a distinct key and gains no
nominal methods. The table lives beside Typed HIR, is omitted when empty, and is
not recorded by ordinary compile paths. Recovery keeps the original diagnostics
and looks up the original lexical binding even when an unfinished dot is an
invalid expression. It does not guess a local binding for a chained field token.
Namespace and expected-record completion
retain their existing paths; arbitrary chained-expression completion and new
trait-call resolution are outside this bounded change.
