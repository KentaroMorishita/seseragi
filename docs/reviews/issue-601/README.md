# q1 Docs rework: current-state audit and IA benchmark

2026-09-24 snapshot of `origin/main` at `7ef669897d0f3188188459f4a3fdc21c7dd0d0fa` for [#601](https://github.com/KentaroMorishita/seseragi/issues/601), section A. This is an information-architecture decision record, not the final site map, concept inventory, or approval to generate content in bulk.

## Current product and evidence

| Boundary | What exists | Reader / authoring consequence |
| --- | --- | --- |
| Human content | [`pages.json`](../../../apps/docs/content/pages.json) holds 15 authored pages: 17 headings, 22 paragraphs, 28 links, 17 canonical samples, and 5 terminal blocks. Eleven pages are non-section content pages; most of those have only 4–6 blocks. | The baseline demonstrates routes and examples but does not yet answer one reader question thoroughly per page. The collection count is not a quality measure. |
| Page model | [`prepare`](../../../apps/docs/scripts/build.ts) accepts a single array with id, route, title, summary, and ordered blocks. Its authored block vocabulary is paragraph, heading, terminal, sample, and link. | Prose, code and links are validated, but the model cannot express prerequisites, design rationale, cross-language comparisons, warnings, availability, related/next steps, or a concept identity. Writing and structural editing require direct JSON edits. |
| Navigation | [`render.ssrg`](../../../apps/docs/src/render.ssrg) gives every authored page `navigation: true` and renders every such page plus the Reference landing in one sidebar list. Module Reference pages are excluded from that list. | There is no section-scoped sidebar, breadcrumb, on-page outline, or explicit previous/related/next route. Module pages have no contextual return path to the Guide beyond the global Reference landing. |
| Mobile | [`docs.css`](../../../apps/docs/public/docs.css) changes the layout to block at 700 px and turns the full sidebar into a two-column grid before `<main>`. | The reader must pass the complete navigation before each article. This works for a small baseline but scales poorly as content grows. |
| Locale | [`render.ssrg`](../../../apps/docs/src/render.ssrg) fixes `<html lang="ja">`; navigation labels, search strings and prose are Japanese. [`quality.ts`](../../../apps/docs/scripts/quality.ts) requires `lang="ja"`. | No locale-aware page identity, `/ja/` or `/en/` route, same-page language switch, localized search, or `hreflang` contract exists. Adding English text alone would not establish bilingual Docs. |
| Reference | [`build.ts`](../../../apps/docs/scripts/build.ts) validates and renders the compiler-owned [`module.json`](../../../examples/spec/artifacts/stdlib-schema-1/reference/module.json), currently 63 modules and 1,455 items, plus a Reference landing page. Search is derived from those same validated records. | Preserve this source of truth and deterministic generation. Add human-facing paths into and back out of Reference rather than re-authoring signatures in prose. |
| Published quality | The [architecture contract](../../../apps/docs/ARCHITECTURE.md) and build/quality scripts cover canonical example source digests, links, fragments, SEO, static no-JS content, search/copy enhancement and budgets. | Keep these gates. They establish technical correctness, not conceptual coverage, prose quality or locale parity. |

The authored and generated records imply 79 routes today (15 authored + one Reference landing + 63 modules). Search generation implies 1,534 entries (79 pages + 1,455 items). The older [Docs README](../../../apps/docs/README.md) and [`browser.ts`](../../../apps/docs/scripts/browser.ts) still say 78 routes / 1,465 entries. These are **source-derived audit figures**, not a fresh production build result; the q1 implementation must reconcile the fixed expectations with a verified build before using them as a gate.

## Official documentation benchmark

Checked on 2026-09-24. The point is to borrow recognizable reader journeys, not another project's visual design or page count.

| Product | Observed reader journey | Seseragi implication |
| --- | --- | --- |
| [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html) | Entry paths acknowledge different prior backgrounds; the Handbook offers a progressive explanation, while its Reference section is a deeper, non-sequential lookup path. | Start with TypeScript/JavaScript mental-model bridges in context, and keep a clear Guide/Reference separation. Rust/Haskell bridges should be available at the relevant concept, not become the default first page. |
| [Rust Book](https://doc.rust-lang.org/stable/book/ch01-00-getting-started.html) and [standard-library docs](https://doc.rust-lang.org/std/) | The Book advances from setup through concrete programs; the library docs provide searchable module/type/API details. | Give newcomers a working first program and progressive Guide path; leave symbol contracts to compiler-owned Reference. |
| [React Learn](https://react.dev/learn) and [API Reference](https://react.dev/reference/react) | Learn foregrounds everyday concepts and task-oriented examples; Reference is explicitly a detailed lookup surface. | Explain Signal, pure Html and DOM Effect with outcomes and examples before sending readers to exact APIs. |
| [Next.js Docs](https://nextjs.org/docs) | Getting Started, Guides and API Reference are distinct top-level tasks; Guides address practical use cases. | Separate language concepts from application recipes, with few global entries and focused section navigation. |

## Decisions to carry into sections B–F

1. Adopt familiar global destinations: **Getting Started**, **Guide**, **Guides**, and **Reference**. Keep Tour, Playground, Spec, Articles and Internals connected according to their distinct roles; decide their exact placement only after the concept inventory and site map.
2. Use a small global nav and a current-section sidebar. Provide a breadcrumb and local heading outline on article pages; express prerequisites, related material and next steps from concept/page metadata. On mobile, keep the article ahead of the full section tree behind a compact navigation control. This is an IA requirement; the exact component design belongs to section D.
3. Define Guide pages around one primary reader question and its prerequisites, example, rationale, common mistake and onward links. Keep precise signatures, constraints, availability and identity in Reference. Never infer or copy canonical signatures into the authoring source.
4. Maintain **two different structures**: a navigable page tree and a concept dependency/related graph. A symbol or operator is placed by meaning, not by its token spelling. A concept may be explained on one Guide page and linked from several journeys.
5. Preserve canonical `examples/spec` source references, Playground links, compiler-owned Reference generation, deterministic SSG, accessibility, SEO and no-JS baseline. These are useful O03 assets, not reasons to keep the current JSON authoring shape.
6. Give each page and concept a stable, locale-independent identity. Japanese and English share structure and source/example/Reference identities but own natural titles, summaries, prose and navigation labels. Do not route by translated title. Section F must decide missing-translation and fallback policy before implementation.
7. Do not choose Markdown, Seseragi, or a hybrid yet. First define a validated semantic Doc IR and authoring ergonomics against actual Guide/Reference examples; then select the least awkward source format in section E.

## Next evidence needed

- Section B: inventory canonical spec, standard library, tooling, Web, Provider and official packages; mark available / contract-only / internal and record provenance for each concept.
- Section C: assign reader questions and prerequisite/related/next edges before creating a complete site map.
- Sections D–F: test the proposed navigation, authoring and locale contracts against one complete bilingual Guide page plus its Reference links, not a mass-generated corpus.
- Section G: recalculate route/search expectations from the actual validated corpus and compare with a fresh build; remove stale fixed totals in tests and prose.
