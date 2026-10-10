# Seseragi / design prototype (vertical slice)

This PR is an independently designed, browser-useful sample of the future Docs, not a migration of the old Docs page system.

Pages: Home, Docs index, four short language articles (functions, pattern match, composition, Effects), a human-curated notation reference and Examples. Japanese and English each have their own static routes (16 total).

Use the repository's existing official logo without redesigning it. Full runnable sample links seed the Playground from existing repository sources; shorter article snippets illustrate sections of those sources. They have not been independently compiler-verified as standalone files.

Functionality: responsive nav and local chapter menu, same-page anchors, quick search/filter for article and notation entries, copy-code controls, language switch and precise links to canonical sample source and Playground. Static links and content work without JS; search/copy are progressive enhancements.

Intentionally deferred: API corpus generation, content migration, rewriting the compiler/SSG, 3,976-route acceptance/regression tests. The objective is to evaluate the product experience end to end before choosing the final implementation architecture.

Important: this branch swaps apps/site/vercel.json to serve *only this static preview*. Do not merge that Vercel config into main without deliberately migrating the runtime/build. Existing Docs production and Playground must remain intact while reviewing.
