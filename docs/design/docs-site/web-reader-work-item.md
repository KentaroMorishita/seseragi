# Local work item: reusable HTML card and browser entry

This is a local draft for the existing fifteen-route Web reader batch. It is not
an external issue, publication record or first-time-reader acceptance.

## Reader task

An everyday TypeScript developer builds one release-note card, reuses it twice,
prints escaped HTML, then optionally connects a Show details button through
`dom.app`. The reader should distinguish an `Html` description, a serialized
`String`, returned DOM work and its actual execution without a Tour prerequisite.

## Exact scope

The complete identities, namespaces, kinds and existing routes are recorded in
`apps/site/scripts/web-reader.ts` as `webReaderRoutes`. The fifteen entries are:

1. `std/web/html` module
2. `std/web/html::Html`, type / opaque-type
3. `std/web/html::ElementProps`, type / alias
4. `std/web/html::trait(IntoChildren)`, trait / trait
5. `std/web/html::section`, value / function
6. `std/web/html::h2`, value / function
7. `std/web/html::p`, value / function
8. `std/web/html::text`, value / function
9. `std/web/html::fragment`, value / function
10. `std/web/html::button`, value / function
11. `std/web/html::attribute`, value / function
12. `std/web/html::renderToString`, value / function
13. `std/web/html::renderDocument`, value / function
14. `std/web/dom` module
15. `std/web/dom::app`, value / function

No `/docs/applications/web/...` page exists in this implementation; none is linked
or counted. Other HTML/DOM/Signal leaves, CSS, core language pages, compiler,
runtime and normative specification remain outside the batch.

## Deliverables and checks

- [x] Japanese-first and English typed locale copy for all fifteen identities
- [x] Dedicated typed Block helper and identity/module/namespace/kind guards
- [x] Exact source/output panels and compiler-owned declarations preserved
- [x] Twelve native process examples, one browser app and three TS counterparts
- [x] Official CLI checks, strict TS comparators and committed-WASM execution
- [x] Positive correction and negative diagnostic fixtures
- [x] Browser-target artifact build and real generated host `#app` inspection
- [x] Exact compiled app provider mock, separately labelled from browser behavior
- [x] Real public Playground execution of the unchanged app source and four clicks
- [x] Thirty bounded production-rendered bodies, localized paragraphs and title links
- [x] Independent full-body EN/JA agent reader review and resulting corrections
- [ ] Whole-site integration gates and publication of this batch
- [ ] Desktop/mobile review of the published documentation routes
- [ ] Actual first-time TypeScript-reader feedback

The parent integrator owns shared registries, aggregate imports, lock updates,
normal site gates, publication and deployment QA. This authoring task creates no
commit, push, PR, issue or deployment.

## Reader acceptance questions

Ask an independent reader to change both card titles/details, explain the type
before and after `renderToString`, repair the mixed String/Html array using a
typed text local, explain why a serialized button cannot react, and find the
browser continuation without visiting Tour. Ask what `target`, `initial`,
`update` and `view` each supply and what fails when `#app` is missing.

Actual browser clicks on the tiny program do not establish that the documentation
is understandable, that browser cleanup/focus behavior is verified, or that the
whole Web library is reader-complete. Record these acceptance categories apart.
