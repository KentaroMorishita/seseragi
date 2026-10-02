# JSON reader verification

Status: bounded authoring, executable evidence and independent model-reader
review complete. Whole-site integration/publication, real browser UI checks and
actual first-time-human reader feedback remain separate.

## Scope and provenance

The twenty exact existing identities are listed in `json-reader-work-item.md`
and `apps/site/scripts/json-reader.ts`. All forty locale bodies use the existing
typed Block renderer. Canonical compiler metadata/signatures and runtime sources
are unchanged. The batch introduces no new reference route.

Execution uses official Seseragi 0.61.19 (release commit a8641b5a81a4, Linux x64,
Unicode 17.0.0), Bun 1.3.9 and TypeScript 5.8.3. Native examples are copied to
isolated directories before lint/run, avoiding site/example package lock churn.
The exact displayed Seseragi bytes are used in the Playground source URL and
compiled through the committed WASM artifact.

## Final code and meaning checks

The final exact-source focused command passes **11 tests, 4,623 assertions**:

```
bun test apps/site/tests/json-reader.test.ts apps/site/tests/json-reader-editorial.test.ts
```

The run uses the verified release CLI through `SESERAGI_BIN`. Evidence and renders
can be retained with `JSON_READER_EVIDENCE_DIR` and `JSON_READER_RENDER_DIR`.

- Twelve complete canonical native programs lint, format-check and execute;
  twelve ordinary TypeScript counterparts strict-check and print the same output
- All twelve exact Playground source seeds compile and execute through committed
  WASM and the browser runtime execution adapter. Their entry contracts require
  only Console and no installed provider
- Twenty-four invalid sources each produce one intended diagnostic; three focused
  repair programs execute successfully
- A separately strict-checked current-runtime harness makes thirty-four
  assertions about paths, first failure, record ordering, optional/null, callback
  selection, defects and representative numeric/parser differences
- A separate input variation exercises the parse-error example's duplicate-field
  branch without pretending native JSON.parse has the same duplicate policy
- All forty complete bodies retain their exact displayed sources/output panels,
  canonical signatures, identity guards, source seeds, title-labelled links and
  same-identity locale navigation
- The reading overlay changes only four JSON type identities plus record. Its
  full-catalog controls find two changed English readings and five changed
  Japanese readings among 1,812 reference symbols
- The DecodeError kind-label wrapper preserves English and unrelated labels;
  collision controls and both complete rendered module/API pages verify it

All 104 owned or narrowly authorized Seseragi source files pass final format
checks. The focused host TypeScript check uses the site's established flags;
example comparisons and the runtime harness use strict checking. Biome and
`git diff --check` pass.

Negative fixtures cover unavailable Float/BigInt codecs, missing JsonDecode
without deriving, text passed to decodeJson, the wrong Decoder input type,
heterogeneous record decoders, raw pairs passed to JsonObject, private
DecodeError construction/patterns, and absent Eq/Show/Debug for the five JSON
error/path families. Repairs use a Map for JsonObject, Decimal or an explicitly
chosen JSON String for numbers, and a domain-owned Either for product rules.

## Fair TypeScript contracts

Both versions validate the intended data and handle their stated failure cases.
Comparisons use the same displayed inputs. They do not claim universal behavior
for every JSON text or TypeScript number. Some TS examples directly parse fixed,
known-valid text; their local explanation states that boundary instead of
claiming malformed external text is handled.

Separate executed probes establish differences for repeated keys, exact large
and fractional decimals, UTF-8 byte offsets and integer-looking object-key
order. Native JSON.parse followed by an integer guard can accept a lexical
fraction already rounded to an integer. These details support relevant limits,
not a large theory block before the first useful settings example.

## Complete-body independent reading

Two independent model readers read all twenty English and all twenty Japanese
bodies, including both complete source panels, output and generated declaration
reading. They did not supply missing explanations from source implementation or
the specification. They found two specific contradictions:

1. The generic opaque-struct label said DecodeError's fields were private, despite
   readable path/kind fields. A symbol-aware label wrapper now changes only the
   exact DecodeError identity's Japanese label, in the module list and API panel
2. record's generated reading described its array-of-pairs argument as a function.
   An exact function-identity overlay now explains the array and the checker in
   each pair, preserving the canonical signature

The DecodeError generated reading was also clarified to distinguish traversing
its path segments from matching its reason. The corrected production render
changed exactly five article bodies: English record/DecodeError and Japanese
record/DecodeError/std/json. Each affected body was independently reread in full.
The other thirty-five bodies remained byte-identical, and the final exact rerun's
forty bodies match the reviewed snapshot. All forty clear this model-reader
review. Actual first-time-human acceptance remains unverified.

Two bounded advisory Japanese prose passes were run on rendered prose with code
excluded. Both report the same 116 style findings: 107 repeated polite endings,
eight contrast constructions and one module-list ratio warning. These are
retained intentionally: precise definitions keep consistent language, genuine
distinctions remain explicit, and the generated API inventory remains complete.
The reports are advisory, not a readability score or substitute for full reading.

## Tooling attempts and corrections

- An initial lint invocation inside the canonical example package encountered an
  unrelated existing unused binding. Exact-source copies were then checked in
  isolation; unrelated source was not changed
- An exploratory stricter-than-site host check reached an existing reference
  narrowing issue. The runtime harness was separated into its proper strict lane;
  both that lane and the established site host flags pass without runtime edits
- The first family dispatcher used a long nested identity conditional and took
  roughly four minutes to compile. Shallow owner/namespace-kind dispatch and
  small exact-identity helpers reduced the same isolated selection check to a few
  seconds. The final render/guard stages remain bounded and took about 32/36
  seconds, without relaxing compiler behavior or raising the final stage bounds
- One test-only label probe attempted field access without importing its result
  representation. The probe now uses the public locale translate operation and
  passes; the rendered page checks had already passed
- Final canonical formatting normalized the dispatcher closing indentation and
  seven negative fixtures. The complete final suite was rerun afterward

Earlier feasibility authoring attempts, diagnostics, source/HTML hashes and
execution records are retained separately. No compiler/runtime fix or public bug
claim is implied by this documentation work.

## Remaining acceptance boundaries

The focused renderer includes the full JSON module and the four selected prelude
leaves; it does not substitute for full-site reference ordering or integration.
Native/WASM execution does not prove a real clicked browser session. Whole-site
registration and lock integration, desktop/mobile layout, actual Playground
navigation/execution, publication checks and genuine first-time-reader feedback
remain explicitly separate. No completed code or model-reading check bulk-
accepts the remaining JSON constructors, combinators or codec instance pages.
