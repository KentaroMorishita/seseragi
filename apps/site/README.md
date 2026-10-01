# Seseragi official site

`apps/site` is the first-party Seseragi static-site generator for the language
home page and documentation. Page content is authored as typed Seseragi
modules; Markdown, MDX and JSON page records are not part of the rendering
pipeline.

## Responsibilities

- `src/pages/`: one directory per stable page, with `page.ssrg`, `en.ssrg` and
  `ja.ssrg`. All explanation prose lives in those locale modules as typed
  `ExplanationCopy`; the article summary reads the same explanation copy, so
  it cannot drift into a second translation. A small `guide.ssrg` selects the
  example with `ArticleExample` or `WithoutExample` and pairs the locale copies. The
  shared explanation component moves that article's existing example beside
  its explanation; it never substitutes a Tour lesson or removes later rules.
- `src/model/`: closed page, navigation, locale and build-input types.
- `src/components/`: reusable documentation patterns such as code examples,
  callouts, API references, sidebars and in-page navigation.
- `src/layouts/`: home, documentation and general landing shells.
- `src/navigation/`: the implemented page tree; generated page output is
  derived from this tree rather than maintained as a second list.
- `src/reference/`: compiler-owned Standard Library modules and symbols are
  projected into module sections and individual Reference pages in Seseragi.
- `styles/`: split visual responsibilities compiled into one static asset.
- `client/`: browser-only enhancements. Mobile navigation uses a native modal
  dialog for focus isolation, Escape and backdrop dismissal, and preserves the
  reader's scroll position. The same Seseragi-rendered tree remains an off-canvas
  sidebar without JavaScript; no client routing or second content tree is used.
  The globe language picker is native details with same-page locale links;
  its enhancement only adds outside/Escape dismissal and focus handling.
- `scripts/reference.ts`: validates and joins the canonical Reference and
  Prelude instance artifacts without owning routes or page prose.
- `scripts/reference-copy.ts`: Japanese-first reader descriptions and their
  reviewed English counterparts, keyed by exact canonical descriptions. The
  original metadata remains intact. Unknown upstream copy fails the build;
  it never silently falls back to untranslated copy.
- `src/reference/instance-copy.ssrg`: explains what each standard trait
  implementation lets the reader do, together with conditional constraints.
- `scripts/signature-reading.ts`: derives argument and result explanations
  from canonical signatures, keeping nested function arrows distinct from
  top-level arguments. These explanations supplement, not replace, API prose.
- `scripts/build.ts`: host adapter that reads canonical inputs, invokes the
  Seseragi generator and publishes static files.

The complete planned guide hierarchy and per-page coverage obligations live in
`docs/design/docs-site/reference-content-map.md`. `check:site` validates that
ledger and also verifies the complete compiler-owned Standard Library artifact:
63 module pages and 1,812 symbol and instance pages in both locales.

Use each destination page's localized title for page-name links, including
related rules and design principles. Do not introduce a second translation or
shortened name in a link. `check:site` compares standalone language-reference
links against the generated destination headings in both locales and exercises
the design-principle links in desktop and mobile browsers.

## Commands

```sh
bun run check:site
bun run build:site:production
```

Run `seseragi lock update apps/site` explicitly after changing Seseragi source
or package inputs.

## Japanese editorial checks

Write Japanese first using yomiyasu's technical-writing guidance, then revise
English from the same explanation. Both locales explain the same rules; neither
is a reduced summary. See `docs/design/docs-site/reader-contract.md` for the
editing and reader-acceptance criteria.
Start with what the reader can do and a concrete example, then explain the
rule and its limits. Introduce technical terms where they are first needed;
do not stack compiler-internal terms or untranslated English nouns into prose.
Keep source keywords, API names, type signatures and diagnostics unchanged.
Explain why rejected examples fail and what the reader can change.

Review page copy together with code captions, related links and sidebar labels.
Read the rendered page in order: a passing build or a phrase assertion does
not establish that the explanation is understandable. Keep current compiler
limitations explicit and distinct from the language specification.

After building, run the installed skill's bundled lint against every rendered
Japanese page, excluding code blocks and protecting inline code:

```sh
python3 apps/site/scripts/check-prose.py target/site \
  /path/to/yomiyasu/scripts/yomiyasu_lint.py
```

This foreground command emits an advisory JSON report and starts no background
process. A lint score does not establish reader acceptance. Review findings in
context, with at most two editorial correction passes; preserve legitimate
technical restrictions and examples.

## Vercel ownership

Until the root-site cutover is completed, the two Vercel projects have separate
responsibilities:

- the repository root `vercel.json` continues to deploy the Playground to the
  `seseragi` project and `seseragi.vercel.app`;
- the existing `seseragi-docs` project is connected to this repository with
  `apps/site` as its Root Directory, so it reads `apps/site/vercel.json`
  without ever reading the root Playground configuration.

The site build uses `https://seseragi-docs.vercel.app` as its default canonical
origin. An explicit deployment can override it with `SESERAGI_SITE_ORIGIN`.
The Playground link remains `https://seseragi.vercel.app/` until its durable
post-cutover route is implemented in the cutover task.

The separate project deploys `main` through its Git integration or the
`Docs production` deploy hook. For an explicit CLI deployment, run Vercel from
the same `apps/site` root used by the project:

```sh
cd apps/site
vercel deploy --project seseragi-docs
```

Do not promote this artifact to the `seseragi` project. Moving the language
site to the root domain and relocating the Playground are one coordinated
cutover, not part of the site-shell implementation.
