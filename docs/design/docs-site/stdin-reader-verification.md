# Stdin12 verification

Status: owned authoring verification passed. Aggregate integration, reader-ledger
acceptance and publication remain separately coordinated.

## Exact scope and execution

The twelve existing destinations and all canonical identities, owners,
namespaces, kinds, signatures and targets are preserved. `Stdin` and
`StdinError` retain `std/prelude` identities and `std/stdin` ownership. The
eleven qualified-module destinations stay process-only; only Prelude
`readLine` has a source-seeded Playground link.

Six complete Seseragi/TypeScript pairs are rendered in 24 EN/JA bodies. The
final combined run passed **14 tests, 4,485 assertions**, with **209 recorded
subprocess commands**. Commands include expected negative observations and
must not all be described as successful exits.

- Official CLI 0.61.19, SHA-256
  `987b0834c1ac299d0e4922e33160a324fa1d85dde909867b15660bac6e9d53b7`
- Node v24.19.0, Bun 1.3.9 and TypeScript 5.8.3
- All six exact native sources formatted, linted, built as release process
  entries and executed with synthetic input under Node/Bun
- All six displayed TypeScript sources checked with `strict`,
  `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes`, without skipped
  library checking, then executed under Node/Bun
- Synthetic blank/EOF, LF/CRLF, final unterminated lines, Unicode byte limits,
  rejected-line recovery, absolute malformed-byte offsets, exact/default limits,
  caught and uncaught failures, configuration boundaries and diagnostic repairs
- Actual POSIX octal `printf` bytes and the displayed malformed-input pipeline
- Six bounded open-pipe observations separating first-line reading from
  whole-input collection, with child timeouts and controlled pipe closure
- Qualified package process success and web rejection; standalone web bundle
  success retained separately, without claiming a browser input host

Every test child receives only synthetic allowlisted environment settings.
Tests do not inspect real settings, secrets or arbitrary input files. The
independent strict byte-policy helper is fixture-only. Ordinary native
`readline` replacement of malformed UTF-8 and bare-CR splitting are captured
as differences, not false parity with Seseragi.

## Reader review and retained behavior

All **24 initial complete bodies** and **24 final complete bodies** were read
independently, including code, commands, output, links and canonical tails.
Japanese review identified two error-tail contradictions; the exact
`StdinError` and `StdinConfigError` readings now explain public constructor
patterns. Together with the approved host-supplied `Stdin` correction, exactly
**three readings change in each locale**. All **1,809 other readings** and all
signatures/classifications remain unchanged.

The pure configuration pages now use validation/run headings without implying
stdin consumption. Their displayed TypeScript validator distinguishes unsafe
or fractional numbers from nonpositive byte counts, while preserving the five
documented outputs. Multibyte output parity is also executed for the displayed
TypeScript limit example.

Rendered tests verify exact source panels, localized prose, real API titles,
lowercase routes, locale/topic/pager journeys, and browser-link suppression on
all **22 process-only locale bodies**. Six unselected EN/JA article controls
match checkpoint16 byte-for-byte. Final accepted rendering matches all 24
independently reread full HTML/body snapshots exactly.

All 38 family modules pass format checks. Owned site-flags TypeScript, Biome
and diff checks pass. Static inspection found no stale existing global stdin
absence assertion; no unrelated family guards were changed.

## Browser evidence boundary

The portable Prelude source remains byte-identical:
`3da2a0a10804c02c391cee2ec589983dcb4370168566c8f5b175a1111a5dcd15`.
Seven finite-text cases pass through the committed WASM and current production
browser-execution path, with supplemental raw callback checks. The previously
recorded public Playground observations of `parcel`, one newline and empty
Input remain applicable to that exact source. This batch claims no new public
browser session or deployed compiler/runtime version.

Source links do not prefill Input. Browser text input does not prove arbitrary
malformed raw bytes, OS stdin, concurrent reads or interactive terminal behavior.
Captured browser output does not prove stderr separation. The eight qualified
Console destinations remain gated separately.

## Evidence and limitations

Authoring root: `/workspace/shared/seseragi-stdin12-authoring-20261002`.
Retained feasibility: `/workspace/shared/seseragi-stdin12-plan-20261002`.

- `accepted-tests/run-EGvVA4`: exact command/input/output records and source hashes
- `accepted-tests/render-QN520V`: copied production renderer closure, 1,066 sources
- `accepted-tests.log`, `final-test-summary.json`, `final-tool-receipts.json`
- `initial-body-manifest.json`, `final-body-manifest.json`, and both rendered sets
- `english-reader-review.json`, `japanese-reader-review.json` and final rereviews
- `reading-change-audit-final.json`, `final-source-output-manifest.json`
- `final-unselected-controls.json`, `reviewed-final-body-parity.json`
- `prior-public-playground-observation.json`, `retained-plan-integrity.json`

Failures and superseded attempts remain available: a missing isolated package
lock, one model array-bracket error, a runner PATH missing Bun, a direct leaf
lint timeout, and test import ordering rejected by Biome. Each was corrected
or replaced by the relevant passing isolated check; the timed-out process was
terminated. No repository compiler/runtime fix, dependency or repository lock
change, commit or publication was performed by this batch.
