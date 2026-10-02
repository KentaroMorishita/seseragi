# Terminal output and startup-input verification

## Scope

Nine existing destinations, eighteen EN/JA bodies, seven complete paired
Seseragi/TypeScript examples. Exact routes are in `terminal-reader.ts`.

- Three portable Prelude output functions: println, print, printValue
- Six process-only destinations: std/process, Process, ProcessError,
  environment, arguments, currentDirectory
- Eight qualified std/console destinations remain gated outside this tranche;
  no stdin, signal, compiler/runtime or provider implementation change

## Executed source and reader-body checks

The focused suites are `terminal-reader.test.ts` and
`terminal-reader-editorial.test.ts`. The final combined run passed **10 tests,
3,401 assertions** with the supplied official Seseragi v0.61.19 release CLI,
Node 24.19.0 and Bun 1.3.9. There are 75 recorded source-verification subprocess
commands. Builds, package closure staging and their locks remain in isolated
verification directories, not repository package roots.

- All seven exact Seseragi examples pass formatter checking, deny-warnings lint,
  release process build and separate stdout/stderr assertions on Node/Bun
- All seven displayed TypeScript files pass strict TypeScript 5.8.3 checks,
  including unchecked-index and exact-optional-property checking, then produce
  the same tested outputs under Node/Bun
- Synthetic environment cases distinguish preview, present-empty and missing.
  Application argv preserves spaces, empty text, flag-like text and Japanese
  text, plus the no-argument case
- currentDirectory is executed from the controlled `/tmp/seseragi-process-demo`
  directory. The tests do not read its contents or remove a pre-existing folder
- Empty/NUL names exercise actual ProcessError matching. A Maybe-as-String
  rejection is paired with a complete executable repair
- Release manifests declare entry.js. CLI run rejects both extra positional
  application arguments and a -- forwarding separator. Run/web rejection and
  build/web missing-Process-capability rejection are recorded separately
- Every process subprocess receives an explicit small infrastructure environment
  plus named synthetic variables, not an inherited environment dump
- Every Process example has standalone=false. Rendered tests verify zero
  source-seeded browser links across all twelve Process locale bodies

## Portable execution and browser limits

The exact three portable Prelude sources compile through the committed
v0.61.19 WASM artifact and execute through the unchanged production
browser-execution path under Bun. Supplemental runs retain raw current-browser-
host callbacks to prove newline behavior that the display path trims. Source,
seed, generated TypeScript and implementation hashes are retained.

All four Process source programs compile under WASM but execution rejects the
unsupported `@seseragi/runtime/process` import. That does not make them browser
programs. Console capture combines normal and error writes; no browser stderr
separation is asserted.

An earlier coordinated public Playground observation verified one exact
combined Prelude output source. That observation supplements the committed-
WASM proof; it is not a public-browser run of every current page, a deployed
compiler-version assertion or a fresh site-layout review.

## Metadata, prose and navigation

Exact dispatch guards reject wrong owners, namespaces, kinds, identities and
qualified Console lookalikes. Only std/process receives the new module body.
Production rendering verifies every full body, actual sources and output
panels, local launch commands, locale links, exact next-operation titles and
module return journeys.

Independent initial Japanese and English whole-body readings found the generic
Process opaque-type instruction misleading; Japanese also found the ProcessError
access guidance contradictory to its public pattern matching. Two fully guarded
site-owned reading overrides correct those tails. Comparing all 1,812 records
before/after shows exactly those two reading texts changed, with all 1,810 other
records and every signature/classification unchanged.

The TypeScript samples were subsequently repository-formatted and all exact
changed sources rerun. The cwd demonstration directory was made neutral instead
of carrying a batch-number label. Final independent whole-body rereads cover all
nine articles in each locale, including the entire corrected declaration tails.
These are simulated reader judgments, not observed human usability feedback.

## Evidence and remaining gates

The artifact bundle records initial/final full-body manifests, source/output
hashes, compiler responses, CLI commands, clean environments, separate streams,
failed attempts, session identifiers and per-route initial/final reader reports.
Test-harness errors and integration-format/typechecking attempts are retained,
not reported as passes. Owned helpers/tests pass the existing site TypeScript
flags and Biome after avoiding unintended static runtime-source inclusion.

The bounded review-ledger append preserves the full prior prefix and all historical
checkbox states. The parent owns broad site integration, CSS/mobile browser checks,
canonical main and shared registrations, commit and publication. No
repository compiler build, compiler/runtime fix or public issue was part of this
bounded authoring task. The remaining Console backlog is preserved in the work
item and the original seventeen-destination planning evidence.
