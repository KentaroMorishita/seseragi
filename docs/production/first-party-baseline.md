# First-party production baseline (O02)

Measured on 2026-09-09 using the O02 implementation stack, compiler/runtime
version 0.61.8, Bun 1.3.9, macOS arm64. These are local Impl measurements, not
published-release or installed-dogfood evidence. #554 must validate the terminal
SHA through the full gate, WASM, canonical release and local dogfood before O02
closes or #533 begins.

## Artifact measurements

| Application | Source modules | Runtime modules | Generated TS bytes | Bundled JS bytes | Minified JS bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| minimal | 1 | 16 | 290 | 25,705 | 10,988 |
| console | 1 | 23 | 302 | 88,395 | 37,947 |
| collections | 1 | 24 | 2,390 | 91,946 | 39,347 |
| reachability | 3 | 23 | 1,688 | 89,677 | 38,346 |
| web-starter | 2 | 53 | 5,338 | 1,235,973 | 1,043,791 |
| flow-app | 9 | 54 | 33,589 | 1,254,813 | 1,053,176 |

[Machine-readable baseline](first-party-baseline.json) records the complete
retained Seseragi module list, official runtime modules with retention reasons,
source-map policy, build identity and measured sizes for every application.
The budget contract lives in `examples/spec/fixtures/production/budgets.json`.
Source/runtime allowlists detect reintroduced modules even when the byte budget
still passes. Budget headroom is intentional; these are not exact-byte snapshots
across platform or tool versions. A reviewed feature change may update its
allowlist/budget, with the manifest diff explaining the new roots.

## Behavioral evidence

`CARGO_INCREMENTAL=0 bun run check:production` passes all six builds, repeated
output-location determinism, file inventory and SHA-256 identity validation,
source-map omit policy, required initializers and allowed source/runtime sets.
The four process applications also run in development and release with identical
exit status, stdout and stderr. The CLI production suite independently covers
failure dictionaries, normal-source stack/release-shape fixtures, Unicode
source-map composition, and emit/omit parity. The original #342 compiler-stage
shape checks remain independent; bundler output is never used to establish those
compiler semantics.

`bun run check:production:browser` runs Chromium against canonical CLI-built
Web starter and multi-module Flow app in both development and release. It checks
source-map defaults, starter counter updates, Flow theme switching, focus
add/remove, empty-form validation, story addition/editing and draft reset. Final
visible text and input values match across profiles; no JavaScript page errors
occur. Four full-page screenshots and JSON evidence are produced under
`target/production-browser`. Third-party Unsplash photography is replaced by a
deterministic image in this offline verification only.

Visual inspection confirms working controls and visible state. Flow's canonical
CLI build currently emits default CSS plus its inline styles; its Playground
utility classes have no utility stylesheet in the standalone artifact. Both
profiles share this existing presentation limitation. This report does not claim
visual equivalence to Playground. No separate calculator application is present
in this checkout; the collection process sample covers the small pure/data case.

The regression adds a dead module and an unused math-backed export: generated
files, source inventory, runtime inventory and all sizes remain identical. Making
that math feature observable alongside the existing answer retains the declaration
and `@seseragi/runtime/math`, increases the minified size and changes the output.
This demonstrates an attributable feature cost rather than silent artifact growth.

## Known retained roots and costs

The application roots are `main` and its required failure-display dictionary,
with compiler-resolved references retaining helpers and dependencies. Runtime
module reports distinguish pure helpers, startup-required initializers,
provider/resource bootstrap and entry-owned behavior. Hash entropy and Unicode
version initialization remain required and covered by startup tests. Provider
setup/cleanup is not annotated away. Foreign packages remain outside this gate.

Web currently retains a common browser provider dispatcher. Its switch references
all supported adapters, including timezones; the timezone database accounts for
much of the approximately 1 MB minified baseline even in the starter. This is
reachable code, not an unexplained external package. Specializing provider
dispatch could be a later optimization only with initialization and cleanup
parity evidence. A blanket package `sideEffects: false` is not used.

## Reproduction and #533 handoff

Install root and Playground dependencies with their frozen lockfiles once, and
install the pinned Playwright Chromium browser via
`cd apps/playground && bunx playwright install chromium`. Then run:

```sh
CARGO_INCREMENTAL=0 bun run check:production
bun run check:production:browser
```

Both lanes run in the full and release source gate. `SESERAGI_BIN` can select a
packaged CLI; output paths can be overridden with `SESERAGI_PRODUCTION_OUTPUT`
and `SESERAGI_PRODUCTION_BROWSER_OUTPUT`. The full terminal gate remains #554's
responsibility; a local scoped pass is not release acceptance.

#533 should regenerate these measurements with the accepted O02 release, then
record Docs/SSG modules, runtime reasons and sizes separately. #463 can subsequently
distinguish first-party pipeline cost, Docs/SSG application cost and external
package/binding cost. Neither downstream queue is started by this handoff.
