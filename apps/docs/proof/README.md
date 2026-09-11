# Static generation proof (#592)

Run `bun apps/docs/proof/check.ts` from the repository. `SESERAGI_BIN` optionally
selects the canonical CLI. Requires installed Seseragi and Bun; the checker
creates a unique temporary directory and removes it even on failure.

The Seseragi process program composes and renders two complete HTML documents.
The host checker publishes the two output records to `index.html` and
`language/index.html`, checks exact escaped output, development/release parity,
repeated execution and repeated release build identity. `pages.json` accounts
for the published files separately from the generator's artifact manifest.
The line-based protocol is only a bounded proof using fixed single-line content;
it must be replaced by structured page records for arbitrary authored content.

## Decision evidence

Verified 2026-09-11 with released CLI/runtime 0.61.9 and Bun 1.3.9. Both pages
were byte-identical across all three builds and two executions per build.
Page sizes: 170 and 160 bytes. Client JavaScript: 0 bytes. Generator release
sizes: 1,520 generated TypeScript bytes, 104,310 bundled JavaScript bytes,
45,438 minified JavaScript bytes. These numbers describe this small probe only.
The generator runs during the build and is not a website download dependency.

SSG starts as an application generator command using canonical `process` builds.
There is no reproduced need for a new compiler target or web prerender phase.
`html.html/head/title/body/main` plus `renderToString` already provide document
composition, text escaping and structural landmarks. Only the constant doctype
prefix is concatenated; content is serialized through Html.

This closes the initial feasibility question, not the production Docs contract.
Phase C must replace fixed page mapping with validated structured records,
route/base handling, content and navigation, metadata and static assets.
Reference, client attachment, browser/accessibility review, corpus quality and
deployment remain later leaves. No compiler gap was reproduced by this probe.
