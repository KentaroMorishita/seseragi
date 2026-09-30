# Official site implementation architecture

## Ownership

`apps/site` is the only active official-site implementation. It owns the main
language entrance, Language Reference, Standard Library, Examples and Releases.
Playground remains an independent application, and its Tour is the sole ordered
learning path.

The former `apps/docs` implementation is removed. The new application does not
keep compatibility readers, copied CSS, a second renderer or a second content
inventory.

## Deployment ownership

The language site and Playground use independent Vercel configurations until
the root-domain cutover:

- root `vercel.json` owns the current `seseragi` Playground deployment;
- the existing `seseragi-docs` project uses `apps/site` as its Vercel Root
  Directory, so `apps/site/vercel.json` owns its Git and explicit deployments.

The site configuration is not copied over the root configuration during normal
development. Its commands return to the repository root explicitly for the
shared lockfile, Rust compiler build and first-party SSG, then publish the
artifact inside `apps/site/dist`. The final cutover must first choose and
implement the Playground's durable route, then move the site and canonical
origin to the root domain as one reviewed change.

## Page authoring model

Pages are Seseragi modules, not Markdown or JSON records. Each stable page owns
one directory:

```text
src/pages/language/syntax/function-application/
├─ page.ssrg  # typed structure and component composition
├─ en.ssrg    # English page copy
└─ ja.ssrg    # Japanese page copy
```

`page.ssrg` constructs a `PageDefinition` from typed values:

- `Paragraph`
- `Heading`
- `BulletList` and `OrderedList`
- `CodeExample`
- `Terminal`
- `Callout`
- `ApiReference`

Inline content is also typed as words, inline code, internal links, external
links, emphasis and strong text. Adding a documentation pattern means adding a
model constructor and a reusable component, not inventing per-page HTML.

## Locale ownership

English uses unprefixed canonical routes. Japanese mirrors the same page at
`/ja/`:

```text
/docs/language/syntax/function-application/
/ja/docs/language/syntax/function-application/
```

Page-specific copy stays next to the page. Shared interface copy lives under
`src/i18n/`. The page module pairs both languages structurally, so adding a
heading or semantic block cannot silently update only one locale.

## Navigation ownership

`src/navigation/catalog.ssrg` composes the page modules into the hierarchical
area, group and section tree. The same typed page values provide routes, titles,
sidebar entries and rendered output. There is no separate page-ID JSON file
that can drift from the sidebar.

`components/mobile-sidebar.ssrg` renders the mobile trigger, drawer header and
the same navigation tree as the desktop sidebar. Shared labels belong to the
locale modules. `client/mobile-navigation.ts` is a narrow browser API adapter:
native dialog opening/dismissal, focus isolation and scroll restoration only.
It does not render content or own routes. The host build publishes and links
this external script; the Seseragi HTML API intentionally does not expose
script tags. CSS keeps the same tree off-canvas when JavaScript is unavailable.

`components/language-menu.ssrg` owns the globe trigger, native disclosure,
same-page locale URLs, language names and current-language indication.
`client/language-menu.ts` adds outside dismissal, Escape/focus return and
closure when focus leaves the picker. It does not render or translate content.
The picker remains usable without JavaScript on home, index and article pages.

The catalog follows reader questions instead of specification file boundaries.
Every independently useful language concept remains an addressable leaf, while
the internal coverage map may assign several normative sections to that page.
Effect, Task, cancellation, resources, Signal, modules, interop and binding
generation never collapse into one summary article. Language and Standard
Library pages use separate navigation trees; only the current tree is rendered.

## External source boundary

The TypeScript build host may supply only data that is not site-authored:

- canonical executable sources from `examples/spec`;
- syntax-highlight spans and exact Playground URLs for those sources;
- compiler-owned Standard Library symbol metadata;
- deployment origin and product version.

The host does not supply page prose, navigation, layout or content HTML.
Seseragi decodes the closed build input and renders complete documents through
pure Html. Publishing may attach the external browser-enhancement asset, but
must not alter the content tree or author browser UI markup.

## Compiler-owned Reference

The build host decodes the canonical Standard Library Reference and Prelude
instance artifacts. It normalizes exact signatures and adds syntax-highlight
spans without authoring prose or choosing routes and navigation. Seseragi's
`src/reference/catalog.ssrg` owns the projection from compiler identities to:

- one module landing page per registered `std/*` module;
- one symbol page per compiler item, including compiler-provided and structural
  instance pages, separated by symbol kind when names overlap;
- grouped Standard Library navigation matching the approved site map;
- module indexes that link every generated child.

The module tree remains the source of generated page output. The sidebar lists
all modules and expands symbol children only for the current module, avoiding a
flat 1,812-link rail on every page. The build rejects duplicate routes,
duplicate compiler symbol keys, unresolved internal links and unresolved
fragments. `check:site` also runs the compiler conformance tests that compare
the committed Reference and Prelude artifacts with live compiler metadata.

## Source layout

```text
apps/site/
├─ src/
│  ├─ main.ssrg
│  ├─ model/
│  │  ├─ build.ssrg
│  │  ├─ locale.ssrg
│  │  └─ page.ssrg
│  ├─ i18n/
│  │  ├─ model.ssrg
│  │  ├─ messages.ssrg
│  │  ├─ en.ssrg
│  │  └─ ja.ssrg
│  ├─ navigation/catalog.ssrg
│  ├─ reference/catalog.ssrg
│  ├─ pages/<page path>/{page,en,ja}.ssrg
│  ├─ components/
│  │  ├─ article.ssrg
│  │  ├─ callout.ssrg
│  │  ├─ code-example.ssrg
│  │  ├─ on-this-page.ssrg
│  │  ├─ primitives.ssrg
│  │  ├─ sidebar.ssrg
│  │  └─ site-header.ssrg
│  ├─ layouts/{home,landing,article}.ssrg
│  └─ render/document.ssrg
├─ styles/
├─ scripts/{build,reference,check}.ts
└─ tests/{build,browser}.test.ts
```

## Rendering flow

```text
page modules + locale modules + navigation catalog
                         │
canonical examples ── BuildInput ── compiler Reference
                         │
                Seseragi process SSG
                         │
          English and Japanese static HTML
```

The site must compile with the repository CLI, produce deterministic static
routes, render at desktop and mobile widths without horizontal overflow, and
contain no unresolved build sentinels before Vercel preview. The current output
contains 3,938 English/Japanese routes: ninety-four authored site and
documentation pages plus 63
compiler-owned module pages and 1,812 compiler-owned symbol and instance pages
per locale.

## Style ownership

- `tokens.css`: design tokens only.
- `base.css`: reset and document defaults.
- `shell.css`: product header and footer.
- `home.css`: language entrance composition.
- `docs.css`: documentation frame, hierarchy and local outline.
- `article.css`: prose and semantic documentation components.
- `code.css`: highlighted code and API panels.
- `responsive.css`: breakpoint changes only.

The build concatenates these files into one static asset. Source responsibility
stays split even though the browser receives one request.
