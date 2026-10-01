# Library landing: task selection and module directory

Author verification on 2026-10-01, based on local checkpoint
`6bd210c7903b9c98b1de9f924daa750c191640dc`. This is a bounded change to the
existing `/docs/library/` and `/ja/docs/library/` landing. It does not accept
all generated module/API articles or establish first-time-reader acceptance.

## Changed reader surface

The landing retains its identity, title, `collections` anchor, 12 existing
`directory-*` anchors, and 63 module destinations. Its Japanese-first paired
copy now starts with six practical choices:

1. Find, transform, or combine collection elements.
2. Process text or read JSON.
3. Check numeric results or work with bytes.
4. Keep explicit state or coordinate concurrent work.
5. Use files or run processes.
6. Build a web interface or use the network.

Task links jump to existing directory groups. Each module destination appears
once in that directory, with the module page's existing localized purpose and
the canonical module target list. The body retains the existing `Array.get`
example link and adds purpose-labelled links to seven core explanations and
First Run. Their visible labels match the current destination titles.

This is a selector, not a new tutorial or a second example registry. It adds
no runnable code panel and makes no new source-execution claim. It does not
require a Tour or sequential reading course. It assumes familiar JavaScript /
TypeScript functions, arrays and strings, then briefly explains modules,
Maybe alternatives, checked result alternatives, Effect construction, and
services where those distinctions affect the choice.

## Semantic sources and boundaries

- `crates/seseragi-project/src/standard.rs`: canonical module target and service
  metadata. The target labels are `process` and `browser`; they are not an
  unconditional promise that a host supplies every service.
- `examples/spec/artifacts/stdlib-schema-1/reference/module.json`: the actual
  63-module inventory and public signatures.
- `docs/spec/10-library-surface.md`: module/provider distinctions, numeric and
  byte operations, JSON, state and filesystem contracts.
- `docs/spec/05-effects.md`: Effect construction versus execution and services.
- `docs/spec/13-web-ui.md`: pure HTML/SVG values versus live browser operations.
- Existing typed `reference/module-copy.ssrg`: the purpose shown beside each
  module link, reused rather than duplicated. Independent reading led to two
  approved description-only corrections: List is called immutable in English,
  and Japanese browser storage copy includes both read and write failures.

Specific distinctions preserved in both locales:

- Array/List/Map/Set updates produce new values. A missing lookup result is
  different from a typed failure reason.
- Int and Float are separate; checked numeric operations do not all return
  Either. `Nothing`/`Just` and `Left`/`Right` result shapes remain distinct.
- Parsing JSON and decoding the required field shape are separate steps.
- Ref, Queue, Deferred and Semaphore use Effect; constructing work does not
  execute it or turn ordinary immutable values into mutable objects.
- `std/fs` is available for both targets but requires FileSystem. Browser
  eligibility does not grant arbitrary operating-system filesystem access.
- `std/process`, `std/stdin`, `std/child-process`, `std/http/server` and
  `std/websocket/server` are process-only.
- HTML/SVG construction supports both targets; `std/web/dom`, navigation,
  storage and file are browser-only. HTTP sending requires HttpClient.

## Renderer boundary

Owned implementation files:

- `apps/site/src/pages/library/overview/{page,en,ja}.ssrg`
- `apps/site/src/components/reference-directory.ssrg`
- `apps/site/src/layouts/documentation-landing.ssrg`
- `apps/site/styles/docs.css`
- `apps/site/src/reference/module-copy.ssrg` (the two approved strings above)
- `apps/site/tests/library-landing.test.ts`

The landing layout enables directory details only for `library.overview`.
Other landing pages retain the prior plain-link directory markup. A Boolean
selects the detailed rendering branch; passing a conditional empty array of
ReferenceModule values exposed an existing nominal type-identity mismatch
while assembling the change, so that intermediate form was removed.

Styling retains the existing directory grid/card layout and adds only link,
purpose and target text treatment. No editorial catalog, canonical metadata,
compiler, runtime, shared test registry or lock was changed by this task.

## Executed checks

Environment: Bun 1.3.9, Linux x64, published CLI
`seseragi 0.61.19 (release, commit a8641b5a81a4, target x86_64-unknown-linux-gnu)`.

```sh
source /workspace/scratch/677b5e8ef737/seseragi-env.sh
CLI="$(cat /tmp/seseragi-first-run-699-path)/bin/seseragi"
SESERAGI_BIN="$CLI" \
  SESERAGI_LIBRARY_LANDING_OUTPUT=target/site-library-landing-review \
  bun test apps/site/tests/library-landing.test.ts
```

Result: **1 test passed, 772 assertions, 9.35 seconds**. The test compiles and
runs the real production page/layout/directory modules with all 63 canonical
modules, using empty symbol lists to bound the render. It checks:

- Both localized landing documents, title/locale identity and unique headings.
- All 63 unique directory links in all 12 groups, with exact destination,
  localized purpose and canonical targets associated with each module.
- All original directory anchors plus the six task sections and context.
- Actual group-anchor destinations and group labels; exact current titles for
  eight core/First Run links.
- No duplicate module destination list in the authored body.
- FileSystem/HttpClient and target-versus-service caveats in the rendered text.
- Byte-for-byte prior plain-link directory markup on a second, unrelated
  synthetic landing using the same full module input. It receives no purpose
  or target additions.

Scoped TypeScript typecheck, Biome checks for the new test and changed CSS,
Seseragi formatting checks for the six changed `.ssrg` files, and
`git diff --check` passed. Parent integration owns registry insertion, locks,
complete catalog generation and the full non-browser gate.

## Rendered reading and limits

Retained output:

- `target/site-library-landing-review/docs/library/index.html`
- `target/site-library-landing-review/ja/docs/library/index.html`

The author read both complete rendered bodies and all displayed purpose/target
pairs. An independent agent also read both complete bodies without using source
files to fill explanatory gaps. It found no explanation blocker and requested
the two purpose-copy corrections described above. Japanese advisory lint ran on the rendered prose, then once after the two
review corrections (two passes total), excluding code:
`target/site-library-landing-yomiyasu.json`. Two sentence-ending repetition
warnings and one immutable-value contrast were retained: the parallel
technical statements and the difference from familiar mutation are intentional.
The advisory score is not a reader-acceptance result.

The focused catalog contains real directory metadata but no symbol leaves or
full core navigation. It proves rendered structure/text and link labels, not
whole-site route closure, a real click round trip, desktop/mobile appearance,
keyboard behavior, overflow or browser execution. Browser access remains
blocked; no visual/browser pass is claimed. Independent body review and any
later complete-site gate must be recorded separately.
