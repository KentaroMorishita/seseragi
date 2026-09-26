# Docs rebuild implementation architecture

## Reuse versus replacement

Keep these O03 contracts:

- process-target Seseragi generator and pure `html.renderDocument` output;
- explicit origin/base inputs and safe trailing-slash routes;
- compiler-owned Standard Library metadata;
- canonical example source digests and exact Playground links;
- deterministic production manifests and repeated-build comparison;
- static no-JavaScript content, search/copy as narrow enhancements;
- link, fragment, SEO, accessibility, viewport and artifact-budget gates;
- the canonical brand assets under `assets/brand`.

Replace these implementation shapes:

- the single human-authored `content/pages.json` array;
- the flat `navigation: Bool` list;
- the single `src/render.ssrg` module owning model, components, layout and
  document rendering;
- the monolithic `public/docs.css` file;
- module-only Reference routes with in-page symbol anchors;
- fixed Japanese rendering and hard-coded route/search totals.

## Target source layout

```text
apps/docs/
├─ content/
│  ├─ sitemap.json
│  ├─ glossary/
│  │  ├─ en.json
│  │  └─ ja.json
│  └─ pages/
│     ├─ home/
│     │  ├─ en.md
│     │  └─ ja.md
│     ├─ get-started/...
│     ├─ language/...
│     ├─ standard-library/...
│     ├─ applications/...
│     ├─ interop-and-projects/...
│     ├─ tooling/...
│     └─ internals/...
├─ src/
│  ├─ main.ssrg
│  ├─ model/
│  │  ├─ document.ssrg
│  │  ├─ navigation.ssrg
│  │  ├─ page.ssrg
│  │  └─ reference.ssrg
│  ├─ components/
│  │  ├─ article.ssrg
│  │  ├─ breadcrumb.ssrg
│  │  ├─ callout.ssrg
│  │  ├─ code-example.ssrg
│  │  ├─ language-switch.ssrg
│  │  ├─ on-this-page.ssrg
│  │  ├─ page-links.ssrg
│  │  ├─ search-root.ssrg
│  │  ├─ sidebar-tree.ssrg
│  │  └─ site-header.ssrg
│  ├─ layouts/
│  │  ├─ article.ssrg
│  │  ├─ landing.ssrg
│  │  └─ reference.ssrg
│  └─ render/
│     ├─ block.ssrg
│     ├─ document.ssrg
│     └─ page.ssrg
├─ styles/
│  ├─ tokens.css
│  ├─ reset.css
│  ├─ document.css
│  ├─ shell.css
│  ├─ navigation.css
│  ├─ article.css
│  ├─ code.css
│  ├─ components.css
│  └─ responsive.css
├─ scripts/
│  ├─ content/
│  │  ├─ discover.ts
│  │  ├─ parse.ts
│  │  ├─ validate.ts
│  │  └─ prepare.ts
│  ├─ reference/
│  │  ├─ prepare.ts
│  │  └─ routes.ts
│  ├─ build.ts
│  ├─ browser.ts
│  ├─ quality.ts
│  └─ production.ts
└─ tests/
   ├─ content.test.ts
   ├─ navigation.test.ts
   ├─ reference.test.ts
   ├─ render.test.ts
   └─ production.test.ts
```

Names may be refined during implementation, but responsibility may not be
collapsed back into one renderer, one page array or one stylesheet.

## Authoring and rendering boundary

Human prose uses one Markdown file per stable page identity and locale. A small
closed directive vocabulary represents semantic components such as Example,
FromTypeScript, DesignRationale, CommonMistake, Warning, Availability,
Prerequisites, Related, NextSteps and ApiReference.

The host build layer performs only filesystem discovery, Markdown parsing,
source/provenance loading and validation. It emits a versioned, closed Doc AST.
It does not render HTML.

Seseragi owns:

- the typed Doc AST decoder;
- semantic component rendering;
- layouts and navigation presentation;
- complete document composition through pure Html;
- locale-aware links and accessible markup.

This retains a Seseragi-built site without forcing long-form authors to write
escaped prose inside `.ssrg` source files.

## Page identity and locale routes

Each page has one locale-independent id and one position in the navigation
tree. English is the canonical unprefixed route and Japanese is the `/ja/`
mirror:

```text
page id: language.syntax.function-application
English: /docs/language/syntax/function-application/
Japanese: /ja/docs/language/syntax/function-application/
```

Locale files may differ in prose structure but share page identity, canonical
examples, Reference identities, prerequisites and related edges. Missing
Japanese content is a build-visible availability state; production does not
silently display English under `lang="ja"`.

## Navigation model

The navigation tree and concept graph are separate typed inputs.

- The tree owns sidebar order, nesting, disclosure state, previous and next.
- The concept graph owns prerequisites, related pages and reader journeys.
- Breadcrumbs derive from tree ancestry.
- On-this-page derives from the rendered heading tree.
- Search records area, page kind, locale, hierarchy and symbol kind.
- Generated API symbols appear as children of their module and symbol group.
- Mobile uses the same tree in a modal drawer; it never emits the whole tree
  before the article.

## Style ownership

- `tokens.css`: colors, typography, spacing, borders, layers and code tokens.
- `document.css`: document defaults and accessibility utilities.
- `shell.css`: global header, secondary tabs and three-column frame.
- `navigation.css`: sidebar tree, disclosure, active path and mobile drawer.
- `article.css`: prose rhythm, headings, tables and page footer.
- `code.css`: code panel, syntax tokens and copy/Playground actions.
- `components.css`: callouts, metadata, badges and semantic blocks.
- `responsive.css`: breakpoint behavior only; no component semantics.

## Migration sequence

1. Freeze the complete site map and coverage ledger.
2. Introduce file-per-page authoring and the versioned Doc AST beside the old
   pipeline.
3. Split the Seseragi renderer into model, component, layout and render modules.
4. Implement the hierarchical shell and split styles against a small bilingual
   vertical slice.
5. Integrate compiler Reference as module and symbol pages.
6. Migrate and rebuild content by documentation area.
7. Switch search/SEO/quality/production to the new route inventory.
8. Delete `pages.json`, the legacy flat renderer and obsolete CSS only after
   parity gates pass.

The old site remains buildable during steps 2–6. There is one final cutover;
there are not two long-lived Docs products.

