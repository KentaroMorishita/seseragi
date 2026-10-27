# API document output

`seseragi doc package` emits `dist/doc/index.html`, `search-index.json`, and
escaped source pages. The entry point accepts `--test`, `--target node`, and
`--include-private`. It uses the existing locked package loader and compiler;
document comments never supply types or defaults.

The JSON index has `schema: 1`, `kind: "seseragi-api-document"`, a generator
identity, package identity, module records, symbol records, and positioned
warnings. Symbols retain their canonical identity, namespace, public spelling,
checked interface metadata, signature, source range and link. A member records
its canonical owner. A deprecated symbol adds `deprecated: { message, since? }`
and the original `deprecationSource`; ordinary symbols omit both fields. HTML
puts the escaped notice immediately before the signature.

Public module records come from the package's explicit export paths. The
private option includes other modules and private workspace declarations under
`scope: "private"`. Public comments cannot resolve code links to that scope.
Resolved links use the declaration's module, namespace and member owner;
qualified module paths and explicitly imported aliases retain canonical
identities. Missing, ambiguous or unpublished links remain text with a warning.
A source link distinguishes the defining declaration from a facade's reexport
when the provider source is unavailable.

CommonMark rendering uses pulldown-cmark with table support. Raw HTML events
become text, and non-web executable URL schemes are removed. The static pages
have no JavaScript. Search data is a separate JSON file, not an inline script.
Comments preserve their original UTF-8 byte positions for link warnings.

With `--test`, existing fence selection, execution and report order run before
publication. A failed fence or package compilation preserves the previous
output bytes. A successful run publishes a complete staged directory and
removes stale owned files. A directory marker and output lease distinguish
generated output from user files; symlinks and nonempty unmanaged directories
are rejected. A publication failure restores the preceding directory, and a
rollback failure retains that directory at a reported recovery path.

The renderer is local tooling. It does not replace the separate Docs website,
publish a site, start a server, or execute examples without `--test`. Versioned
CLI, compiler WASM, release and dogfood acceptance remain separate gates.
