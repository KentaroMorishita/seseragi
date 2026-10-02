# Shareable URL reader batch

## Reader task

An ordinary TypeScript developer edits a shareable search link: replace a search
term, retain repeated selected tags in order, reset the page, and choose the
results section. Native URL and URLSearchParams are already concise. The
comparison explains Seseragi's checked URL result and updates returning new
values without pretending native TypeScript lacks encoding or duplicate support.

Every deep-link article explains the names and syntax it uses. Japanese is
authored first, then English. Familiar tasks precede representation details;
functional-programming terminology is not required.

## Exact existing scope

`apps/site/scripts/url-reader.ts` owns twenty existing identities and their exact
routes, kinds, namespaces, canonical source IDs and expected output:

- `std/web/navigation`
- `std/web/navigation::{Url, Query, UrlBuildError}`
- `std/web/navigation::{parseUrl, resolveUrl, renderUrl}`
- `std/web/navigation::{emptyQuery, parseQuery, appendQuery, setQuery, removeQuery,
  queryValues, queryEntries, renderQuery}`
- `std/web/navigation::{urlQuery, withQuery}`
- `std/web/navigation::{urlFragment, withFragment, withoutFragment}`

That is one module, three types, fifteen functions and one value: forty EN/JA
bodies supported by thirteen complete Seseragi/TypeScript source pairs. emptyQuery
is a value, not a function. Identity dispatch remains shallow
and guards owner, namespace and kind. No new route, compiler signature or runtime
behavior is introduced. Canonical constructor leaves, path operations, origin,
HTML conversion, navigation services/history, and Show/Debug instance pages
remain outside the authored count.

## Execution boundary

The canonical module is browser-only. Examples belong to the separate
`apps/site/examples/projects/url-reader` web-target package and must never be
imported into `apps/site/examples/src/main.ssrg` or the process-target aggregate.
Rendering source text in the process SSG does not import its browser module.

The examples calculate URL/query/fragment values and print through Console; they
require no Navigation/DOM/Storage service and never change live browser history.
Readers run the source in Playground or build it with `--target web`.

An isolated standalone-file process invocation happened to succeed in feasibility
checks. The equivalent package rejects process with SES-K0203 and builds for web.
That observed path difference does not add supported process availability.

## Contracts to preserve

- Absolute HTTP(S) parsing and explicit-base resolution differ from navigation
- Native TS counterparts use the same inputs, output and relevant policy; extra
  validation checks are only needed in examples comparing failure classifications
- Query order and duplicate names are retained; set replaces at the first old
  position; remove clears every matching name; empty and absent remain distinct
- Query strings use form-style encoding; decoded input is encoded once on render
- Parse errors use UTF-16 code-unit offsets; parseQuery strips its first leading
  question mark before computing the offset
- The current double-leading-question-mark behavior is bounded and documented;
  do not claim native URLSearchParams and parseQuery are identical for all strings
- Well-formed percent triplets do not establish valid UTF-8. Current fragment/path
  getters can throw URIError after parsing succeeds. This limit belongs after the
  useful case on relevant pages, not in every first paragraph
- Query extraction is detached; rebuilding replaces the entire query and preserves
  other components. Pure functions do not establish destination trust

Runtime/compiler repair and public bug reports are not part of this batch.

## Verification and acceptance

Exact-source web compilation, committed-WASM/current-runtime execution, strict TS
comparison, bounded runtime probes, precise invalid fixtures, production body
rendering and independent whole-body reading are separate gates. The verification
document records observed results rather than assuming plans have passed.

The public Playground feasibility proof is valid only for share-search-link
source SHA-256 `1f79418621f5d6575557ab9c9bfc488c23ddadf68a367ed989c51dea0ed79a59`.
Changed or additional examples require their own exact-source checks. One actual
host run is not blanket browser verification or a deployed-version match.

Whole-site integration, publication, UI checks and genuine first-time-human reader
feedback remain separate from bounded authoring and independent model reading.
