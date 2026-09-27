# Official site implementation architecture

## Ownership

`apps/site` is the only active official-site implementation. It owns the main
language entrance, Documentation, Examples and Releases. Playground remains an
independent application and is linked as a primary product surface.

The former `apps/docs` implementation is removed. The new application does not
keep compatibility readers, copied CSS, a second renderer or a second content
inventory.

## Deployment ownership

The language site and Playground use independent Vercel configurations until
the root-domain cutover:

- root `vercel.json` owns the current `seseragi` Playground deployment;
- `apps/site/vercel.json` owns explicit deployments to the existing
  `seseragi-docs` project.

The site configuration is not copied over the root configuration during normal
development. The final cutover must first choose and implement the Playground's
durable route, then move the site and canonical origin to the root domain as one
reviewed change.

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

Every language concept remains an independent leaf. Effects, Signals, modules,
interop and binding generation may share an area, but never collapse into one
summary article.

## External source boundary

The TypeScript build host may supply only data that is not site-authored:

- canonical executable sources from `examples/spec`;
- syntax-highlight spans and exact Playground URLs for those sources;
- compiler-owned Standard Library symbol metadata;
- deployment origin and product version.

The host does not supply page prose, navigation, layout or HTML. Seseragi
decodes the closed build input and renders complete documents through pure Html.

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
├─ scripts/{build,check}.ts
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

The vertical slice must compile with the repository CLI, produce deterministic
static routes, render at desktop and mobile widths without horizontal overflow,
and contain no unresolved build sentinels before Vercel preview.

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
