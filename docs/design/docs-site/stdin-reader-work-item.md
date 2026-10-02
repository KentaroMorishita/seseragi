# Stdin12 reader task

Status: owned authoring, execution checks and independent full-body reviews
passed; shared aggregate integration and publication are separate. This is a bounded bilingual
reference batch for ordinary TypeScript readers. It is not whole-site reader
acceptance or publication approval.

## Scope

Author exactly twelve existing destinations, in Japanese first and then English:

- `std/stdin` module
- `std/prelude::Stdin` and `std/prelude::StdinError`, owned by `std/stdin`
- `std/stdin::readLine`, `LineLimit`, `lineLimit`, `defaultLineLimit`,
  `readLineWith`, `StdinConfigError`, `InvalidStdinUtf8`, `StdinLineTooLong`
- `std/prelude::readLine`

The eleven `std/stdin` destinations retain their process-only catalog target.
The Prelude convenience function retains its process/browser metadata. Exact
identity, owner, namespace and kind guards distinguish aliases and homonyms.
Routes use the canonical owner, not the prefix of an aliased identity.

## Reader task

Feed a short label through a pipe; read one line; distinguish a blank line from
EOF; then validate a byte limit and handle rejected input before reading the
next line. Each body contains a complete standalone TypeScript comparison,
complete Seseragi source, concrete input and output, a process launch recipe,
local explanations of `attempt`/`match`, and links using actual API titles.

The ordinary TypeScript comparison uses Node/Bun `readline`. Limited examples
count bytes after `readline` supplies a string; they do not claim Seseragi's
strict raw UTF-8 decoding, bare-CR policy or bounded line buffering. The large
proof-only byte reader is not teaching material. First-line reading and
asynchronous whole-input collection are different tasks.

## Host and error boundaries

- Build process sources with the verified official release CLI and launch its
  declared `entry.js` under Node or Bun. Pipes supply only synthetic bytes. Do
  not introduce application argument forwarding through `seseragi run`.
- Suppress browser launch links for qualified sources, including pure
  configuration examples importing the process-only module. Successful
  standalone-file web bundling does not establish a qualified stdin host.
- Only the unchanged Prelude `one-line` source has public Playground proof.
  Source links do not prefill Input; describe the finite text to enter. The
  browser cannot demonstrate arbitrary malformed UTF-8 bytes or OS stdin.
- Qualified `StdinError` lacks a working `Show` instance in the verified release
  observation. Match public constructors and retain a plain fallback. This is
  separate from Prelude `readLine`, whose exact `Show` fallback passed.
- An empty closed pipe is EOF; LF/CRLF alone is a blank line. Rejected lines are
  consumed through the terminator or EOF. Offsets are absolute byte positions;
  `limitBytes` is the configured maximum, not the observed length.
- Handled read failures print a report and exit successfully. Uncaught failure
  stderr/exit status is separately tested and must not be implied by a caught
  report.
- The exact `Stdin` reading override explains a runtime-supplied service with
  no public constructor. The independently observed `StdinError` and
  `StdinConfigError` tail contradictions receive two separately approved exact
  overrides explaining public constructor patterns. Preserve canonical
  declarations and all 1,809 unrelated fallback readings.

## Ownership and acceptance

Owned files are the `stdin-reader` editorial family, canonical examples,
example helper, one guarded reading helper, source and rendered-body tests,
fixtures, this work item and the verification record. Shared registration,
reference wiring, canonical main imports, locks and aggregate integration
remain separately coordinated.

The final owned run passed 14 tests and 4,485 assertions. Exact sources and
input/output bytes ran on Node/Bun, displayed TypeScript passed strict checks,
and focused negative diagnostics, repairs, portable committed-WASM/browser
execution and package target boundaries were retained. All 24 initial bodies
were read independently; after corrections, all 24 complete final bodies were
reread with no remaining findings. Source/body hashes, six unrelated article
controls and failed attempts are preserved. The reader ledger requires its
separate approved append.

The eight qualified Console destinations remain gated backlog. Streams,
chunks, cancellation, concurrent reads, providers, compiler/runtime fixes and
publication are outside this batch.

## Evidence

- Feasibility: `/workspace/shared/seseragi-stdin12-plan-20261002`
- Authoring: `/workspace/shared/seseragi-stdin12-authoring-20261002`
- Baseline commit: `0c24d253d5469f1c8cae8c1ec8220310ef9aaf4a`

Retained feasibility is supporting evidence, not acceptance of newly authored
bodies or edited samples.
