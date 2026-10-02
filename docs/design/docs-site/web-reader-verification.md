# Reusable release-note card: bounded Web reader verification

## Scope and ownership

Authoring began at clean local HEAD
`73239319b69a16ae0a7749594cc37dae9c6f3de7`. This batch changes fifteen existing
identities only: the HTML and DOM module introductions, HTML's `Html`,
`ElementProps`, `IntoChildren`, `section`, `h2`, `p`, `text`, `fragment`, `button`,
`attribute`, `renderToString`, `renderDocument`, and DOM's `app`. Their exact
identity/namespace/kind/route tuples live in `webReaderRoutes` in
`apps/site/scripts/web-reader.ts`.

Thirty Japanese/English bodies live in
`apps/site/src/reference/editorial/web-reader/`. Japanese was written first using
the reader contract and pinned yomiyasu tech guidance; English carries the same
inputs, outputs, restrictions and execution boundaries. The helper uses the
existing semantic Block model. Exact identity matching plus module, namespace
and item-kind guards select each leaf. Module selection uses exact specifiers.
Shared compiler-family summaries and declarations are not rewritten. Shared catalog/build/check registrations and aggregate examples are verified
in the separate integration checkpoint.

No new route, absent application page, CSS change, core/compiler/runtime edit,
external issue, commit, push or publication is part of this authoring task.
The previous HEAD's separate publication is not evidence for this uncommitted
batch. All other Web and Signal reference leaves remain outside this scope.

## Reader task and semantics

The entry starts with one paragraph and explicit output, then reuses an ordinary
card function twice. `Html` is a pure description rather than a String or live
Node. `renderToString` supplies escaping at serialization; `println` supplies
output. `renderDocument` prepends one doctype and does not synthesize
html/head/body. Those existing destination pages are optional, accurately titled
links. Module introductions retain generated import/availability/API content.

Each deep-link article explains its local necessary syntax and mistake. The
static card uses typed local heading/description values; mixed content uses a
typed local text value. These are real restrictions of tested CLI 0.61.19:
unanchored nested children fail with `SES-T0201`, mixed String/Html children
fail with `SES-T0101` (with a follow-on requirement diagnostic), and an inline
text call in the corrected mixed shape also needs a local type anchor. The
working forms are executed; no compiler diagnosis or fix is claimed.

`ElementProps` explains required children, optional fields, omitted versus False
versus True boolean attributes, and rejection of String `hidden`. `IntoChildren`
covers Unit, String, one Html, Array<Html> and List<Html>, preserving source order
without implicit display conversion. `attribute` consumes both Right and Left,
uses the validated attribute in a real element and displays a reserved-name
failure. It does not attempt to Show an Attribute or claim arbitrary CSS/URL
validation. Static button output proves the default type, disabled/ARIA behavior
and omission of the event payload.

The browser continuation locally explains ToggleDetails, its one match branch,
Bool state, pure update/view and `pub effect fn main = dom.app`. The existing
`#app` target is explicit. app returns work requiring the DOM service, with
String failure and Unit success; creating that work is not its execution. The
module distinguishes pure content/binding/target descriptions from returned
query/run/mount/app work. Source/spec ownership of internal Signal, default
mounting and cleanup is stated without counting mock cleanup as real-browser
cleanup. Effectful actions/custom options/lifetime control remain optional links
to existing query/run/mount/signal pages, without a mandatory reading sequence.

Normative traceability: `docs/spec/13-web-ui.md` (HTML, children, props, SSR,
DOM/app and lifetime), compiler-owned standard-library schema, and current
`runtime/ts/src/{html,dom}.ts`. The planning record is
`/tmp/seseragi-web-reader-plan.rHOhjL/plan.md`. Its older style-signature and Debug
prose discrepancies are intentionally not taught, changed or “resolved” here.
Final executable sources, rather than old planning outputs, supply the panels.

## Fair TypeScript comparisons

The SSR comparison uses an ordinary ReleaseCard interface, a card function,
`escapeText` and array mapping/joining. It has byte-identical output for the same
two cards, including < and &. It does not compare against unsafe interpolation or
pad TS with classes. Prose explicitly credits TS template libraries with
providing escaping and browser `textContent` with safely setting text.

The interactive TS version uses createElement, textContent, a boolean, one click
handler, matching hidden/aria-expanded/labels, and an explicit unmount function.
It is strictly typechecked; it has not been executed in a real browser for this
batch. The separate pure TS snapshot program matches the Seseragi
False→True→False output exactly. Neither the comparator typecheck nor those
snapshots are presented as TS browser-event evidence.

## Code verification

Official CLI: 0.61.19, release commit `a8641b5a81a4`, Linux x86_64,
`/tmp/seseragi-first-run-699.oOVHG8/bin/seseragi`. Bun 1.3.9; TS 5.8.3.
The canonical examples are twelve process programs and one standalone web app,
plus three TS comparison files. Source files feed display, hashes and launch URLs.

Focused example tests plus existing HTML/DOM runtime tests: **42 pass, zero fail,
487 assertions** after restoring the browser-verified app launch link. They cover:

- Bounded isolated native runs, exact stdout, formatting and lint
- Three native negative fixtures and executable positive corrections
- Strict TS checks for all three comparison programs
- Native/TS exact card and repeated-toggle snapshot output parity
- All twelve exact process Playground seeds through committed WASM and the
  in-process execution harness
- The exact browser app seed through committed WASM and a DOM-provider mock
- Real standalone CLI web artifact build, manifest and generated `#app` host
- Mock dispatch of the actual compiled app's button action twice; target selector,
  state snapshots, missing target, mounting failure and owned unmount callback

These mock tests do not open a browser. Source-runtime and released-native
boundaries are distinct. Logs and retained browser artifacts are under
`/tmp/seseragi-web-reader-proof.bKZdAN/`; the final example log is
`final-bounded-dependency-tests.log`.

The first test version statically imported the complete runtime module registry,
which expanded the site-only TypeScript gate into unrelated baseline runtime
diagnostics. The final test uses URL-based dynamic imports for the execution-only
HTML/DOM/registry boundary while preserving the actual implementations and all
mock assertions. Typed Effect/ServiceResult/Signal imports and a small local
opaque provider contract describe the harness boundary. The exact site-style
TypeScript command now passes for the helper and both new tests without changed
compiler options. Expanded diagnostics and matching HEAD/worktree blob IDs for
all eight implicated files are retained as `expanded-runtime-dependency-typecheck.log`
and `expanded-dependency-boundary.txt`; full-runtime typecheck acceptance is not
claimed. No runtime file was changed.

## Actual browser evidence

Separate public Playground browser verification exercised exactly
`release-card-app.ssrg` SHA-256
`66a926fbb2d3a76f6fafccf9409521b4482a173c480957b7fecc526947c2559b`.
The source URL loaded the program; Run switched to Interactive and Preview.
The rendered title retained literal `Release <notes>`. Four clicks verified
Show→Hide→Show→Hide→Show, paragraph visibility and expanded/collapsed accessibility
state each time. No site/runtime console errors were observed in that check.

This narrowly supports restoring the exact unchanged app's canonical launch URL.
The article explains Run and Preview and also retains the native CLI build path.
Screenshots are
`/workspace/scratch/677b5e8ef737/seseragi-web-probe-expanded.jpg` and
`/workspace/scratch/677b5e8ef737/seseragi-web-probe-collapsed.jpg`.
No localhost route, tunnel, file URL, access bypass or local preview server was
used. Browser focus, listener teardown, mobile app behavior, TS DOM execution,
other Web programs and browser rendering of this documentation batch are not
established by that probe.

## Rendered bodies and remaining acceptance

`apps/site/tests/web-reader-editorial.test.ts` checks the exact 13 leaves and 2
module guards against mutated identities/modules/namespaces/kinds, every
unselected HTML/DOM/Signal control, and the production renderer's relative import
closure. All 30 locale bodies retain exact own-locale paragraphs without
other-locale leakage, canonical declarations, source/output panels, exact source
launch URLs, owning-module round trips, locale round trips, same-topic previous/next links,
and **116**
purpose-labelled title links against actual destination H1s. Deliberately wrong
link titles are rejected. No route stub is substituted for those destinations.
The bounded catalog still is not a full-site navigation or layout test.
The post-reader editorial suite passes **2 tests, zero failures, 1,579 assertions**.
Its own scoped site-style TypeScript check, Biome and Seseragi format checks pass.

Latest rendered bodies and review manifest are under
`target/web-reader-review/`. The independent agent reader completed a first full-body pass and requested
corrections. The refreshed full-body reread is clear; neither agent review nor author checks
are actual first-time-reader feedback. The local work-item draft is
`web-reader-work-item.md`. Full-site gates, production publication and
mobile/desktop documentation review require separate verification and remain pending in this scoped record.

Japanese lint runs in the foreground on fifteen selected rendered articles,
excluding code panels. Its advisory findings are mainly repeated polite endings,
explicit technical contrasts (Html versus DOM, markup versus events) and the
module's generated API list. These are not acceptance scores. Necessary negative
examples and execution distinctions are retained; the generated API list is not
rewritten to reduce a prose-list metric. At most two advisory passes are made.


## Independent-reader corrections

The first independent pass read all thirty complete bodies and separately
checked 42 source panels, 36 launch seeds, 38 output/build panels, 26 declarations,
116 title/purpose links and all locale counterparts. It found no artifact drift,
but found real local-reading gaps. The revision:

- Adds the verified typed text local, including the CLI 0.61.19 restriction, to
  the HTML entrance, p and fragment deep links
- Uses explicitly illustrative ElementProps fields instead of pointing to a
  question mark absent from its compact alias declaration; locates the
  IntoChildren where clause on element functions rather than its trait page
- Explains Action, C, IntoChildren and record fields locally on the affected
  element/fragment pages, and calls the h2 page’s function as `heading ()`
- Gives every static deep link Open in Playground → Run → Output → Text steps,
  using verified current UI labels rather than directing SSR strings to Preview
- Corrects the DOM module’s above/below reference and Japanese TS boolean names
- Clarifies that Unit’s value () carries no extra information, separately from
  these particular static examples binding no handlers; Unit does not prohibit
  event payloads, as the unchanged button program demonstrates

The new manifest is `target/web-reader-review/body-manifest.json`, SHA-256
`a263439cfa04c6d3ce7d11f0eb1f020d245c81b96a9a202bb2ee5eb42237f709`.
`reader-correction-body-diff.json` preserves exact old/new artifacts and hashes:
29 full bodies changed and English app is byte-identical. On every route, all
preformatted source/output/declaration panels and article link destinations are
byte-identical. `reader-correction-example-integrity.json` verifies all sixteen
canonical Seseragi/TS files are unchanged, including the browser-proved app.
The original manifest and artifacts remain under `before-independent-corrections/`.

The combined post-correction run passes **44 tests, zero failures, 2,066
assertions**. It re-executes the official native examples, strict TS comparisons,
WASM, web compilation, actual-runtime provider mocks and all rendered-body checks.
Scoped site-style TypeScript, Biome, Seseragi formatting and diff whitespace checks
pass. See `post-reader-tests.log` and `post-reader-typecheck.log`. Two Japanese
lint advisory passes preceded these targeted reader corrections; no third scoring
pass was added to chase warnings. Final independent rereading was completed separately, as recorded below.


### Independent reread result

The independent agent reader completed the original thirty-body pass, then read
all twenty-nine changed full bodies after corrections. The unchanged English app
body independently matched its saved baseline. All reported reader findings are
resolved and the bounded agent review is clear. This is an agent review against
the ordinary TypeScript-reader contract, not first-time human feedback.

The reader independently verified the fresh manifest above and
`reader-correction-body-diff.json` SHA-256
`893e5b660d71f57acc0265d870a454e4d94a068fb97c206ea53e18ee4be508d8`,
plus `reader-correction-example-integrity.json` SHA-256
`1631f60e00dc9a93534779f5a017f90435b6e7d17c460041acfddbf580059687`.
All 106 preformatted panels and 244 article-link destinations remain byte-identical;
all sixteen programs remain unchanged. The 116 current title/purpose links and
thirty locale counterpart links pass the independent checks. No reviewer source
or ledger edit was made. Production documentation desktop/mobile review,
whole-site integration acceptance and actual first-time-human feedback remain
pending and are not inferred from this result.
