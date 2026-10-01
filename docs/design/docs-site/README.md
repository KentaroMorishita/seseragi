# Official Docs rebuild design

This directory records the information architecture and visual
direction for [#601](https://github.com/KentaroMorishita/seseragi/issues/601).
It is the design input for building `apps/site`; it is not a second language
specification.

## Decisions

- English keeps the unprefixed canonical route; Japanese mirrors the same page
  identity below `/ja/`. Author public explanations Japanese-first, then English
  with the same meaning, examples and constraints.
- Documentation uses a persistent expandable tree on desktop and a compact
  drawer on mobile.
- The tree has real hierarchy: area, subject group, article, module, symbol
  group, and API symbol where applicable.
- The Docs site must explain Seseragi's purpose, appeal, basic concepts and
  detailed rules without requiring the Tour. Write for everyday TypeScript
  developers; the [reader contract](reader-contract.md) defines their baseline.
  Links to `docs/spec` establish provenance but never replace explanations.
- Human-authored content is a typed Seseragi page module. Page-specific English
  and Japanese copy lives beside that page as `en.ssrg` and `ja.ssrg`.
- Rendering, layouts, navigation and semantic documentation components remain
  a first-party Seseragi application using pure Html and process-target SSG.
- Compiler-owned metadata remains the source of truth for API signatures and
  public symbol identity.

## Site map and page contracts

- [Site architecture](site-architecture.md) defines the global destinations,
  expandable sidebar groups, documentation shell and article contract. Its
  [entrance and pilot map](site-architecture.md#12-entrance-and-pilot-page-map)
  defines #697's design input for #698–#702, including a practical first-run page
  and six existing values/functions articles; these changes are not yet rendered
  or accepted merely because this design is recorded.
- [Reference content map](reference-content-map.md) maps every normative
  specification section to a stable reader-facing leaf and defines what the
  leaf must explain.
- The current map covers every H2 section in `docs/spec/00-language.md` through
  `docs/spec/17-sqlite-package.md`, plus the grammar appendix and generated
  Standard Library symbol pages.

## Approved mockups

- `seseragi-home.png`: main language-site entrance.
- `docs-language-function-application.png`: nested Guide/Language article.
- `docs-standard-library-get.png`: nested generated API symbol page.

The lower-page mockups intentionally demonstrate two different tree depths:

```text
Language Reference
└─ Syntax and operators
   └─ Function application

Standard Library
└─ Collections
   └─ std/array
      └─ Functions
         └─ get
```

## Implementation boundary

See [implementation architecture](implementation-architecture.md). The rebuild
keeps the proven deterministic SSG, safe-route, canonical-example, compiler
Reference, no-JavaScript, SEO and accessibility contracts. There is no Markdown,
MDX, JSON page array or legacy renderer in the active application.
