# Bytes inspection11 verification

Status: bounded authoring checks passed in the cloud; Mac integration is pending.

## Exact candidate and transferred evidence

Draft PR #753 final HEAD `85e5de3867a97cee21de5695f85af4f926d74904`
was merged normally into the all-open-issues branch in `e0540f7c3`.
The final Library bundle `bytes-inspection11-mac-handoff.zip` is 1,199,925 bytes,
SHA-256 `e0521f133a349ba8ff60948965cd10bbd26192d2e887070a7bb36fbe9814a276`.
The Mac checked ZIP integrity and all 297 manifest entries before use. All 79
source files matched the integrated checkout except `apps/site/scripts/check.ts`,
which also retains the Mac's optimized current-source CLI selection. No bundled
source was copied over Git integration.

## Bounded results and their limits

The final cloud run passed 14 tests, 11,804 assertions and the configured site
TypeScript check using official CLI 0.61.19, Bun 1.3.9, Node 24.19.0 and TS 5.8.3.
It checked six exact standalone Seseragi/TypeScript pairs, format/lint/build and
process output, strict TypeScript and actual Node/Bun output, six committed-WASM
seeds with the current runtime in a host process, 31 boundary observations and
six precise rejection/repair pairs. That host process is not an actual browser.

All 22 EN/JA rendered bodies were read independently by model reviewers before
and after corrections. The final reading accepted all 22 complete bodies,
including generated declaration tails. The corrections explain Byte's absence
of a public constructor, the integer values bound by get's loop, and the public
InvalidByteRange constructor/pattern beside BytesSliceError's opaque declaration.
Equivalent English corrections are retained. This is model review, not evidence
from genuine first-time human readers.

Thirty unselected localized std/bytes bodies remained identical. Complete
63-module metadata comparison found only the intended BytesSliceError reading
change; all 1,811 other readings, signatures, targets and kinds were retained.
The exact scope remains eleven existing identities and 22 localized bodies, with
no new module or constructor page. The expected authored count is 355 identities;
Mac whole-site generation must establish that count afresh.

## Durable proof bindings

These paths are relative to the transferred ZIP's extracted root. Historical
cloud absolute paths in receipts describe provenance, not local execution.

| Proof | SHA-256 |
| --- | --- |
| `evidence/final/completion.json` | `1fc72552202c49be0abfb104ada6fc3dfedf0e78eebcc067f7126e3037ae39a0` |
| `evidence/final/reference-parity.json` | `336d4790cd1d627ffba907b2b60a26d74ce3681e9f6048adf86935d8bfc748cd` |
| `evidence/final/unselected-article-preservation.json` | `62a16856adb32d53a9687c3058b7c0a53a3d788ec0f939d6e20befb3dee6b730` |
| `evidence/final/actual-site-typescript.json` | `5c8b920e5749cdb86832a7a1ff3ee7ea6c3b6e16904d7de5f6b0397c5d793ed7` |

## Pending Mac gates

Refresh the two authorized package locks if needed, then run the unified
0.61.20 candidate through the complete site aggregate, two bounded full builds,
unchanged-site audit and actual browser review. Record current executable
versions and hashes. The cloud's official 0.61.19 results do not prove the new
compiler candidate. Merge, public deployment and human acceptance are not
claimed by this verification record.
