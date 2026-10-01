# Integer counts and decimal measurements: bounded reader verification

## Scope and ownership

This documentation batch starts at checkpoint7
`15bd8858becffa32d19f55f66d630593ac90c603` on `docs/697-reader-contract`.
It authors thirteen existing identities in Japanese and English:

- `std/int` and `std/float` module introductions
- `std/int::{parse,format,checkedAdd,checkedSubtract,checkedMultiply,checkedDivide,checkedRemainder}`
- `std/float::{parse,format,fromInt,isFinite}`

The 26 typed locale bodies live under
`apps/site/src/reference/editorial/numeric-reader/{int,float}/`.
The dedicated `model.ssrg` uses the existing semantic Block renderer and typed
CorrectionCopy fields; `catalog.ssrg` selects complete identity, owner, value
namespace and function kind. It does not change a shared compiler description.
All callable declarations and module availability/target metadata remain
compiler-owned. The module introductions retain the generated import guidance
and public API index. Parent-owned integration registers the catalog, examples,
tests and aggregate imports.

There are 58 Int/Float/Math callables in the compiler inventory. Eleven receive
these overlays; the other 47 remain outside this batch. `std/math` is entirely
unchanged. The two Float rounding operations, `toInt` and `roundIntegral`, are
explicitly gated rather than counted as reviewed. The existing built-in-types
core pages, compiler, runtime and normative specification are unchanged.

## Reader and comparison choices

The task sequence starts with settings/form counts, checked count calculations,
and decimal measurements. Each deep-link page explains its own required syntax:
imports, whitespace-separated calls, function arguments and return annotations,
local result alternatives, match/if branches, and the execution wrapper.
Just/Nothing and Right/Left are explained as returned alternatives rather than
as assumed FP terminology or as thrown exceptions. No Rust, Haskell or ADT
background is required.

Thirteen standalone Seseragi examples and thirteen TypeScript counterparts live
under `apps/site/examples/{src,comparisons}/api-numeric-reader/`.
`apps/site/scripts/numeric-readers.ts` records each exact output separately.
The canonical source file supplies the displayed source, hash and Playground
seed. TypeScript source panels have no Seseragi launch link.

Comparisons use whole-string regex/Number checks, ordinary numeric calculations,
String(value) and Number.isFinite. The integer division counterpart uses native
BigInt conversion and division so it does not inherit rounded floating-point
quotients near the safe-integer boundary. Its precondition is stated locally:
inputs are already safe integers. The integer-parser comparison requires the
matched text to equal the entire input, so JavaScript `$` matching before a final
newline cannot accidentally broaden accepted input. Float parsing explicitly
rejects surrounding whitespace before its regex and numeric conversion.

Eleven pairs have identical display output. Float `format` and `fromInt` retain
honest spelling differences: TypeScript String(number) omits `.0`, loses the
negative-zero sign, and uses `1e+21`; Float format preserves `-0.0` and uses
`1e21`. Both outputs are shown and separately verified. The prose acknowledges
that TypeScript is already concise where applicable. Seseragi's benefit is the
visible Int/Float distinction, explicit conversion and named/absent returned
failure, rather than shorter code or general superiority.

## Semantic boundaries

- Int is exactly -9007199254740991 through 9007199254740991, with no fraction,
  NaN, infinity or negative zero. Negative integers are valid; positivity is a
  separate application condition
- Int parse consumes the complete untrimmed decimal spelling and rejects extra
  leading zeroes, suffixes, underscores and literal prefixes. EmptyInt,
  InvalidIntDigit and IntOutsideRange are explained locally; the shared
  InvalidIntRadix alternative belongs to radix-taking operations. Error offsets
  are UTF-8 byte positions, not visual character positions
- Mixed syntax/range failures do not promise a universal priority between error
  alternatives. The observed out-of-range-plus-suffix result is retained
- checkedSubtract takes the amount subtracted first. checkedDivide and
  checkedRemainder take divisor before dividend. Quotients truncate toward zero;
  nonzero remainders follow the dividend's sign and exact zero is normalized
- checked addition/subtraction/multiplication return Nothing on overflow. Checked
  division/remainder return Left IntDivisionByZero on a zero divisor. These
  returned values require caller branches and do not clamp, retry, or impose
  positive-quantity rules
- Float parse distinguishes explicit NaN/Infinity spellings from finite numeric
  overflow. Underflow may succeed as signed zero; successful parsing need not
  yield a finite measurement
- Float format is canonical round-trip text, not localized or fixed-place
  display. fromInt is exact for every Int; later Float arithmetic may round
- isFinite excludes only NaN and infinities. It does not establish positivity,
  integrality, the Int range or exact decimal arithmetic

Normative traceability is `docs/spec/10-library-surface.md` §10.8, the canonical
stdlib reference artifact, and current `runtime/ts/src/{int,float,number}.ts`.
Planning inventory/proofs are retained under `/tmp/seseragi-numeric-reader-plan`;
planning executions do not stand in for final example verification.

## Rounding limitation kept visible

The planning probes confirmed a current HalfUp defect: values immediately below
0.5 can round upward, some large already-integral values change, and toInt can
reject an in-range endpoint after incorrect rounding. The Float module discloses
this verified limitation in both locales and labels its links to the existing
rounding declarations accordingly. It does not teach those outputs as intended
semantics. No runtime fix, public issue, compiler rebuild, release or semantic
acceptance of either gated page is included.

## Verification boundaries

The official CLI is 0.61.19, release commit `a8641b5a81a4`, Linux x86_64, at
`/tmp/seseragi-first-run-699.oOVHG8/bin/seseragi`. It is the published compiler,
not a binary rebuilt from the documentation checkout. Bun is 1.3.9, TypeScript
5.8.3 and Biome 2.1.1. Native-driver execution runs generated programs; it does
not prove an independent native arithmetic backend.

The focused suite is `apps/site/tests/numeric-readers.test.ts`. Its checks cover:

- Every final Seseragi example linted and executed from an isolated temporary
  directory, with exact stdout and empty stderr
- All thirteen TypeScript examples strictly checked together as independent
  modules and executed against their own documented output
- Every exact Playground seed compiled through the committed WASM artifact and
  executed in the existing runtime harness
- Bounded current-runtime assertions for lexical rejection, safe-range endpoints,
  operand order, signed/truncated division, normalized zero, Float spelling,
  overflow/underflow, exact integer conversion and finite classification
- A canonical, formatted native fixture with 47 labelled boundary observations
- Fractional checkedAdd input and an Int returned as Float rejected with
  SES-T0101; the successful examples demonstrate the valid Int or explicit
  fromInt alternatives
- Eleven exact callable selectors, five identity/metadata variants for each,
  47 untouched callable controls and exact module-name selection
- Both production locale bodies for all thirteen identities, including complete
  copy pairing, canonical declarations, source panels, exact output panels,
  exact Playground source bytes, locale round trips and owning-module links
- 74 authored title links checked against actual rendered H1 titles, with
  deliberately wrong-title mutation tests; gated rounding pages remain controls

The renderer copies only the production import closure into isolated temporary
packages. Its temporary lock update is local to those packages. This is bounded
rendered HTML evidence, not a full site build or browser/layout acceptance.

An initial static-import test typecheck traversed unrelated runtime modules and
reported existing errors in foreign.ts, http-client.ts and web-file.ts. All three
files were verified byte-identical to the base commit. The evidence records exact
diagnostics and blob IDs in
`target/numeric-reader-review/expanded-runtime-static-check.txt`. The test now
loads the same current runtime modules dynamically and executes the same bounded
assertions. A scoped docs/test TypeScript pass is not a full runtime typecheck.

## Final results and review

The frozen-source focused run passed **six tests, zero failures and 1,486
assertions** in 183.24 seconds. The exact log is retained as
`target/numeric-reader-review/final-tests.stderr`. Scoped docs/test TypeScript,
Biome, canonical `.ssrg` format checks and `git diff --check` also passed.
The 47-line native fixture and each final expected output were verified again;
old planning results are not counted as acceptance.

The complete local HTML draft is `target/numeric-reader-review/rendered/`.
`body-manifest.json` lists all 26 reviewed-scope routes, their actual HTML and
complete extracted article files, including generated API chrome. Source hashes
are in `frozen-source.sha256`. The manifest was sent to a separate agent reader
for Japanese-first, then English review. The separate reader read all 26 complete
bodies and identified two Japanese paragraphs that called Unit a value rather
than a type, plus redundant zero normalization after TypeScript BigInt division.
Both Japanese explanations now call () a value of Unit type, and checkedDivide's
TS counterpart directly interpolates the integer quotient. The checkedRemainder
counterpart retains its genuine JavaScript -0 normalization.

After those three corrections, the thirteen-example and 26-body production
checks were rerun: **two tests passed, zero failed, 1,244 assertions**, in 95.25
seconds. Four unaffected tests were filtered from this second run; the complete
six-test pass remains separate evidence. Corrected source hashes, HTML and
reading files replaced the prior local draft. The independent reader fully reread all four changed locale bodies, verified the
other 22 were byte-identical, and found no remaining bounded reader blocker.
Independent structure checks verified 74 exact-title links, 26 native
source-seeded panels, 26 TS panels, 52 outputs, 52 locale links, 22 canonical
callable declarations and 260 local fragment occurrences. The independent final
26-page HTML hash manifest is
`target/numeric-reader-review/independent-reviewed26-final.sha256`; its own
SHA-256 is
`621ea5a32193c1e46aad856732a8273c95db41401837d40f7a4e7ec774358923`.
Each sorted artifact-relative path is paired with SHA-256 of its complete HTML
bytes. This is separate agent reader review, not first-time human-reader evidence.

Japanese was authored first, following the pinned yomiyasu tech guidance at
`30ee6041c328ce21d38a7963f667e079a93d7a12`. Two foreground advisory passes over
rendered Japanese article prose excluded preformatted source/output and marked
inline identifiers. The final retained report has 43 polite-ending warnings and
seven technical contrast notes (the first pass had 41 and seven). No mechanical rewrite was made merely to clear them.
Necessary contrasts distinguish complete parsing from prefix parsing, returned
failures from thrown exceptions, conversion from parsing, and a Float's value
from its display spelling. Repeated polite endings in short step explanations
and purpose-labelled links remain deliberate. The shared colon-separated title
plus purpose format is preserved for navigation consistency. This is advisory
lint, not a readability score or reader acceptance.

No browser, desktop/mobile layout, first-time human-reader or full-site
acceptance is claimed by this batch. The checks launched no browser or long-lived
lint process.
