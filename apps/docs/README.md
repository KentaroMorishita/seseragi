# Seseragi Docs application

Phase C (#593) established the static shell. Phase D (#594) adds a generated API
Reference landing page and one page for each public standard-library module.
Phase E (#595) provides 15 human-facing pages across Getting Started, Language
Guide, Concepts and Application Guide. Search/copy enhancements and deploy
remain O03 phases F–H.

The shell uses the same canonical `seseragi-icon.svg`, dark palette and syntax
token colors as Playground. The icon is copied from `assets/brand/public/brand`
during each build, so Docs cannot silently acquire a separate logo. Code examples are
highlighted at build time with Playground's Seseragi highlighter. Every example
links to the canonical Playground with its exact verified source preloaded;
reading and highlighting the page still require no client JavaScript.

`content/pages.json` is the structured authoring input. Paragraphs, headings,
terminal commands, links and canonical sample references are supported. Its 17
executable blocks point to 17 distinct files in `examples/spec/lessons`; source
text is never copied into content. The API Reference consumes
`examples/spec/artifacts/stdlib-schema-1/reference/module.json`, which the
compiler generates from the same module registry used for imports. The build
rejects unknown targets or namespaces, unsafe or duplicate module routes, and
duplicate export identity/kind pairs. Namespace, declaration kind, type
parameters and constraints remain compiler-owned. There is no handwritten
signature inventory.

`src/render.ssrg` owns navigation, page composition, Reference cards and complete
document rendering. `scripts/build.ts` validates both inputs, invokes the
canonical process build and sends typed JSON data to the generator over stdin.
It reads structured JSON page records and never renders HTML. This extends the
generator-command decision from #592 without adding a target.

## Build and check

Install root and Playground dependencies with `bun install --frozen-lockfile`
and `bun install --frozen-lockfile --cwd apps/playground`. Docs imports the
canonical Playground highlighter at build time. Use the matching workspace
compiler: phase C adds typed `html.meta` props, absent in v0.61.9.

```sh
cargo build -p seseragi-cli
SESERAGI_BIN="$PWD/target/debug/seseragi" bun apps/docs/scripts/build.ts /tmp/my-docs https://docs.example.com /docs/
bun run check:docs
```

OUTPUT must not exist. ORIGIN has no path; BASE is `/` or a trailing-slash path.
Mount the contents of OUTPUT at BASE. Only this directory is deployable; the
process generator and its runtime stay in a temporary build directory. Existing
output is never deleted or overwritten. Inputs and generated records are checked
before the staged output directory is renamed into place. Failed builds clean
up their own temporary output.

The site manifest records page routes, canonical source digests, Reference
language version/module/item counts and source digest, output bytes/digests
(including the shared logo), client JavaScript cost and the full separate
generator artifact manifest with provenance/retention measurements.
It is a site inventory, not an alternative compiler artifact schema. No wall-clock
time or temporary directory is part of the page output. Origin/base and source
changes intentionally change output identity.

The full integration gate includes `check:docs`. Browser verification uses the
Playground's pinned Playwright dependency and installed Chromium:

```sh
bun install --frozen-lockfile --cwd apps/playground
SESERAGI_BIN="$PWD/target/debug/seseragi" bun apps/docs/scripts/browser.ts
```

Set `DOCS_SCREENSHOTS` to retain review PNGs. Browser verification covers all 78
routes at 1280, 390 and 320 pixels with JavaScript disabled, authored and
Reference navigation, every exact-source Playground URL, canonical sample text,
syntax highlighting, metadata, skip link and document overflow. It cleans its
server, browser and generated site after verification.

## Reproduced gap and fix

The shell requires viewport and description metadata. On v0.61.9, `html.meta`
uses generic void props, so `name`/`content` produce unknown-prop warnings;
`html.attribute "name"` returns `ReservedAttributeName`. Phase C adds compiler-
owned optional `charSet`, `name`, `content`, `httpEquiv` props and their runtime
serialization. Metadata continues through pure Html and escapes once. Driver
compilation and runtime tests cover this boundary; it requires the terminal
compiler/runtime release and regenerated WASM before publication.

[Architecture and source ownership](ARCHITECTURE.md) remains the design contract.
