# Top-level deprecation references

This is the first implementation slice of #861 and tooling specification 12.18.
The issue remains open for member metadata, API document HTML/JSON, and generated
binding/source-map metadata.

## Acceptance

```seseragi
pub deprecated "Use modern" since "0.61.0"
fn historical value: Int -> Int =
  if value < 0 then historical 0 else value

pub fn modern value: Int -> Int = value
pub fn answer -> Int = historical 41
```

Previously, lint and the editor did not report the call to `historical` in
`answer`. They must report one `SES-L0201` warning at that reference, with the
message, version, and original declaration location. The recursive use inside
`historical` is exempt. Ordinary lint succeeds; `--deny-warnings` returns 1.
Generated program output and type/evidence identities remain unchanged.

The existing executable fixture
`examples/spec/fixtures/rust-backend-projects/deprecation` remains the module and
path-package acceptance program: its curried `older`, generic `render`, `Label`
constructor, and cold Effect must keep the committed `expected.stdout` in
TypeScript development/release on Bun/Node and native execution. Lint adds
warnings at `older`/`render`/`cold` uses and re-export sites; it does not warn at
the `Label` constructor merely because its type is deprecated.

The same canonical metadata must survive named/namespace imports and re-exports,
including module and path-package boundaries. An unused import does not emit a
deprecation warning. Re-exporting an old API does emit one at the exported
binding. Shadowing a name, constructing a deprecated type's constructor, or
referring to an unrelated field must not inherit metadata. Each occurrence of
the same canonical symbol has at most one warning.

Completion and document symbols remain present and receive the LSP deprecated
tag. Hover includes message/version, semantic tokens receive the deprecated
modifier, and reference diagnostics receive `DiagnosticTag.Deprecated`.
Definition and rename preserve their original canonical targets. UTF-8 source
ranges and cross-module related locations must use the defining source, never a
range projected into the consumer's text.

Document symbols use the flat `SymbolInformation[]` protocol form, supported
without hierarchical-symbol capability negotiation. See the
[LSP document-symbol contract](https://github.com/microsoft/language-server-protocol/blob/gh-pages/_specifications/lsp/3.17/language/documentSymbol.md).

## Checks and boundaries

- Driver tests cover public/private top-level declarations, namespaces,
  shadowing, own-body exclusion, imports/re-exports, deduplication, and Unicode.
- CLI tests cover default/denied warning exit codes and JSON provenance.
- LSP tests cover tags, hover, UTF-8/UTF-16 positions, imported related locations,
  and unchanged definition/rename targets.
- Existing deprecation compilation fixtures compare TypeScript development and
  release output with unannotated programs. The final batch uses its required
  native/TypeScript/Bun/Node, compiler WASM, CI, release, and dogfood gates.

The implementation does not add member deprecation syntax/ownership, a new API
document renderer, binding/source-map metadata, or warnings based on names or
host-library versions. It does not change unrelated lint rules, type checking,
visibility, generated JavaScript, or source arity. Pending checks are not
successful checks.
