# Terminal startup and output reader tranche

## Reader and scope

An ordinary TypeScript developer wants to print a short message, assemble output,
show a count, read an optional setting or launch argument, find the working
directory, and handle a failed settings read. Japanese is authored first, then
English. Teach the familiar task and native Node.js/Bun TypeScript before the
Seseragi program and only the syntax the program uses.

This tranche contains nine existing destinations and eighteen locale bodies:

- Prelude println, print, printValue
- std/process entrance, Process, ProcessError, environment, arguments,
  currentDirectory

The exact metadata tuples and routes are recorded by
`apps/site/scripts/terminal-reader.ts`. The parent terminal plan retains the eight
std/console-owned destinations as a gated backlog: its entrance, Console,
ConsoleError, println, print, printValue, error and errorLine. Console/ConsoleError
have Prelude canonical identities but std/console ownership. They are not added
here. Qualified error-type interchangeability, failure rendering and provider
inference caveats remain for a later decision. No compiler/runtime fix, new API,
stdin content, signals, error-constructor page or navigation expansion is in scope.

## Canonical examples and runtime boundaries

Seven complete Seseragi/TypeScript pairs cover the visible stories. The three
Prelude sources are portable and receive exact-source Playground seeds. The four
Process sources explicitly declare Console and Process services, use one local
attempt/match boundary, and receive no browser launch link. Their examples print
status reports to stdout; they are not stderr demonstrations.

- println adds a newline; print and printValue do not. printValue uses Show rather
  than JSON. Browser capture trims trailing whitespace, so raw process output is
  the newline reference
- The TypeScript process.stdout.write examples target Node.js/Bun, not browsers
- A setting that is present but empty is distinct from an absent setting; defaults
  apply only when absent. Programs read only named synthetic setting variables
- Arguments are read after launching the declared release entry.js. The CLI run
  parser does not forward extra application arguments or accept a -- separator
- currentDirectory reads the launch working directory and returns Path. The
  controlled POSIX-shell demo uses /tmp/seseragi-process-demo; it does not read
  arbitrary file contents or promise symlink resolution
- Empty/NUL setting names exercise actual ProcessError handling. Encoding error
  names do not establish strict Unicode validation or every-host reproducibility
- Output reporting can itself fail with ConsoleError after a ProcessError has
  been handled. No application-wide error ADT is needed for these local examples

## Ownership and verification

The terminal-reader family/model/catalog, seven example pairs, helper, dedicated
tests/fixtures and verification/work-item files belong to this tranche. Shared
catalog, build/check/reference adapters, canonical main and package locks remain
parent-owned. No commit or publication is performed by this authoring task.

Use the verified v0.61.19 release CLI for isolated source builds and rendering;
never build the repository compiler as a convenience. Record exact source hashes,
commands, controlled environments, separate streams, statuses, failed attempts and
session IDs. Strict TypeScript comparison and Node/Bun execution are distinct from
committed-WASM/current-browser-path proof and real public browser observations.

Independently read all eighteen initial whole bodies and reread all eighteen final
bodies after corrections. Assert exact dispatch, rejected owner/kind/namespace
lookalikes, examples/output hashes, locale parity, actual absence of Process
Playground links, current titles and return navigation. Reader judgments are
separate from compilation, advisory prose scores and parent integration gates.
