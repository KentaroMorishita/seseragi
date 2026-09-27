# Seseragi official site

`apps/site` is the first-party Seseragi static-site generator for the language
home page and documentation. Page content is authored as typed Seseragi
modules; Markdown, MDX and JSON page records are not part of the rendering
pipeline.

## Responsibilities

- `src/pages/`: one directory per stable page, with `page.ssrg`, `en.ssrg` and
  `ja.ssrg`.
- `src/model/`: closed page, navigation, locale and build-input types.
- `src/components/`: reusable documentation patterns such as code examples,
  callouts, API references, sidebars and in-page navigation.
- `src/layouts/`: home, documentation and general landing shells.
- `src/navigation/`: the implemented page tree; generated page output is
  derived from this tree rather than maintained as a second list.
- `styles/`: split visual responsibilities compiled into one static asset.
- `scripts/build.ts`: host adapter that reads canonical examples and compiler
  metadata, invokes the Seseragi generator and publishes static files.

The complete planned reference hierarchy and per-page coverage obligations
live in `docs/design/docs-site/reference-content-map.md`. `check:site` validates
that ledger independently from the currently implemented vertical slice.

## Commands

```sh
bun run check:site
bun run build:site:production
```

Run `seseragi lock update apps/site` explicitly after changing Seseragi source
or package inputs.
