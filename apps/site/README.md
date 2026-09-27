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
- `src/reference/`: compiler-owned Standard Library modules and symbols are
  projected into module sections and individual Reference pages in Seseragi.
- `styles/`: split visual responsibilities compiled into one static asset.
- `scripts/reference.ts`: validates and joins the canonical Reference and
  Prelude instance artifacts without owning routes or page prose.
- `scripts/build.ts`: host adapter that reads canonical inputs, invokes the
  Seseragi generator and publishes static files.

The complete planned guide hierarchy and per-page coverage obligations live in
`docs/design/docs-site/reference-content-map.md`. `check:site` validates that
ledger and also verifies the complete compiler-owned Standard Library artifact:
63 module pages and 1,812 symbol and instance pages in both locales.

## Commands

```sh
bun run check:site
bun run build:site:production
```

Run `seseragi lock update apps/site` explicitly after changing Seseragi source
or package inputs.

## Vercel ownership

Until the root-site cutover is completed, the two Vercel projects have separate
responsibilities:

- the repository root `vercel.json` continues to deploy the Playground to the
  `seseragi` project and `seseragi.vercel.app`;
- `apps/site/vercel.json` is selected explicitly when deploying this site to
  the existing `seseragi-docs` project.

The site build uses `https://seseragi-docs.vercel.app` as its default canonical
origin. An explicit deployment can override it with `SESERAGI_SITE_ORIGIN`.
The Playground link remains `https://seseragi.vercel.app/` until its durable
post-cutover route is implemented in the cutover task.

The separate project can be deployed from the repository root without linking
the checkout away from the Playground project:

```sh
vercel deploy --project seseragi-docs --local-config apps/site/vercel.json
```

Do not promote this artifact to the `seseragi` project. Moving the language
site to the root domain and relocating the Playground are one coordinated
cutover, not part of the site-shell implementation.
